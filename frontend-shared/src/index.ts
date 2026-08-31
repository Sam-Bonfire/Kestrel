// ── Design tokens ───────────────────────────────────────────────
export {
  colors,
  tagColors,
  typography,
  fonts,
  spacing,
  radius,
  shadows,
  borders,
  buttonStyles,
  animations,
  keyframesCSS,
  tokens,
} from './tokens/index.js';

export type { DesignTokens } from './tokens/index.js';

export { default as SettingsModal } from './components/SettingsModal.svelte';

// ── API client ──────────────────────────────────────────────────
export {
  ApiError,
  getHealth,
  register,
  createToken,
  loginWithProvider,
  getCallbackUrl,
  getMe,
  deleteAccount,
  getProviders,
  getMessages,
  getMessage,
  markAsRead,
  archiveMessage,
  trashMessage,
  searchMessages,
  searchEvents,
  getAttachmentRedirectUrl,
  createSyncStream,
  triggerSync,
  getCalendars,
  getEvents,
  createEvent,
  updateEvent,
  deleteEvent,
} from './api/index.js';

export type {
  HealthResponse,
  RegisterResponse,
  TokenResponse,
  ProviderBranding,
  PaginatedMessages,
  Message,
  FullMessage,
  Attachment,
  SearchResult,
  Calendar,
  CalendarEvent,
  EventSearchResult,
  Snippet,
  Signature,
} from './api/index.js';

// ── Auth store & Settings store ─────────────────────────────────
export {
  initAuth,
} from './stores/auth.svelte.js';

export {
  authState,
  login,
  logout,
  revokedAccounts,
  addRevokedAccount,
  mailDenseMode,
  mailDefaultLandingView,
  mailSignature,
  labelCustomizations,
  initializeSettings,
  syncInterval,
} from './stores/index.js';

export {
  initSyncEvents,
  closeSyncEvents,
  getSyncState,
  getGlobalSyncState,
} from './stores/sync.svelte.js';

export {
  templateStore,
  TemplateStore,
} from './stores/templates.svelte.js';

// ── Offline queue ───────────────────────────────────────────────
export {
  enqueueMutation,
  dequeuePending,
  acknowledgeMutation,
  clearQueue,
  queueSize,
} from './offline/index.js';

export type { QueuedMutation } from './offline/index.js';

// ── Components ──────────────────────────────────────────────────
export {
  Button,
  Spinner,
  Avatar,
  ErrorBanner,
  LabelPill,
  ProviderBadge,
  Login,
  WindowControls,
  EmailPillInput,
  Dropdown,
  ReauthBanner,
  UndoToast,
  ServerConfig,
} from './components/index.js';

// ── Utils ───────────────────────────────────────────────────────
export {
  getLabelStyle,
  buildLabelTree,
  getFlattenedLabels,
} from './utils/labels.js';

export type {
  LabelMeta,
  LabelNode,
  FlattenedLabelItem,
} from './utils/labels.js';

export {
  detectConferenceLink,
} from './utils/conference.js';

export type {
  ConferenceLink,
} from './utils/conference.js';

export { default as ContactAutocomplete } from './components/ContactAutocomplete.svelte';

export { parseIcs } from './utils/icsParser.js';
export type { IcsEvent } from './utils/icsParser.js';
export { parseChecklists } from './utils/markdownParser.js';
