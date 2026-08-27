import { describe, it, expect } from 'vitest';
import { detectConferenceLink } from './conference.js';

describe('detectConferenceLink', () => {
  it('detects Google Meet links', () => {
    const text = 'Join my meeting at https://meet.google.com/abc-defg-hij';
    const result = detectConferenceLink(text);
    expect(result).toEqual({
      url: 'https://meet.google.com/abc-defg-hij',
      provider: 'Google Meet',
      displayLabel: 'Google Meet',
    });
  });

  it('detects Zoom links', () => {
    const text = 'Zoom meeting here: https://us02web.zoom.us/j/1234567890?pwd=abc';
    const result = detectConferenceLink(text);
    expect(result).toEqual({
      url: 'https://us02web.zoom.us/j/1234567890?pwd=abc',
      provider: 'Zoom',
      displayLabel: 'Zoom Meeting',
    });
  });

  it('detects Microsoft Teams links', () => {
    const text = 'Teams: https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc123%40thread.v2/0?context=%7b%22Tid%22%3a%22abc%22%2c%22Oid%22%3a%22def%22%7d';
    const result = detectConferenceLink(text);
    expect(result).toEqual({
      url: 'https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc123%40thread.v2/0?context=%7b%22Tid%22%3a%22abc%22%2c%22Oid%22%3a%22def%22%7d',
      provider: 'Microsoft Teams',
      displayLabel: 'Teams Meeting',
    });
  });

  it('detects Webex links', () => {
    const text = 'Webex: https://company.webex.com/meet/username';
    const result = detectConferenceLink(text);
    expect(result).toEqual({
      url: 'https://company.webex.com/meet/username',
      provider: 'Webex',
      displayLabel: 'Webex Meeting',
    });
  });

  it('detects Jitsi links', () => {
    const text = 'Jitsi: https://meet.jit.si/MyCoolMeeting';
    const result = detectConferenceLink(text);
    expect(result).toEqual({
      url: 'https://meet.jit.si/MyCoolMeeting',
      provider: 'Jitsi',
      displayLabel: 'Jitsi Meet',
    });
  });

  it('detects generic video meeting links', () => {
    const text = 'Generic: https://example.com/video-call/123';
    const result = detectConferenceLink(text);
    expect(result).toEqual({
      url: 'https://example.com/video-call/123',
      provider: 'Video Conference',
      displayLabel: 'example.com',
    });
  });

  it('detects links inside markdown or html', () => {
    const text = 'Location: <a href="https://meet.google.com/xyz-uvw-qrs">Meeting</a>';
    const result = detectConferenceLink(text);
    expect(result).toEqual({
      url: 'https://meet.google.com/xyz-uvw-qrs',
      provider: 'Google Meet',
      displayLabel: 'Google Meet',
    });
  });

  it('returns null if no link is found', () => {
    const text = 'Just a regular text string with no links.';
    expect(detectConferenceLink(text)).toBeNull();
  });
});
