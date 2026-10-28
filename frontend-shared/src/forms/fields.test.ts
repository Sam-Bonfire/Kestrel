import { describe, expect, it } from 'vitest';
import { buildSignupFields, MAX_ATTACHMENT_BYTES } from './fields.js';

describe('buildSignupFields', () => {
  it('trims the email and drops empty optional values', async () => {
    const fields = await buildSignupFields({
      email: '  someone@example.com ',
      name: '   ',
      message: ' hello ',
    });

    expect(fields).toEqual({ email: 'someone@example.com', message: 'hello' });
  });

  it('encodes a small attachment as a data URL with its file name', async () => {
    const file = new File(['hello'], 'brief.pdf', { type: 'application/pdf' });
    const fields = await buildSignupFields({ email: 'a@b.c', attachment: file });

    expect(fields.attachmentName).toBe('brief.pdf');
    expect(fields.attachment?.startsWith('data:application/pdf;base64,')).toBe(true);
  });

  it('rejects an attachment over the size cap before reading it', async () => {
    const file = new File([new Uint8Array(MAX_ATTACHMENT_BYTES + 1)], 'big.pdf');

    await expect(buildSignupFields({ email: 'a@b.c', attachment: file })).rejects.toThrow(
      'larger than 2 MB',
    );
  });
});
