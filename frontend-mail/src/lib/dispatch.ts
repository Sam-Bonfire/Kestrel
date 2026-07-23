export interface OutboundMessage {
  to: string;
  subject: string;
  body: string;
  provider: 'gmail' | 'outlook';
  accessToken: string;
}

export async function sendEmailDirect(msg: OutboundMessage): Promise<void> {
  if (msg.provider === 'gmail') {
    // Construct MIME base64 request for Gmail API
    const rawMessage = `To: ${msg.to}\r\nSubject: ${msg.subject}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${msg.body}`;
    const encodedMessage = btoa(rawMessage).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');

    const res = await fetch('https://gmail.googleapis.com/gmail/v1/users/me/messages/send', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${msg.accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ raw: encodedMessage }),
    });

    if (!res.ok) {
      throw new Error(`Gmail API direct send failed: ${res.statusText}`);
    }
  } else {
    // Construct Microsoft Graph API sendMail payload
    const res = await fetch('https://graph.microsoft.com/v1.0/me/sendMail', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${msg.accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        message: {
          subject: msg.subject,
          body: { contentType: 'HTML', content: msg.body },
          toRecipients: [{ emailAddress: { address: msg.to } }],
        },
      }),
    });

    if (!res.ok) {
      throw new Error(`Microsoft Graph API direct send failed: ${res.statusText}`);
    }
  }
}
