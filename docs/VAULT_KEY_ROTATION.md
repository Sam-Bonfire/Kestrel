# Vault Key Rotation Runbook

## Overview

Kestrel enforces strict key separation:
- `JWT_SECRET` (or `SESSION_SECRET`): signs user session tokens.
- `VAULT_SECRET` (or `KESTREL_VAULT_KEY`): encrypts stored third-party OAuth and provider access/refresh tokens in the accounts table.

Both keys are derived independently and cannot fall back to one another. In production (`ENVIRONMENT=production` or `KESTREL_ENV=production`), missing either variable fails closed at startup.

Ciphertext format:
- `v2:<salt_b64>:<nonce_b64>:<ciphertext_b64>`: uses Argon2id memory-hard KDF + AES-256-GCM.
- `v1:<nonce_b64>:<ciphertext_b64>`: legacy SHA256 derivation + AES-256-GCM (dual-read supported).

## Zero-Downtime Key Rotation Procedure

To rotate the `VAULT_SECRET` without bricking existing connected accounts or forcing mass re-authentication, use the dual-read fallback window:

### Step 1: Set Dual-Read Configuration
Deploy the new vault secret while keeping the old secret in `VAULT_OLD_SECRET`:

```env
VAULT_SECRET=<new-random-32-byte-secret>
VAULT_OLD_SECRET=<current-vault-secret>
```

### Step 2: Restart or Deploy Backend
1. The backend boots with `VAULT_SECRET` as the primary encryption key and `VAULT_OLD_SECRET` as the read fallback.
2. Existing accounts encrypted under `VAULT_OLD_SECRET` continue to decrypt without issue.
3. Any new or refreshed account credentials will automatically be encrypted using `v2` with `VAULT_SECRET`.

### Step 3: Verify and Re-encrypt Existing Accounts
Run an account sync or token refresh across connected accounts to migrate them to the new key.
You can monitor that decryption failures (which surface as `reauth_required`) are zero.

### Step 4: Decommission the Old Key
Once all active accounts have refreshed tokens, or after the desired migration grace period:
Remove `VAULT_OLD_SECRET` from the environment:

```env
VAULT_SECRET=<new-random-32-byte-secret>
# VAULT_OLD_SECRET unset
```

Deploy the backend. Rotation is complete.
