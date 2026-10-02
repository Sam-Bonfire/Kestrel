# Security Policy

## Reporting

Do not open public issues for vulnerabilities. Open a private security
advisory on GitHub (`Security` tab → `Advisories`) or contact the maintainer
directly. Expect triage within 7 days.

## Supported

Only the latest `dev` and the latest release are patched. Self-hosters
should track releases; `docker-compose.yml` pulls the current image.

## Daily automation (not an afterthought)

Security runs as code, every day:

| Check | Where | When |
|---|---|---|
| `cargo audit` (RustSec) | `security.yml` + `mise run audit` | every PR, every merge, weekly |
| `pnpm audit --audit-level=high` (all workspaces) | `security.yml` + `mise run audit` | every PR, every merge, weekly |
| Verified-secret scan (TruffleHog) | `security.yml` | every PR, every merge, weekly |
| Dependabot patch PRs (cargo + npm) | `dependabot.yml` | daily 06:00 UTC |
| CodeQL code scanning | repo setting (enable once, runs on schedule) | weekly |

Run locally before pushing: `mise run audit`. It needs the `cargo-audit`
binary once (`cargo install cargo-audit --locked`, or the prebuilt
release asset — CI uses the prebuilt via `rustsec/audit-check`).

## Standing rules

- No secrets in git. `.env` and `secrets/` are ignored; `.env.example`
  carries placeholders only. Verified-secret hits fail CI.
- Session signing and vault encryption use **separate** keys
  (`JWT_SECRET`, vault key) — never one value for both. Production
  refuses to boot with generated throwaway keys.
- No `AllowOrigin::mirror_request()` + credentials, no raw `{@html}`
  of untrusted markup, no `?token=` outside the SSE stream.
  These are covered by regression tests; reintroducing them fails CI review.
- Dependencies: every install recorded in its manifest + lockfile.
  `pnpm-lock.yaml` and `Cargo.lock` are committed; audit jobs read them.

## Manual steps (owner, one time)

1. Repo Settings → Security → enable **CodeQL default setup** and
   **secret scanning** (needs GHAS/admin scope the CLI lacks).
2. Add a branch ruleset on `dev` + `main` requiring the Daily Security
   checks (and existing validation) to pass before merge.
3. Triage the current Dependabot backlog oldest-first; highs first.
