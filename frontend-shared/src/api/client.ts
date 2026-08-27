import { authState, logout } from '../stores/auth.svelte.js';

export const API_BASE = 'http://localhost:8080/api/v1';

/**
 * Typed API client for the Kestrel backend.
 * All functions return parsed JSON or throw ApiError.
 */

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
    public readonly body?: unknown,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

interface RequestOptions extends Omit<RequestInit, 'method' | 'body'> {
  token?: string;
  body?: unknown;
}

// ── Internal helpers ─────────────────────────────────────────────

function buildHeaders(token?: string): HeadersInit {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  const activeToken = token || authState.token; if (activeToken) { headers["Authorization"] = `Bearer ${activeToken}`; } return headers;
}

import { enqueueMutation, dequeuePending, acknowledgeMutation } from '../offline/queue.js';

async function request<T>(
  method: string,
  path: string,
  opts: RequestOptions = {},
): Promise<T> {
  const { token, body, ...init } = opts;
  
  if (!navigator.onLine && method !== 'GET') {
    enqueueMutation(path, method, body);
    return undefined as T; // Return early for void operations (or mock for others)
  }

  try {
    const res = await fetch(`${API_BASE}${path}`, {
      method,
      headers: buildHeaders(),
      credentials: 'include',
      body: body != null ? JSON.stringify(body) : undefined,
      ...init,
    });

    if (res.status === 204) {
      return undefined as T;
    }

    if (!res.ok) {
      if (res.status === 401) {
        logout();
      }
      let parsed: unknown;
      try {
        parsed = await res.json();
      } catch {
        parsed = null;
      }
      throw new ApiError(
        res.status,
        `${method} ${path} failed: ${res.statusText}`,
        parsed,
      );
    }

    return res.json() as Promise<T>;
  } catch (err: any) {
    if (err instanceof TypeError) {
      if (method !== 'GET') {
        enqueueMutation(path, method, body);
        return undefined as T;
      }
      throw new ApiError(0, 'Network error');
    }
    throw err;
  }
}

export async function replayOfflineQueue(): Promise<void> {
  if (!navigator.onLine) return;
  const pending = dequeuePending();
  for (const mut of pending) {
    try {
      await fetch(`${API_BASE}${mut.path}`, {
        method: mut.method,
        headers: buildHeaders(),
        credentials: 'include',
        body: mut.body != null ? JSON.stringify(mut.body) : undefined,
      });
      acknowledgeMutation(mut.id);
    } catch (err) {
      console.error('Failed to replay mutation', mut, err);
      // Stop replay on first network failure to maintain order
      if (err instanceof TypeError) break; 
    }
  }
}

// ── Search (FTS5) ───────────────────────────────────────────

export interface SearchResult {
  id: string;
  account_id: string;
  external_id: string;
  thread_id: string;
  subject: string | null;
  sender_name: string | null;
  sender_email: string;
  snippet: string | null;
  date_received: number;
  is_read: boolean;
}

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  query: string;
}

export async function searchMessages(query: string, limit?: number): Promise<SearchResponse> {
  const q = new URLSearchParams();
  q.append('q', query);
  if (limit !== undefined) {
    q.append('limit', limit.toString());
  }
  return request<SearchResponse>('GET', `/search?${q.toString()}`);
}

// ── Types ───────────────────────────────────────────────────────

export interface HealthResponse {
  status: 'ok';
}

export interface RegisterResponse {
  user_id: string;
}

export interface TokenResponse {
  token: string;
  user_id: string;
}

export interface ProviderBranding {
  id: string;
  name: string;
  logo_url: string;
  auth_url: string;
}

export interface PaginatedMessages {
  messages: Message[];
  total: number;
  page: number;
  per_page: number;
}

export interface Message {
  id: string;
  subject: string;
  from: string;
  from_email: string;
  snippet: string;
  date: string;
  read: boolean;
  labels: string[];
  has_attachments: boolean;
}

export interface FullMessage extends Message {
  body: string;
  body_html?: string;
  body_text?: string;
  recipients?: string[] | string;
  to: string[];
  cc: string[];
  attachments: Attachment[];
}

export interface Attachment {
  filename: string;
  mime_type: string;
  size: number;
}

export interface SearchResult {
  query: string;
  messages: Message[];
  total: number;
}

export interface Calendar {
  id: string;
  name: string;
  color: string;
  visible: boolean;
}

export interface CalendarEvent {
  id: string;
  calendar_id: string;
  title: string;
  description: string;
  start: string;
  end: string;
  all_day: boolean;
  location: string;
}

export interface EventSearchResult {
  query: string;
  events: CalendarEvent[];
  total: number;
}

// ── Auth endpoints ──────────────────────────────────────────────

export async function getHealth(): Promise<HealthResponse> {
  return request<HealthResponse>('GET', '/health');
}

export async function register(
  email: string,
  password: string,
  token?: string,
): Promise<RegisterResponse> {
  return request<RegisterResponse>('POST', '/auth/register', {
    token,
    body: { email, password },
  });
}

export async function createToken(
  email: string,
  password: string,
): Promise<TokenResponse> {
  return request<TokenResponse>('POST', '/auth/token', {
    body: { email, password },
  });
}

export async function getMe(): Promise<{ user_id: string }> {
  return request<{ user_id: string }>('GET', '/auth/me');
}

export async function loginWithProvider(provider: string) {
  window.location.href = `${API_BASE}/auth/login?provider=${encodeURIComponent(provider)}`;
}

export function getCallbackUrl(): string {
  return `${API_BASE}/auth/callback`;
}

// ── Account endpoints ───────────────────────────────────────────

export async function deleteAccount(
  accountId: string,
  token?: string,
): Promise<void> {
  return request<void>('DELETE', `/accounts/${accountId}`, { token });
}

// ── Provider endpoints ──────────────────────────────────────────

export async function getProviders(
  token?: string,
): Promise<ProviderBranding[]> {
  return request<ProviderBranding[]>('GET', '/providers', { token });
}

// ── Message endpoints ───────────────────────────────────────────

export async function getMessages(
  token?: string,
  page = 1,
  perPage = 50,
): Promise<PaginatedMessages> {
  const params = new URLSearchParams({
    page: String(page),
    per_page: String(perPage),
  });
  return request<PaginatedMessages>('GET', `/messages?${params}`, {
    token,
  });
}

export async function getMessage(
  messageId: string,
  token?: string,
): Promise<FullMessage> {
  return request<FullMessage>('GET', `/messages/${messageId}`, { token });
}

export async function markAsRead(
  messageId: string,
  token?: string,
): Promise<void> {
  return request<void>('POST', `/messages/${messageId}/read`, { token });
}

export async function archiveMessage(
  messageId: string,
  token?: string,
): Promise<void> {
  return request<void>('POST', `/messages/${messageId}/archive`, {
    token,
  });
}

export async function snoozeMessage(
  messageId: string,
  token?: string,
): Promise<void> {
  return request<void>('POST', `/messages/${messageId}/snooze`, {
    token,
  });
}

export async function muteMessage(
  messageId: string,
  token?: string,
): Promise<void> {
  return request<void>('POST', `/messages/${messageId}/mute`, { token });
}

export async function reportPhishing(
  messageId: string,
  token?: string,
): Promise<void> {
  return request<void>('POST', `/messages/${messageId}/report-phishing`, {
    token,
  });
}

export async function blockSender(
  email: string,
  token?: string,
): Promise<void> {
  return request<void>('POST', '/senders/block', {
    token,
    body: { email },
  });
}

export function getEmlDownloadUrl(messageId: string): string {
  return `${API_BASE}/messages/${messageId}/raw`;
}

export interface SendMessageRequest {
  to: string;
  subject: string;
  body: string;
  thread_id?: string;
}

export interface SendMessageResponse {
  id: string;
}

export async function sendMessage(
  payload: SendMessageRequest,
  token?: string,
): Promise<SendMessageResponse> {
  return request<SendMessageResponse>('POST', '/messages/send', {
    token,
    body: payload,
  });
}

export async function trashMessage(
  messageId: string,
  token?: string,
): Promise<void> {
  return request<void>('POST', `/messages/${messageId}/trash`, {
    token,
  });
}


export async function toggleStar(id: string, is_starred: boolean, token?: string): Promise<void> {
  await request('POST', `/messages/${id}/star`, {
    token,
    body: { is_starred },
  });
}

export async function updateLabels(id: string, labels: string[], token?: string): Promise<void> {
  await request('POST', `/messages/${id}/labels`, {
    token,
    body: { labels },
  });
}

export type BulkActionType = 'mark_read' | 'archive' | 'trash' | 'toggle_star';

export async function bulkAction(message_ids: string[], action: BulkActionType, action_value?: boolean, token?: string): Promise<void> {
  await request('POST', '/messages/bulk', {
    token,
    body: {
      message_ids,
      action,
      action_value,
    },
  });
}

// ── Search & Attachments ────────────────────────────────────────────

export async function searchMessagesByToken(
  query: string,
  token?: string,
): Promise<SearchResult> {
  const params = new URLSearchParams({ q: query });
  return request<SearchResult>('GET', `/search?${params}`, { token });
}

export async function searchEvents(
  query: string,
  token?: string,
): Promise<EventSearchResult> {
  const params = new URLSearchParams({ q: query });
  return request<EventSearchResult>('GET', `/search/events?${params}`, {
    token,
  });
}

// ── Attachment redirect ─────────────────────────────────────────

export function getAttachmentRedirectUrl(
  messageId: string,
  filename: string,
): string {
  return `${API_BASE}/messages/${messageId}/attachments/${encodeURIComponent(filename)}/redirect`;
}

/**
 * Download an attachment's raw bytes from the backend.
 * Uses cookie auth (credentials: 'include'), so it works inside Tauri
 * where the webview shares the backend session cookie.
 */
export async function downloadAttachment(
  messageId: string,
  filename: string,
): Promise<ArrayBuffer> {
  const res = await fetch(
    `${API_BASE}/messages/${messageId}/attachments/${encodeURIComponent(filename)}`,
    {
      method: 'GET',
      credentials: 'include',
    },
  );
  if (!res.ok) {
    throw new ApiError(res.status, `Downloading attachment failed: ${res.statusText}`);
  }
  return res.arrayBuffer();
}

// ── Sync endpoints ──────────────────────────────────────────────

export function createSyncStream(token?: string): EventSource {
  const url = new URL(`${API_BASE}/sync/stream`, window.location.origin);
  if (token) {
    url.searchParams.set('token', token);
  }
  return new EventSource(url.toString(), { withCredentials: true });
}

export async function triggerSync(token?: string): Promise<void> {
  return request<void>('POST', '/sync/trigger', { token });
}

// ── Calendar endpoints ──────────────────────────────────────────

export async function getCalendars(
  token?: string,
): Promise<Calendar[]> {
  return request<Calendar[]>('GET', '/calendars', { token });
}

export async function getEvents(
  token?: string,
  start?: string,
  end?: string,
): Promise<CalendarEvent[]> {
  const params = new URLSearchParams();
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  const qs = params.toString();
  const path = qs ? `/events?${qs}` : '/events';
  return request<CalendarEvent[]>('GET', path, { token });
}

export async function createEvent(
  event: Omit<CalendarEvent, 'id'>,
  token?: string,
): Promise<CalendarEvent> {
  return request<CalendarEvent>('POST', '/events', { token, body: event });
}

export async function updateEvent(
  eventId: string,
  patch: Partial<CalendarEvent>,
  token?: string,
): Promise<CalendarEvent> {
  return request<CalendarEvent>('PATCH', `/events/${eventId}`, {
    token,
    body: patch,
  });
}

export async function deleteEvent(
  eventId: string,
  token?: string,
): Promise<void> {
  return request<void>('DELETE', `/events/${eventId}`, { token });
}

export const apiClient = {
  get: (path: string) => request<any>('GET', path.replace('/api/v1', '')),
  post: (path: string, body?: any) => request<any>('POST', path.replace('/api/v1', ''), { body }),
  delete: (path: string) => request<any>('DELETE', path.replace('/api/v1', ''))
};

// ── Settings ──────────────────────────────────────────────

export async function getSettings(): Promise<SettingsPayload> {
  return request<SettingsPayload>('GET', '/settings', {
    // Override API_BASE to use /api/settings instead of /api/v1/settings
  }).catch(() => {
    // If it fails, fallback to standard fetch if the request helper prepends /api/v1
    return fetch(`${API_BASE.replace('/api/v1', '')}/api/settings`, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
    }).then(r => r.json());
  });
}

export async function updateSettings(settings: SettingsPayload): Promise<SettingsPayload> {
  return fetch(`${API_BASE.replace('/api/v1', '')}/api/settings`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify(settings),
  }).then(r => r.json());
}

export interface LabelCustomization {
  iconName: string;
  colorName: string;
}

export interface Snippet {
  id: string;
  title: string;
  shortcut: string;
  template: string;
}

export interface Signature {
  id: string;
  accountId: string | null;
  name: string;
  htmlContent: string;
  isDefault: boolean;
}

export interface SettingsPayload {
  mailDenseMode?: boolean;
  mailDefaultLandingView?: string;
  mailSignature?: string;
  labelCustomizations?: Record<string, LabelCustomization>;
  syncInterval?: number;
  theme?: string;
  snippets?: Snippet[];
  signatures?: Signature[];
}
