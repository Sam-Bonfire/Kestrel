// ΓöÇΓöÇ Design tokens ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
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

// ΓöÇΓöÇ API client ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
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

// ΓöÇΓöÇ Auth store & Settings store ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
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
  mailDensity,
  type MailDensity,
  theme,
  mailDefaultLandingView,
  mailDefaultSendAction,
  mailSignature,
  labelCustomizations,
  initializeSettings,
  syncInterval,
  swipeLeftAction,
  swipeRightAction,
  type SwipeActionType,
  mailSnoozeDefault,
  smartTriageEnabled,
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

export {
  relativeTimeTick,
} from './stores/index.js';

// ΓöÇΓöÇ Offline queue ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
export {
  enqueueMutation,
  dequeuePending,
  acknowledgeMutation,
  clearQueue,
  queueSize,
} from './offline/index.js';

export type { QueuedMutation } from './offline/index.js';

// ΓöÇΓöÇ Components ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
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
  ShortcutCheatSheet,
  ServerConfig,
} from './components/index.js';

// ΓöÇΓöÇ Utils ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
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
export {
  DEFAULT_SNOOZE_PRESET,
  SNOOZE_PRESETS,
  resolveSnoozeTimestamp,
  snoozePresetLabel,
  type SnoozePreset,
} from './utils/snooze.js';

export { detectTimezone } from './utils/timezoneDetector.js';
export type { DetectedTimezoneResult } from './utils/timezoneDetector.js';
export {
  formatRelativeTime,
  formatExactDateTime,
} from './utils/relativeTime.js';
export {
  checkForAppUpdate,
  installAppUpdate,
  type UpdateStatus,
} from './utils/updater.js';
export {
  buildEventDensityMap,
  daysInMonth,
  toISODateString,
  type DatedEvent,
  type DayDensity,
} from './utils/eventDensity.js';
export {
  DEFAULT_WORKING_HOURS,
  isWorkingDay,
  isWorkingHour,
  parseTimeToMinutes,
  type WorkingHoursConfig,
} from './utils/workingHours.js';
export { findDuplicateGroups, normalizeContactName } from './utils/merge.js';
export { isNewsletter } from './utils/newsletters.js';
export { setUnreadBadge } from './utils/badge.js';
export {
  senderDomain,
  isDomainAllowed,
  allowSenderDomain,
  blockRemoteImages,
} from './utils/trackingProtection.js';
export {
  parseKestrelDeepLink,
  buildThreadDeepLink,
  buildEventDeepLink,
  type KestrelDeepLink,
} from './utils/deepLinks.js';
export {
  firstTimeSenders,
  approveSender,
  loadScreened,
  type ScreenedSender,
} from './utils/screener.js';
export {
  resolveEffectiveTheme,
  applyTheme,
  initTheme,
  type ThemeMode,
} from './utils/theme.js';
export {
  stagePopoutDraft,
  takePopoutDraft,
  popoutDraftNonce,
  buildPopoutUrl,
  openComposePopout,
  type PopoutDraft,
} from './utils/popout.js';
export { categorizeEmail, type EmailCategory } from './utils/categorize.js';
export {
  parseNaturalEvent,
  shiftTime,
  type ParsedEvent,
} from './utils/nlpEvent.js';
export { plainText } from './utils/html.js';
export { mergeDuplicateEvents, type MergedEvent } from './utils/duplicates.js';
export {
  buildDailyBriefing,
  shouldShowBriefing,
  markBriefingShown,
  type DailyBriefing,
} from './utils/briefing.js';
export { focusDayColumn, focusEventInColumn } from './utils/gridNav.js';
export {
  triageCandidates,
  shouldRunTriage,
  markTriageRun,
} from './utils/triage.js';
export { findConflicts, nextFreeSlot, type Schedulable, type FreeSlot } from './utils/conflicts.js';
