import { describe, it, expect } from 'vitest';
import { parseKestrelDeepLink, buildThreadDeepLink, buildEventDeepLink } from './deepLinks.js';

describe('Deep links', () => {
  it('parses routed thread and event links', () => {
    expect(parseKestrelDeepLink('kestrel://mail/thread/t-1')).toEqual({ app: 'mail', kind: 'thread', id: 't-1' });
    expect(parseKestrelDeepLink('kestrel-calendar://event/e-3')).toEqual({ app: 'calendar', kind: 'event', id: 'e-3' });
  });

  it('rejects oauth callbacks, wrong kinds and garbage', () => {
    expect(parseKestrelDeepLink('kestrel://oauth/callback?code=x')).toBeNull();
    expect(parseKestrelDeepLink('kestrel://mail/event/e-1')).toBeNull();
    expect(parseKestrelDeepLink('kestrel://calendar/event/e-9')).toBeNull();
    expect(parseKestrelDeepLink('kestrel-mail://thread/t-2')).toBeNull();
    expect(parseKestrelDeepLink('kestrel://mail/thread/')).toBeNull();
    expect(parseKestrelDeepLink('https://example.com/mail/thread/1')).toBeNull();
    expect(parseKestrelDeepLink('not a url')).toBeNull();
  });

  it('tolerates trailing slashes and decodes ids', () => {
    expect(parseKestrelDeepLink('kestrel://mail/thread/t-1/')).toEqual({ app: 'mail', kind: 'thread', id: 't-1' });
    expect(parseKestrelDeepLink(buildThreadDeepLink('id with space'))).toEqual({ app: 'mail', kind: 'thread', id: 'id with space' });
  });

  it('round-trips builders through the parser', () => {
    expect(parseKestrelDeepLink(buildThreadDeepLink('t-7'))).toEqual({ app: 'mail', kind: 'thread', id: 't-7' });
    expect(parseKestrelDeepLink(buildEventDeepLink('e-8'))).toEqual({ app: 'calendar', kind: 'event', id: 'e-8' });
  });
});
