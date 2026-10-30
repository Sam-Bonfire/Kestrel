# Provider plugins

Provider plugins are WASM components built from [`wit/kestrel.wit`](../wit/kestrel.wit)
and loaded by `kestrel-server` from `PLUGINS_DIR` (default `backend/plugins`).
Gmail and Outlook are the reference implementations under [`plugins/`](../plugins).

## Build

```sh
mise run build:plugins
```

This builds every plugin with `cargo component` and copies the `.wasm`
artifacts into `backend/plugins/`. Plugins are rebuilt from source on every
release; they are not backed up (see [DEPLOYMENT.md](DEPLOYMENT.md)).

## Outbound network access

A plugin can only reach hosts it declares. Return them from
`get-branding` in the `hosts` field:

```rust
hosts: vec!["gmail.googleapis.com".to_string(), "www.googleapis.com".to_string()],
```

Rules enforced by the host (`backend/src/plugins/wasm_runtime.rs`):

- **Exact hosts only.** No wildcards, no subdomain matching, no ports. A
  request to a host that is not listed fails with `Host '<host>' is not in the allowlist`.
- **HTTPS only.** `http://` URLs are rejected before any connection.
- **Empty `hosts` denies everything.** A plugin that returns an empty list
  cannot make any request.
- **Redirects are re-checked.** Each hop must also be on the list, up to five hops.
- **Blocked addresses.** The request host is resolved first; loopback, private
  (`10/8`, `172.16/12`, `192.168/16`), link-local and cloud metadata
  (`169.254.169.254`), reserved, unspecified and IPv4-mapped-IPv6 equivalents
  are rejected even when the host name is allowlisted.
- **Size caps.** Request bodies are limited to 1 MB and responses to 10 MB.

Attachment URLs returned by `get-attachment-url` go through the same host
allowlist and address checks, because the backend fetches them.

## Sandbox limits per invocation

Each plugin invocation gets a fresh store with:

| Limit | Value |
| --- | --- |
| Fuel | 10,000,000 instructions |
| Wall-clock deadline | 5 s (epoch interruption, 1 s ticker) |
| Linear memory | 50 MB |

Exceeding any of these traps the invocation and surfaces as a typed timeout
error instead of hanging the server.
