export interface ConferenceLink {
  url: string;
  provider: string;
  displayLabel: string;
}

export function detectConferenceLink(text: string): ConferenceLink | null {
  if (!text) return null;

  // Google Meet
  const meetMatch = text.match(/https?:\/\/meet\.google\.com\/[a-zA-Z0-9-]+/);
  if (meetMatch) return { url: meetMatch[0], provider: 'Google Meet', displayLabel: 'Google Meet' };

  // Zoom
  const zoomMatch = text.match(/https?:\/\/[a-zA-Z0-9.-]+\.zoom\.us\/j\/\d+(?:\?pwd=[a-zA-Z0-9]+)?/);
  if (zoomMatch) return { url: zoomMatch[0], provider: 'Zoom', displayLabel: 'Zoom Meeting' };

  // Microsoft Teams
  const teamsMatch = text.match(/https?:\/\/teams\.microsoft\.com\/l\/meetup-join\/[^"'\s<>]+/);
  if (teamsMatch) return { url: teamsMatch[0], provider: 'Microsoft Teams', displayLabel: 'Teams Meeting' };

  // Webex
  const webexMatch = text.match(/https?:\/\/[a-zA-Z0-9.-]+\.webex\.com\/(?:meet|join)\/[a-zA-Z0-9.-]+/);
  if (webexMatch) return { url: webexMatch[0], provider: 'Webex', displayLabel: 'Webex Meeting' };

  // Jitsi
  const jitsiMatch = text.match(/https?:\/\/meet\.jit\.si\/[a-zA-Z0-9-]+/);
  if (jitsiMatch) return { url: jitsiMatch[0], provider: 'Jitsi', displayLabel: 'Jitsi Meet' };

  // Generic fallback for any other video url
  // We'll look for common keywords in URLs, or just any URL if it's the only thing in a "location" field,
  // but to be safe we'll look for video/meeting related words in the url.
  const genericMatch = text.match(/https?:\/\/[^\s<>'"]+(?:meeting|video|conference|call)[^\s<>'"]*/i);
  if (genericMatch) {
    const url = new URL(genericMatch[0]);
    return { url: genericMatch[0], provider: 'Video Conference', displayLabel: url.hostname };
  }

  // A plain URL match if we want to be more aggressive, but the requirements say "generic video URLs".
  // The above regex captures URLs with meeting/video/conference/call.

  return null;
}
