# Self-Hosting Kestrel

Kestrel is designed to run on one machine you own. Nothing here requires a paid
service, and every optional integration stays off until you set its environment
variable.

For container setup, networking and client installs see [DEPLOYMENT.md](./DEPLOYMENT.md).
Backups are covered there too (section 8).

## What runs where

| Piece | Where it runs | Needed for |
| --- | --- | --- |
| `kestrel-server` (Rust) | Your Docker host / NAS | API, SQLite or Postgres, sync workers |
| Landing page | Served by `kestrel-server` | Downloads + portfolio page |
| Desktop / mobile apps | User devices | Mail and calendar UI |
| WASM provider plugins | Loaded by `kestrel-server` | Gmail / Outlook sync ([plugin contract](./PLUGINS.md)) |

## Optional integrations

Each row is safe to leave unset. "Unset" never fails a build or a request.

| Variable | Unset behavior | Notes |
| --- | --- | --- |
| `FORMS_ENDPOINT` | Site forms report `disabled` and send nothing | Bring your own receiver, see below |
| `FORMS_TOKEN` | Requests go out without `Authorization` | Routing key checked by your receiver; it ships in the bundle, so it is not a secret |
| `KESTREL_CRASH_LOG` | Crash reports stay off | `1` writes them to `data/crashes/crashes.log` on the host |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` | Gmail connect is unavailable | Outlook still works |
| `MICROSOFT_CLIENT_ID` / `MICROSOFT_CLIENT_SECRET` | Outlook connect is unavailable | Gmail still works |
| `SYNC_INTERVAL_MINUTES` | `5` | Background mail/calendar sync cadence |
| `PLUGINS_DIR` | `/app/plugins` | Compiled WASM provider plugins |

## Forms ingest contract

Site forms (signup interest, contact) post JSON to `FORMS_ENDPOINT`. Bring any
receiver: a Cloudflare Worker, a small Node service, an n8n webhook.

Request:

```http
POST <FORMS_ENDPOINT>
Content-Type: application/json
Authorization: Bearer <FORMS_TOKEN>   # only when FORMS_TOKEN is set
```

```json
{
  "app": "landing",
  "form": "interest",
  "fields": {
    "_hp": "",
    "email": "someone@example.com"
  },
  "meta": {
    "turnstile": "<cloudflare-turnstile-token>",
    "userAgent": "Mozilla/5.0 ...",
    "locale": "en-GB",
    "sentAt": "2026-10-28T15:20:00.000Z"
  }
}
```

Rules for a receiver:

1. `app` is one of `landing`, `mail`, `calendar`; `form` names the specific form.
2. `fields` holds user input only. A non-empty `_hp` (honeypot) means a bot: drop
   the submission and return `2xx` so the bot learns nothing.
3. Verify `meta.turnstile` server-side when you enable Turnstile. Never trust a
   client-passed "verified" flag.
4. Treat every value as untrusted text: cap lengths, never render it as HTML,
   never put it in shell commands or SQL fragments.
5. Return `2xx` on success. The client maps any other status to a user-facing
   failure and never retries on its own.
6. `attachment` is a base64 data URL (capped at 2 MB client-side) with the
   original name in `attachmentName`. Decode it server-side and never trust the
   extension or MIME type.

The client side is `createFormsClient` in `@kestrel/shared`
(`frontend-shared/src/forms/index.ts`). It never throws: it returns
`sent`, `disabled` or `rejected` so a broken receiver cannot crash a page.

### Minimal receiver

A Cloudflare Worker that accepts the contract, drops honeypot hits, and forwards
to a destination of your choice:

```js
export default {
  async fetch(request, env) {
    if (request.method !== 'POST') return new Response('method not allowed', { status: 405 });
    if (env.FORMS_TOKEN && request.headers.get('authorization') !== `Bearer ${env.FORMS_TOKEN}`) {
      return new Response('unauthorized', { status: 401 });
    }

    let body;
    try {
      body = await request.json();
    } catch {
      return new Response('bad json', { status: 400 });
    }

    const fields = body && typeof body.fields === 'object' && body.fields !== null ? body.fields : {};
    const honeypot = typeof fields._hp === 'string' ? fields._hp : '';
    if (honeypot !== '') return new Response('ok', { status: 200 });

    const app = typeof body.app === 'string' ? body.app : '';
    const form = typeof body.form === 'string' ? body.form : '';
    if (!['landing', 'mail', 'calendar'].includes(app) || form === '') {
      return new Response('bad payload', { status: 400 });
    }

    // Your destination: a webhook, an email worker, a database binding.
    await env.DESTINATION.fetch('https://example.invalid/inbox', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ app, form, fields, meta: body.meta ?? {} }),
    });

    return new Response('ok', { status: 200 });
  },
};
```

Point `FORMS_ENDPOINT` at the Worker URL and set `FORMS_TOKEN` to the same value
in the Worker (`wrangler secret put FORMS_TOKEN`) and in the build environment.

### n8n receiver

[`deploy/n8n/forms-ingest.json`](../deploy/n8n/forms-ingest.json) is a ready
workflow: it authenticates per form, drops honeypots, rate limits, verifies
Turnstile, and creates a page in the Notion database that form routes to.

Setup:

1. Create a data table named `forms_registry` with the columns `app`, `form`,
   `databaseId`, `titleProperty`, `propertyMap`, `requiredFields`, `token`,
   `notifyUrl` (all strings).
2. Import the JSON (`Workflows → Import from File`), then open the `Registry`
   node and select your table — the export carries the author's table ID.
3. Create a Notion credential with write access to the target databases and
   select it on the `Create Notion page` node.
4. Activate the workflow and set `FORMS_ENDPOINT` to
   `https://<your-n8n>/webhook/forms-ingest`.

Registry rows:

| Column | Meaning |
| --- | --- |
| `app` / `form` | The pair the client sends. Anything else gets `400`. |
| `databaseId` | Notion database the submission becomes a page in. Required. |
| `titleProperty` | Name of the database's title property. Defaults to `Name`. |
| `propertyMap` | JSON mapping a form field to `{"name": "...", "type": "..."}`. Unmapped fields become `rich_text` with the same name. |
| `requiredFields` | JSON array of field keys the form cannot omit (e.g. `["email"]`). A submission missing one gets `400`. Blank means none. |
| `token` | Expected token. Blank accepts any token, i.e. a public form. |
| `notifyUrl` | Optional webhook (Discord, Slack) pinged after the page is created. |

```json
{"app":"landing","form":"interest","databaseId":"<notion-db-id>","titleProperty":"Name",
 "propertyMap":"{\"email\":{\"name\":\"Email\",\"type\":\"email\"},\"message\":{\"name\":\"Notes\"}}",
 "requiredFields":"[\"email\"]","token":"","notifyUrl":""}
```

Behavior worth knowing:

- Tokens are accepted as `Authorization: Bearer <token>` or `X-Token: <token>`,
  so the same registry serves the bundle client and a plain `curl`.
- Five submissions per IP, per form, per minute. A filled `_hp` gets a `200` and
  is dropped before anything is stored.
- With `TURNSTILE_SECRET` set in the n8n environment, a submission without a
  token is rejected. Without the variable, verification is skipped — set it
  whenever the site has a Turnstile site key.
- Attachments are refused with `415`: Notion cannot fetch a data URL. Use the
  Worker receiver (which can store the bytes) for the attachment variant.
- `mise run test:scripts` asserts the wiring and both Code nodes of the export
  without needing a live n8n.


## Bring-your-own backend checklist

- Serve the receiver over HTTPS. The client posts from an HTTPS page, and mixed
  content is blocked by the browser.
- Log the submission body only if you accept that user input lands in your logs.
- Deduplicate on `meta.sentAt` if you retry at the edge; the client does not.
- Keep the receiver stateless when you can. A form queue you forget to drain is
  a support ticket.
- Return `4xx` for malformed payloads and `5xx` for genuine receiver failures.
  Both surface as `rejected` to the user, but the split keeps your metrics honest.

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| Forms do nothing, no network call | `FORMS_ENDPOINT` was empty when the landing page was built |
| `401` from the receiver | `FORMS_TOKEN` differs between build env and receiver env |
| Submissions accepted but never arrive | Honeypot check dropping real users because the hidden field got a value |
| Landing page missing after deploy | `frontend-landing/dist` was not copied into the image |
