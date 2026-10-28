// Forms ingest contract. Site forms POST to an operator-owned endpoint
// (FORMS_ENDPOINT); when it is unset the client reports "disabled" and never
// touches the network, so a self-hosted deployment needs no extra service.

export interface FormsConfig {
  endpoint: string;
  token?: string;
}

export interface FormMeta {
  /** Cloudflare Turnstile token; the receiver verifies it server-side. */
  turnstile?: string;
  userAgent?: string;
  locale?: string;
  /** ISO timestamp added by the client. */
  sentAt?: string;
}

export interface FormSubmission {
  app: 'landing' | 'mail' | 'calendar';
  form: string;
  /** User-entered values. `_hp` is the honeypot and must be sent empty. */
  fields: Record<string, string>;
  meta?: FormMeta;
}

export type FormSubmitResult =
  | { status: 'sent' }
  | { status: 'disabled' }
  | { status: 'rejected'; error: string };

export interface FormsClient {
  readonly enabled: boolean;
  submit(submission: FormSubmission): Promise<FormSubmitResult>;
}

export function createFormsClient(config: FormsConfig): FormsClient {
  const endpoint = config.endpoint.trim();
  const token = config.token?.trim();
  const enabled = endpoint !== '';

  return {
    enabled,

    async submit(submission) {
      if (!enabled) return { status: 'disabled' };

      const fields = { _hp: '', ...submission.fields };
      const body: FormSubmission = {
        ...submission,
        fields,
        meta: { ...submission.meta, sentAt: new Date().toISOString() },
      };

      try {
        const response = await fetch(endpoint, {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
            ...(token ? { authorization: `Bearer ${token}` } : {}),
          },
          body: JSON.stringify(body),
        });

        if (!response.ok) {
          return { status: 'rejected', error: `Forms endpoint returned ${response.status}` };
        }
        return { status: 'sent' };
      } catch (error) {
        return {
          status: 'rejected',
          error: error instanceof Error ? error.message : 'Network error',
        };
      }
    },
  };
}
