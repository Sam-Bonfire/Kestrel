//! Pure slot-grid engine for booking pages. No I/O, no framework types.

pub const SLOT_STEP_SECS: i64 = 900;
pub const MAX_SLOTS: usize = 200;

/// Free start/end pairs inside `[window_start, window_end)` that fit
/// `duration_secs` plus `buffer_secs` on both sides without overlapping
/// any busy block. Candidates sit on a 15-minute grid anchored at
/// `window_start`; callers align `window_start` to the epoch grid so the
/// advertised slots and bookable slots coincide.
pub fn compute_free_slots(
    window_start: i64,
    window_end: i64,
    busy: &[(i64, i64)],
    duration_secs: i64,
    buffer_secs: i64,
) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    if duration_secs <= 0 || window_end <= window_start {
        return out;
    }
    let mut t = window_start;
    while t + duration_secs <= window_end && out.len() < MAX_SLOTS {
        let pad_start = t - buffer_secs;
        let pad_end = t + duration_secs + buffer_secs;
        let clash = busy.iter().any(|(b0, b1)| *b0 < pad_end && pad_start < *b1);
        if !clash {
            out.push((t, t + duration_secs));
        }
        t += SLOT_STEP_SECS;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_calendar_yields_full_grid() {
        let slots = compute_free_slots(0, 3600, &[], 1800, 0);
        assert_eq!(slots, vec![(0, 1800), (900, 2700), (1800, 3600)]);
    }

    #[test]
    fn busy_block_removes_overlaps() {
        let slots = compute_free_slots(0, 3600, &[(1000, 2000)], 900, 0);
        assert!(!slots.iter().any(|(s, e)| *s < 2000 && 1000 < *e));
        assert!(slots.contains(&(2700, 3600)));
    }

    #[test]
    fn buffer_extends_blocked_range() {
        let no_buffer = compute_free_slots(0, 3600, &[(1800, 2100)], 900, 0);
        let buffered = compute_free_slots(0, 3600, &[(1800, 2100)], 900, 900);
        assert!(buffered.len() < no_buffer.len());
        assert!(
            buffered
                .iter()
                .all(|(s, e)| *e + 900 <= 1800 || *s - 900 >= 2100)
        );
    }

    #[test]
    fn invalid_inputs_yield_nothing() {
        assert!(compute_free_slots(100, 100, &[], 900, 0).is_empty());
        assert!(compute_free_slots(0, 3600, &[], 0, 0).is_empty());
        assert!(compute_free_slots(0, 3600, &[(0, 3600)], 900, 0).is_empty());
    }

    #[test]
    fn slot_cap_holds_on_wide_windows() {
        let slots = compute_free_slots(0, 31 * 86400, &[], 900, 0);
        assert_eq!(slots.len(), MAX_SLOTS);
    }
}
