// Form field assembly for the site forms contract (docs/SELFHOST.md).
// Kept out of the component so the rules are testable without a browser.

export const MAX_ATTACHMENT_BYTES = 2 * 1024 * 1024;

export interface SignupInput {
  email: string;
  name?: string;
  message?: string;
  attachment?: File | null;
}

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error(`Could not read ${file.name}`));
    reader.readAsDataURL(file);
  });
}

export async function buildSignupFields(input: SignupInput): Promise<Record<string, string>> {
  const fields: Record<string, string> = { email: input.email.trim() };
  const name = input.name?.trim();
  const message = input.message?.trim();
  if (name) fields.name = name;
  if (message) fields.message = message;

  const file = input.attachment;
  if (file) {
    if (file.size > MAX_ATTACHMENT_BYTES) {
      throw new Error('Attachment is larger than 2 MB');
    }
    fields.attachment = await readAsDataUrl(file);
    fields.attachmentName = file.name;
  }

  return fields;
}
