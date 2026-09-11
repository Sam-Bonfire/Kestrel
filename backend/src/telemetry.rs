use std::path::{Path, PathBuf};
use std::sync::Once;

static INIT_HOOK: Once = Once::new();

/// Directory holding local crash reports (`crashes/crashes.log`).
pub fn crash_log_path(data_dir: &Path) -> PathBuf {
    data_dir.join("crashes").join("crashes.log")
}

/// Derive the data directory from a `sqlite:<path>` database URL.
pub fn data_dir_from_database_url(database_url: &str) -> PathBuf {
    let path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
    Path::new(path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn format_panic_report(
    payload: &str,
    location: Option<&str>,
    thread: &str,
    timestamp: i64,
) -> String {
    format!(
        "--- panic at {} (thread '{}') ---\nlocation: {}\n{}\n",
        timestamp,
        thread,
        location.unwrap_or("unknown"),
        payload
    )
}

/// Append a crash report to the local log, creating directories as needed.
/// Local-only by design; reports may contain panic arguments, so redact
/// before sharing. Skips writes once the log exceeds 5MB.
pub fn write_crash_report(data_dir: &Path, report: &str) -> std::io::Result<PathBuf> {
    let path = crash_log_path(data_dir);
    if std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) > 5_000_000 {
        return Ok(path);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(report.as_bytes())?;
    Ok(path)
}

fn payload_to_string(payload: &(dyn std::any::Any + Send + 'static)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Box<Any>".to_string()
    }
}

/// Install a process-wide panic hook writing local crash reports.
/// Local-only by design: nothing leaves the machine. Safe to call twice.
pub fn init_crash_reporting(data_dir: PathBuf) {
    INIT_HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            previous(info);
            let thread = std::thread::current();
            let report = format_panic_report(
                &payload_to_string(info.payload()),
                info.location().map(|l| l.to_string()).as_deref(),
                thread.name().unwrap_or("<unnamed>"),
                chrono::Utc::now().timestamp(),
            );
            if let Err(e) = write_crash_report(&data_dir, &report) {
                eprintln!("Failed to write crash report: {}", e);
            }
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_log_path() {
        assert_eq!(
            crash_log_path(Path::new("/data")),
            PathBuf::from("/data/crashes/crashes.log")
        );
    }

    #[test]
    fn test_data_dir_from_database_url() {
        assert_eq!(
            data_dir_from_database_url("sqlite:/var/kestrel/data/kestrel.db"),
            PathBuf::from("/var/kestrel/data")
        );
        assert_eq!(
            data_dir_from_database_url("sqlite::memory:"),
            PathBuf::from(".")
        );
    }

    #[test]
    fn test_format_panic_report() {
        let report = format_panic_report("boom", Some("src/main.rs:10"), "main", 123);
        assert!(report.contains("boom"));
        assert!(report.contains("src/main.rs:10"));
        assert!(report.contains("main"));
    }

    #[test]
    fn test_write_crash_report_appends() {
        let dir = std::env::temp_dir().join(format!("kestrel-crash-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        write_crash_report(&dir, "first\n").unwrap();
        write_crash_report(&dir, "second\n").unwrap();
        let content = std::fs::read_to_string(crash_log_path(&dir)).unwrap();
        assert!(content.contains("first"));
        assert!(content.contains("second"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
