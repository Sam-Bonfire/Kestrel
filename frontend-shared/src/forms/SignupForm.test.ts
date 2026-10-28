import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';
import SignupForm from '../components/SignupForm.svelte';

let target: HTMLDivElement;
let instance: Record<string, unknown> | undefined;

function render(props: Record<string, unknown>): HTMLDivElement {
  target = document.createElement('div');
  document.body.appendChild(target);
  instance = mount(SignupForm, { target, props }) as Record<string, unknown>;
  flushSync();
  return target;
}

afterEach(() => {
  if (instance) unmount(instance);
  instance = undefined;
  target?.remove();
  vi.unstubAllGlobals();
});

const base = { app: 'landing' as const, form: 'interest', endpoint: '' };

describe('SignupForm', () => {
  it('renders nothing when no endpoint is configured', () => {
    const el = render(base);
    expect(el.querySelector('form')).toBeNull();
    expect(el.textContent?.trim()).toBe('');
  });

  it('renders the email variant with a honeypot, and the application variant with a file input', () => {
    const emailOnly = render({ ...base, endpoint: 'https://forms.example.com/ingest' });
    expect(emailOnly.querySelector('input[type="email"]')).not.toBeNull();
    expect(emailOnly.querySelector('input[name="_hp"]')).not.toBeNull();
    expect(emailOnly.querySelector('textarea')).toBeNull();
    unmount(instance!);
    instance = undefined;

    const application = render({ ...base, endpoint: 'https://forms.example.com/ingest', variant: 'application' });
    expect(application.querySelector('textarea')).not.toBeNull();
    expect(application.querySelector('input[type="file"]')).not.toBeNull();
    expect(application.querySelector('input[type="email"]')).not.toBeNull();
  });

  it('posts the contract payload with an empty honeypot and reports success', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, status: 202 });
    vi.stubGlobal('fetch', fetchMock);

    const el = render({ ...base, endpoint: 'https://forms.example.com/ingest' });
    const email = el.querySelector<HTMLInputElement>('input[type="email"]')!;
    email.value = 'someone@example.com';
    email.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();

    el.querySelector('form')!.requestSubmit();
    await vi.waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));

    const init = fetchMock.mock.calls[0]![1] as RequestInit;
    const body = JSON.parse(String(init.body)) as {
      app: string;
      form: string;
      fields: Record<string, string>;
    };
    expect(body.app).toBe('landing');
    expect(body.form).toBe('interest');
    expect(body.fields).toEqual({ email: 'someone@example.com', _hp: '' });

    await vi.waitFor(() => expect(el.textContent).toContain('on its way'));
  });
});
