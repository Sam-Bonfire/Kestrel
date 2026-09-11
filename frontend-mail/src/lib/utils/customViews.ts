export interface CustomViewFilter {
  view: string;
  category: 'All' | 'Primary' | 'Updates' | 'Social' | 'Forums';
  label: string;
  attachmentOnly: boolean;
  dateRange: 'All' | 'Today' | 'This Week' | 'This Month';
  unreadOnly: boolean;
}

export interface CustomView {
  id: string;
  name: string;
  filter: CustomViewFilter;
}

const VIEWS_KEY = 'kestrel:mail:custom_views';

function loadViews(): CustomView[] {
  try {
    if (typeof localStorage !== 'undefined') {
      const raw = localStorage.getItem(VIEWS_KEY);
      const parsed = raw !== null ? JSON.parse(raw) : [];
      if (!Array.isArray(parsed)) return [];
      return parsed.filter(
        (v): v is CustomView =>
          typeof v?.id === 'string' && typeof v?.name === 'string' && !!v?.filter
      );
    }
    return [];
  } catch {
    return [];
  }
}

function saveViews(views: CustomView[]) {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(VIEWS_KEY, JSON.stringify(views));
    }
  } catch {
    // Non-fatal
  }
}

export function getCustomViews(): CustomView[] {
  return loadViews();
}

export function saveCustomView(name: string, filter: CustomViewFilter): CustomView[] {
  const trimmed = name.trim().slice(0, 60);
  if (!trimmed) return loadViews();
  const views = loadViews();
  const view: CustomView = {
    id: `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`,
    name: trimmed,
    filter,
  };
  const next = [...views, view];
  saveViews(next);
  return next;
}

export function deleteCustomView(id: string): CustomView[] {
  const next = loadViews().filter((v) => v.id !== id);
  saveViews(next);
  return next;
}
