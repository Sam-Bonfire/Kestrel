import React, { useState, useEffect, useMemo } from 'react';
import { 
  Calendar as CalendarIcon, 
  ChevronLeft, 
  ChevronRight, 
  Plus, 
  Clock, 
  MapPin, 
  AlignLeft, 
  Trash2, 
  Check, 
  X, 
  Search, 
  Eye, 
  EyeOff, 
  ChevronDown, 
  Edit2,
  CalendarDays,
  CheckSquare,
  Sparkles,
  ExternalLink,
  Flag,
  ToggleLeft,
  ToggleRight,
  Info,
  Bell,
  Globe,
  CornerUpLeft,
  Columns,
  Video,
  Link2,
  Maximize2,
  Settings,
  Menu
} from 'lucide-react';

export interface CalendarEvent {
  id: string;
  title: string;
  date: string; // YYYY-MM-DD
  startTime: string; // HH:MM
  endTime: string; // HH:MM
  description?: string;
  color: string; // 'blue' | 'purple' | 'green' | 'orange' | 'rose' | 'amber' | 'teal'
  location?: string;
  category?: string; // 'Work' | 'Personal' | 'Workspace' | 'General'
  calendarId: string;
  status?: 'Next Up' | 'Scheduled' | 'Completed' | 'In Progress';
  priority?: 'High' | 'Medium' | 'Low' | 'None';
  isAllDay?: boolean;
  organizer?: string;
  rsvpStatus?: 'yes' | 'no' | 'maybe' | 'none';
  attendees?: { name: string; email: string; rsvp: 'yes' | 'no' | 'maybe' | 'none' }[];
}

interface Calendar {
  id: string;
  name: string;
  color: string;
  isActive: boolean;
  isDefault?: boolean;
}

interface Account {
  id: string;
  email: string;
  isExpanded: boolean;
  calendars: Calendar[];
}

interface CalendarViewProps {
  onBackToMail?: () => void;
  showToast: (msg: string, type: 'success' | 'info' | 'error') => void;
}

const COLOR_OPTIONS = [
  { name: 'blue', bg: 'bg-blue-500/20 border border-blue-400 text-blue-200 border-l-[3px] border-l-blue-500', dot: 'bg-blue-500', hex: '#2383e2' },
  { name: 'purple', bg: 'bg-purple-500/20 border border-purple-400 text-purple-200 border-l-[3px] border-l-purple-500', dot: 'bg-purple-500', hex: '#8a4bf5' },
  { name: 'green', bg: 'bg-emerald-500/20 border border-emerald-400 text-emerald-200 border-l-[3px] border-l-emerald-500', dot: 'bg-emerald-500', hex: '#0fa35c' },
  { name: 'orange', bg: 'bg-orange-500/20 border border-orange-400 text-orange-200 border-l-[3px] border-l-orange-500', dot: 'bg-orange-500', hex: '#df6a14' },
  { name: 'rose', bg: 'bg-rose-500/20 border border-rose-400 text-rose-200 border-l-[3px] border-l-rose-500', dot: 'bg-rose-500', hex: '#e03e3e' },
  { name: 'amber', bg: 'bg-amber-500/20 border border-amber-400 text-amber-100 border-l-[3px] border-l-amber-500', dot: 'bg-amber-500', hex: '#dfab00' },
  { name: 'teal', bg: 'bg-teal-500/20 border border-teal-400 text-teal-200 border-l-[3px] border-l-teal-500', dot: 'bg-teal-500', hex: '#0fa3b1' },
];

const STATUS_OPTIONS = ['Next Up', 'Scheduled', 'In Progress', 'Completed'] as const;
const PRIORITY_OPTIONS = ['High', 'Medium', 'Low', 'None'] as const;

export default function CalendarView({ onBackToMail, showToast }: CalendarViewProps) {
  // Main Date displayed in calendar
  const [currentDate, setCurrentDate] = useState(() => new Date());
  // Date displayed in the mini calendar (month view)
  const [miniMonth, setMiniMonth] = useState(() => new Date());
  const [viewMode, setViewMode] = useState<'month' | 'week' | 'day' | '1-day' | '2-day' | '3-day' | '4-day' | '5-day' | '6-day' | '7-day' | 'weekdays' | 'agenda'>('month');
  const [searchQuery, setSearchQuery] = useState('');
  const [isViewDropdownOpen, setIsViewDropdownOpen] = useState(false);
  const [dropdownSubmenu, setDropdownSubmenu] = useState<'none' | 'days' | 'settings'>('none');
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isCalSidebarOpen, setIsCalSidebarOpen] = useState(false);
  const [defaultCalendarId, setDefaultCalendarId] = useState(() => {
    return localStorage.getItem('kestrel_default_cal_id') || 'cal-personal';
  });
  const [startHour, setStartHour] = useState(() => {
    return Number(localStorage.getItem('kestrel_cal_start_hour') || '8');
  });
  const [showWeekends, setShowWeekends] = useState(() => {
    return localStorage.getItem('kestrel_cal_show_weekends') !== 'false';
  });
  
  // Selected Event state (for Details Panel on right side)
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const [isEditingEvent, setIsEditingEvent] = useState(false);

  const [isMobileOrTablet, setIsMobileOrTablet] = useState(false);
  const [isHeaderMonthDropdownOpen, setIsHeaderMonthDropdownOpen] = useState(false);
  const [isMobileSearchOpen, setIsMobileSearchOpen] = useState(false);
  const [touchStartX, setTouchStartX] = useState<number | null>(null);
  const [touchStartY, setTouchStartY] = useState<number | null>(null);

  useEffect(() => {
    const handleResize = () => {
      setIsMobileOrTablet(window.innerWidth < 1024);
    };
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const handleTouchStart = (e: React.TouchEvent) => {
    setTouchStartX(e.touches[0].clientX);
    setTouchStartY(e.touches[0].clientY);
  };

  const handleTouchEnd = (e: React.TouchEvent) => {
    if (touchStartX === null || touchStartY === null) return;
    const diffX = touchStartX - e.changedTouches[0].clientX;
    const diffY = touchStartY - e.changedTouches[0].clientY;

    if (Math.abs(diffX) > Math.abs(diffY) * 1.5 && Math.abs(diffX) > 40) {
      if (diffX > 0) {
        handleNext();
      } else {
        handlePrev();
      }
    }
    setTouchStartX(null);
    setTouchStartY(null);
  };

  // Collapsible Accounts and associated Calendars state (Anonymized, secure fictional data)
  const [accounts, setAccounts] = useState<Account[]>(() => {
    const saved = localStorage.getItem('kestrel_calendar_accounts');
    if (saved && !saved.includes('prathamesh.pathak') && !saved.includes('iiml.ac.in')) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error('Failed to load accounts state', e);
      }
    }
    return [
      {
        id: 'acc-1',
        email: 'alex.rivera@kestrel.io',
        isExpanded: true,
        calendars: [
          { id: 'cal-personal', name: 'Personal Calendar', color: 'blue', isActive: true, isDefault: true },
          { id: 'cal-holidays', name: 'Public Holidays', color: 'green', isActive: true }
        ]
      },
      {
        id: 'acc-2',
        email: 'workspace@kestrel.io',
        isExpanded: true,
        calendars: [
          { id: 'cal-work', name: 'Core Team Sync', color: 'blue', isActive: true, isDefault: true },
          { id: 'cal-launches', name: 'Product Launches', color: 'orange', isActive: true },
          { id: 'cal-marketing', name: 'Marketing & SEO', color: 'purple', isActive: true },
          { id: 'cal-design', name: 'UI/UX Critiques', color: 'rose', isActive: true },
          { id: 'cal-strategy', name: 'Growth Strategy', color: 'teal', isActive: true },
          { id: 'cal-research', name: 'User Research', color: 'amber', isActive: true }
        ]
      }
    ];
  });

  // Save accounts state to LocalStorage
  useEffect(() => {
    localStorage.setItem('kestrel_calendar_accounts', JSON.stringify(accounts));
  }, [accounts]);

  // Load events from LocalStorage or pre-load sample ones
  const [events, setEvents] = useState<CalendarEvent[]>(() => {
    const saved = localStorage.getItem('kestrel_calendar_events');
    if (saved && saved.includes('evt-increff')) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error('Failed to parse calendar events', e);
      }
    }
    
    const today = new Date();
    const formatDate = (offset: number) => {
      const d = new Date(today);
      d.setDate(today.getDate() + offset);
      return d.toISOString().split('T')[0];
    };

    return [
      {
        id: 'evt-increff',
        title: 'Interview: Increff | Technical Product Manager',
        date: formatDate(0), // Today!
        startTime: '13:30',
        endTime: '13:50',
        description: 'Interview Link:\nhttps://app.fabrichq.ai/jobs/3c0ea9c1-225f-49cf-99f9-013da8d96054/?candidate_id=00ea652e-b83c-4ea0-8adb-b4d71f2d0c7e&utm_source=email&utm_medium=email_automation&utm_campaign=interview_scheduled&utm_content=description_link',
        color: 'rose',
        category: 'Work',
        calendarId: 'cal-personal',
        status: 'Scheduled',
        priority: 'High',
        isAllDay: false,
        organizer: 'hello@fabricai.tech',
        rsvpStatus: 'yes',
        attendees: [
          { name: 'Prathamesh Pathak', email: 'pprathamesh98@gmail.com', rsvp: 'yes' }
        ]
      },
      {
        id: 'evt-1',
        title: 'Workspace Design Sync',
        date: formatDate(0),
        startTime: '10:00',
        endTime: '11:00',
        description: 'Reviewing the latest theme guidelines, custom color palettes, and label nesting layouts.',
        color: 'blue',
        location: 'Gather Town Workspace',
        category: 'Workspace',
        calendarId: 'cal-personal',
        status: 'Next Up',
        priority: 'High',
        isAllDay: false,
        organizer: 'workspace@kestrel.io',
        rsvpStatus: 'maybe',
        attendees: [
          { name: 'Alex Rivera', email: 'workspace@kestrel.io', rsvp: 'maybe' }
        ]
      },
      {
        id: 'evt-2',
        title: 'Deep Work Session',
        date: formatDate(0),
        startTime: '14:30',
        endTime: '16:00',
        description: 'Focus block for implementing nested label trees and context menus.',
        color: 'purple',
        category: 'Work',
        calendarId: 'cal-work',
        status: 'Scheduled',
        priority: 'Medium',
        isAllDay: false
      },
      {
        id: 'evt-3',
        title: 'Product Strategy Align',
        date: formatDate(1), // Tomorrow
        startTime: '11:00',
        endTime: '12:00',
        description: 'Syncing on Q3 roadmap goals, marketing strategies, and key results.',
        color: 'teal',
        location: 'Zoom Room B',
        category: 'Work',
        calendarId: 'cal-work',
        status: 'Scheduled',
        priority: 'High',
        isAllDay: false,
        organizer: 'alex.rivera@kestrel.io',
        rsvpStatus: 'none'
      },
      {
        id: 'evt-4',
        title: 'Review PR & Code Quality',
        date: formatDate(1),
        startTime: '09:30',
        endTime: '10:30',
        description: 'Check compilation, TypeScript type-safety, and verify there are no console errors.',
        color: 'teal',
        location: 'Vite Terminal',
        category: 'Workspace',
        calendarId: 'cal-work',
        status: 'Scheduled',
        priority: 'Low',
        isAllDay: false
      },
      {
        id: 'evt-5',
        title: 'Lunch with Lead Designer',
        date: formatDate(1),
        startTime: '12:30',
        endTime: '13:30',
        description: 'Discussing scaling up full-stack Express capabilities and file attachment interfaces.',
        color: 'green',
        location: 'Gourmet Greens Cafe',
        category: 'Personal',
        calendarId: 'cal-personal',
        status: 'Completed',
        priority: 'None',
        isAllDay: false
      },
      {
        id: 'evt-6',
        title: 'Weekly Retro & Coffee',
        date: formatDate(3),
        startTime: '15:00',
        endTime: '16:00',
        description: 'Casual catch-up on week deliverables and styling updates.',
        color: 'orange',
        location: 'Kitchen Island',
        category: 'Personal',
        calendarId: 'cal-personal',
        status: 'Scheduled',
        priority: 'Medium',
        isAllDay: false
      },
      {
        id: 'evt-7',
        title: 'Launch Kestrel Mail v2',
        date: formatDate(-2),
        startTime: '08:00',
        endTime: '09:00',
        description: 'Production deployment to Cloud Run containers.',
        color: 'orange',
        category: 'Work',
        calendarId: 'cal-launches',
        status: 'Completed',
        priority: 'High',
        isAllDay: false
      },
      {
        id: 'evt-holiday-1',
        title: 'Public Holiday Celebration',
        date: formatDate(5),
        startTime: '00:00',
        endTime: '23:59',
        description: 'National public holiday celebration.',
        color: 'green',
        category: 'General',
        calendarId: 'cal-holidays',
        status: 'Scheduled',
        priority: 'None',
        isAllDay: true
      }
    ];
  });

  // Save to LocalStorage whenever events update
  useEffect(() => {
    localStorage.setItem('kestrel_calendar_events', JSON.stringify(events));
  }, [events]);

  // Form Fields State for creating/editing events
  const [formTitle, setFormTitle] = useState('');
  const [formDate, setFormDate] = useState('');
  const [formStartTime, setFormStartTime] = useState('09:00');
  const [formEndTime, setFormEndTime] = useState('10:00');
  const [formDescription, setFormDescription] = useState('');
  const [formColor, setFormColor] = useState('blue');
  const [formLocation, setFormLocation] = useState('');
  const [formCalendarId, setFormCalendarId] = useState('cal-personal');
  const [formStatus, setFormStatus] = useState<'Next Up' | 'Scheduled' | 'Completed' | 'In Progress'>('Next Up');
  const [formPriority, setFormPriority] = useState<'High' | 'Medium' | 'Low' | 'None'>('None');
  const [formIsAllDay, setFormIsAllDay] = useState(false);

  // States for positioning and layout modes of the Details Panel
  const [clickPosition, setClickPosition] = useState<{ x: number; y: number; rectWidth: number; rectLeft: number } | null>(null);
  const [isSidebarMode, setIsSidebarMode] = useState(false);

  // Active Calendar IDs calculated dynamically
  const activeCalendarIds = useMemo(() => {
    const list: string[] = [];
    accounts.forEach(acc => {
      acc.calendars.forEach(cal => {
        if (cal.isActive) {
          list.push(cal.id);
        }
      });
    });
    return list;
  }, [accounts]);

  // Active calendars flattened map for lookup names and colors
  const calendarsMap = useMemo(() => {
    const map: Record<string, { name: string; color: string; email: string }> = {};
    accounts.forEach(acc => {
      acc.calendars.forEach(cal => {
        map[cal.id] = { name: cal.name, color: cal.color, email: acc.email };
      });
    });
    return map;
  }, [accounts]);

  const scrollContainerRef = React.useRef<HTMLDivElement>(null);

  // Timezone Offset String
  const tzOffset = useMemo(() => {
    try {
      const offsetMin = -new Date().getTimezoneOffset();
      const hours = Math.floor(Math.abs(offsetMin) / 60);
      const mins = Math.abs(offsetMin) % 60;
      const sign = offsetMin >= 0 ? '+' : '-';
      const padMins = mins.toString().padStart(2, '0');
      return `GMT${sign}${hours}:${padMins}`;
    } catch {
      return 'GMT+00:00';
    }
  }, []);

  // 24 Hours list
  const HOURS = useMemo(() => {
    return Array.from({ length: 24 }).map((_, i) => {
      const ampm = i >= 12 ? 'PM' : 'AM';
      const hour12 = i % 12 === 0 ? 12 : i % 12;
      return {
        hour24: i,
        label: `${hour12}${ampm}`,
      };
    });
  }, []);

  // Scroll to startHour on load / view mode change
  useEffect(() => {
    if (scrollContainerRef.current) {
      scrollContainerRef.current.scrollTop = startHour * 60;
    }
  }, [viewMode, startHour]);

  // Layout calculations for overlapping events
  const getTimedEventsLayout = (timedEvts: CalendarEvent[]) => {
    const sorted = [...timedEvts].sort((a, b) => a.startTime.localeCompare(b.startTime));
    const groups: Array<typeof sorted> = [];
    
    sorted.forEach(evt => {
      const [sh, sm] = (evt.startTime || '09:00').split(':').map(Number);
      const start = sh + sm / 60;
      const [eh, em] = (evt.endTime || '10:00').split(':').map(Number);
      const end = Math.max(start + 0.5, eh + em / 60);
      
      let added = false;
      for (const g of groups) {
        const overlapsAny = g.some(other => {
          const [osh, osm] = (other.startTime || '09:00').split(':').map(Number);
          const ostart = osh + osm / 60;
          const [oeh, oem] = (other.endTime || '10:00').split(':').map(Number);
          const oend = Math.max(ostart + 0.5, oeh + oem / 60);
          return (start < oend && end > ostart);
        });
        if (overlapsAny) {
          g.push(evt);
          added = true;
          break;
        }
      }
      if (!added) {
        groups.push([evt]);
      }
    });
    
    const layouts: Record<string, { left: number; width: number; start: number; end: number }> = {};
    groups.forEach(g => {
      const columns: string[][] = [];
      g.forEach(evt => {
        const [sh, sm] = (evt.startTime || '09:00').split(':').map(Number);
        const start = sh + sm / 60;
        const [eh, em] = (evt.endTime || '10:00').split(':').map(Number);
        const end = Math.max(start + 0.5, eh + em / 60);
        
        let colIdx = 0;
        while (true) {
          if (!columns[colIdx]) {
            columns[colIdx] = [];
          }
          const hasOverlap = columns[colIdx].some(otherId => {
            const other = g.find(o => o.id === otherId)!;
            const [osh, osm] = (other.startTime || '09:00').split(':').map(Number);
            const ostart = osh + osm / 60;
            const [oeh, oem] = (other.endTime || '10:00').split(':').map(Number);
            const oend = Math.max(ostart + 0.5, oeh + oem / 60);
            return (start < oend && end > ostart);
          });
          if (!hasOverlap) {
            columns[colIdx].push(evt.id);
            break;
          }
          colIdx++;
        }
      });
      
      const totalCols = columns.length;
      g.forEach(evt => {
        const [sh, sm] = (evt.startTime || '09:00').split(':').map(Number);
        const start = sh + sm / 60;
        const [eh, em] = (evt.endTime || '10:00').split(':').map(Number);
        const end = Math.max(start + 0.5, eh + em / 60);
        
        let colIdx = columns.findIndex(col => col.includes(evt.id));
        layouts[evt.id] = {
          left: (colIdx / totalCols) * 100,
          width: 100 / totalCols,
          start,
          end
        };
      });
    });
    
    return layouts;
  };

  // Helper for clicking on empty hourly slot
  const handleColumnClick = (dateStr: string, e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const clickY = e.clientY - rect.top;
    const totalHeight = rect.height;
    
    const hoursDecimal = (clickY / totalHeight) * 24;
    const hour = Math.floor(hoursDecimal);
    const minutesDecimal = (hoursDecimal - hour) * 60;
    const minutes = Math.floor(minutesDecimal / 15) * 15;
    
    const pad = (num: number) => num.toString().padStart(2, '0');
    const startTime = `${pad(hour)}:${pad(minutes)}`;
    const endHour = hour === 23 ? 23 : hour + 1;
    const endMinutes = hour === 23 ? 59 : minutes;
    const endTime = `${pad(endHour)}:${pad(endMinutes)}`;
    
    setFormTitle('');
    setFormDate(dateStr);
    setFormStartTime(startTime);
    setFormEndTime(endTime);
    setFormDescription('');
    setFormColor('blue');
    setFormLocation('');
    setFormCalendarId(activeCalendarIds[0] || 'cal-personal');
    setFormStatus('Next Up');
    setFormPriority('None');
    setFormIsAllDay(false);
    setSelectedEventId(null);
    setIsEditingEvent(true);
    
    const parentElement = document.getElementById('calendar-workspace-layout');
    const parentRect = parentElement?.getBoundingClientRect();
    if (parentRect) {
      setClickPosition({
        x: e.clientX - parentRect.left,
        y: e.clientY - parentRect.top,
        rectWidth: rect.width,
        rectLeft: rect.left - parentRect.left
      });
    }
  };

  // Filter and Search events based on active calendars & query
  const filteredEvents = useMemo(() => {
    return events.filter(evt => {
      // Check if calendar is active
      const calId = evt.calendarId || 'cal-main-1';
      if (!activeCalendarIds.includes(calId)) {
        return false;
      }

      const matchesSearch = 
        evt.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (evt.description || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
        (evt.location || '').toLowerCase().includes(searchQuery.toLowerCase());
      
      return matchesSearch;
    });
  }, [events, searchQuery, activeCalendarIds]);

  // Currently selected event object
  const selectedEvent = useMemo(() => {
    if (!selectedEventId) return null;
    return events.find(e => e.id === selectedEventId) || null;
  }, [selectedEventId, events]);

  // Sync edit form with selectedEvent when switching
  useEffect(() => {
    if (selectedEvent) {
      setFormTitle(selectedEvent.title);
      setFormDate(selectedEvent.date);
      setFormStartTime(selectedEvent.startTime || '09:00');
      setFormEndTime(selectedEvent.endTime || '10:00');
      setFormDescription(selectedEvent.description || '');
      setFormColor(selectedEvent.color || 'blue');
      setFormLocation(selectedEvent.location || '');
      setFormCalendarId(selectedEvent.calendarId || 'cal-main-1');
      setFormStatus(selectedEvent.status || 'Next Up');
      setFormPriority(selectedEvent.priority || 'None');
      setFormIsAllDay(selectedEvent.isAllDay || false);
    }
  }, [selectedEvent]);

  // Group events by date for fast lookup in month/week views
  const eventsByDate = useMemo(() => {
    const map: Record<string, CalendarEvent[]> = {};
    filteredEvents.forEach(evt => {
      if (!map[evt.date]) {
        map[evt.date] = [];
      }
      map[evt.date].push(evt);
    });
    // Sort events in each day by startTime
    Object.keys(map).forEach(date => {
      map[date].sort((a, b) => a.startTime.localeCompare(b.startTime));
    });
    return map;
  }, [filteredEvents]);

  // Month and View mode navigation
  const handlePrev = () => {
    const nextDate = new Date(currentDate);
    if (viewMode === 'month') {
      nextDate.setMonth(currentDate.getMonth() - 1);
    } else if (viewMode === 'week' || viewMode === 'weekdays' || viewMode === 'agenda') {
      nextDate.setDate(currentDate.getDate() - 7);
    } else if (viewMode.endsWith('-day')) {
      const daysCount = parseInt(viewMode, 10) || 1;
      nextDate.setDate(currentDate.getDate() - daysCount);
    } else {
      nextDate.setDate(currentDate.getDate() - 1);
    }
    setCurrentDate(nextDate);
  };

  const handleNext = () => {
    const nextDate = new Date(currentDate);
    if (viewMode === 'month') {
      nextDate.setMonth(currentDate.getMonth() + 1);
    } else if (viewMode === 'week' || viewMode === 'weekdays' || viewMode === 'agenda') {
      nextDate.setDate(currentDate.getDate() + 7);
    } else if (viewMode.endsWith('-day')) {
      const daysCount = parseInt(viewMode, 10) || 1;
      nextDate.setDate(currentDate.getDate() + daysCount);
    } else {
      nextDate.setDate(currentDate.getDate() + 1);
    }
    setCurrentDate(nextDate);
  };

  const handleToday = () => {
    const today = new Date();
    setCurrentDate(today);
    setMiniMonth(today);
  };

  // Keyboard shortcuts for view switching
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        target?.tagName === 'INPUT' ||
        target?.tagName === 'TEXTAREA' ||
        target?.tagName === 'SELECT' ||
        target?.isContentEditable
      ) {
        return;
      }

      const key = e.key.toLowerCase();
      if (key >= '1' && key <= '7') {
        const mode = key === '1' ? 'day' : `${key}-day` as any;
        setViewMode(mode);
        showToast(`View mode changed to: ${key === '1' ? '1 Day' : `${key} Days`}`, 'info');
      } else if (key === 'd') {
        setViewMode('day');
        showToast('View mode changed to: Day', 'info');
      } else if (key === '0' || key === 'w') {
        setViewMode('week');
        showToast('View mode changed to: Week', 'info');
      } else if (key === 'm') {
        setViewMode('month');
        showToast('View mode changed to: Month', 'info');
      } else if (key === 'a') {
        setViewMode('agenda');
        showToast('View mode changed to: Agenda', 'info');
      } else if (key === 't') {
        handleToday();
        showToast('Navigated to Today', 'info');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [showToast]);

  // Mini-calendar Month Navigation
  const handleMiniMonthPrev = () => {
    const d = new Date(miniMonth);
    d.setMonth(d.getMonth() - 1);
    setMiniMonth(d);
  };

  const handleMiniMonthNext = () => {
    const d = new Date(miniMonth);
    d.setMonth(d.getMonth() + 1);
    setMiniMonth(d);
  };

  // Helper for formatting main header label
  const getHeaderLabel = () => {
    const months = [
      'January', 'February', 'March', 'April', 'May', 'June',
      'July', 'August', 'September', 'October', 'November', 'December'
    ];
    const month = months[currentDate.getMonth()];
    const year = currentDate.getFullYear();
    
    if (viewMode === 'month') {
      return `${month} ${year}`;
    } else if (viewMode === 'day') {
      return `${month} ${currentDate.getDate()}, ${year}`;
    } else if (viewMode === 'agenda') {
      return `Agenda — ${month} ${year}`;
    } else {
      // For week, 3-day, 4-day, weekdays
      const days = displayDays;
      if (days.length === 0) return `${month} ${year}`;
      const firstDay = new Date(days[0].dateStr);
      const lastDay = new Date(days[days.length - 1].dateStr);
      
      if (firstDay.getMonth() === lastDay.getMonth()) {
        return `${months[firstDay.getMonth()]} ${firstDay.getFullYear()}`;
      } else if (firstDay.getFullYear() === lastDay.getFullYear()) {
        return `${months[firstDay.getMonth()]} - ${months[lastDay.getMonth()]} ${firstDay.getFullYear()}`;
      } else {
        return `${months[firstDay.getMonth()]} ${firstDay.getFullYear()} - ${months[lastDay.getMonth()]} ${lastDay.getFullYear()}`;
      }
    }
  };

  // Month Grid helper for the center area
  const monthCells = useMemo(() => {
    const year = currentDate.getFullYear();
    const month = currentDate.getMonth();

    const firstDay = new Date(year, month, 1);
    const startOffset = firstDay.getDay();

    const lastDay = new Date(year, month + 1, 0);
    const totalDays = lastDay.getDate();

    const prevLastDay = new Date(year, month, 0).getDate();

    const cells: Array<{ date: string; dayNum: number; isCurrentMonth: boolean; key: string }> = [];

    // Prev month padding
    for (let i = startOffset - 1; i >= 0; i--) {
      const prevDate = new Date(year, month - 1, prevLastDay - i);
      cells.push({
        date: prevDate.toISOString().split('T')[0],
        dayNum: prevLastDay - i,
        isCurrentMonth: false,
        key: `prev-${prevLastDay - i}`
      });
    }

    // Current month days
    for (let i = 1; i <= totalDays; i++) {
      const currDate = new Date(year, month, i);
      cells.push({
        date: currDate.toISOString().split('T')[0],
        dayNum: i,
        isCurrentMonth: true,
        key: `curr-${i}`
      });
    }

    // Next month padding
    const remaining = 42 - cells.length;
    for (let i = 1; i <= remaining; i++) {
      const nextDate = new Date(year, month + 1, i);
      cells.push({
        date: nextDate.toISOString().split('T')[0],
        dayNum: i,
        isCurrentMonth: false,
        key: `next-${i}`
      });
    }

    return cells;
  }, [currentDate]);

  // Mini calendar cells generator
  const miniCells = useMemo(() => {
    const year = miniMonth.getFullYear();
    const month = miniMonth.getMonth();

    const firstDay = new Date(year, month, 1);
    const startOffset = firstDay.getDay();

    const lastDay = new Date(year, month + 1, 0);
    const totalDays = lastDay.getDate();

    const prevLastDay = new Date(year, month, 0).getDate();

    const cells: Array<{ date: string; dayNum: number; isCurrentMonth: boolean; key: string }> = [];

    // Prev month
    for (let i = startOffset - 1; i >= 0; i--) {
      const prevDate = new Date(year, month - 1, prevLastDay - i);
      cells.push({
        date: prevDate.toISOString().split('T')[0],
        dayNum: prevLastDay - i,
        isCurrentMonth: false,
        key: `mini-prev-${prevLastDay - i}`
      });
    }

    // Current month
    for (let i = 1; i <= totalDays; i++) {
      const currDate = new Date(year, month, i);
      cells.push({
        date: currDate.toISOString().split('T')[0],
        dayNum: i,
        isCurrentMonth: true,
        key: `mini-curr-${i}`
      });
    }

    // Next month padding
    const remaining = 42 - cells.length;
    for (let i = 1; i <= remaining; i++) {
      const nextDate = new Date(year, month + 1, i);
      cells.push({
        date: nextDate.toISOString().split('T')[0],
        dayNum: i,
        isCurrentMonth: false,
        key: `mini-next-${i}`
      });
    }

    return cells;
  }, [miniMonth]);

  // List of days depending on active viewMode: week, custom multi-day, weekdays, day
  const displayDays = useMemo(() => {
    let daysList: Date[] = [];
    if (viewMode === 'day') {
      const d = new Date(currentDate);
      daysList = [d];
    } else if (viewMode.endsWith('-day')) {
      const daysCount = parseInt(viewMode, 10) || 1;
      daysList = Array.from({ length: daysCount }).map((_, i) => {
        const d = new Date(currentDate);
        d.setDate(currentDate.getDate() + i);
        return d;
      });
    } else if (viewMode === 'weekdays') {
      const startOfWeek = new Date(currentDate);
      startOfWeek.setDate(currentDate.getDate() - currentDate.getDay() + 1); // Monday of this week
      daysList = Array.from({ length: 5 }).map((_, i) => {
        const d = new Date(startOfWeek);
        d.setDate(startOfWeek.getDate() + i);
        return d;
      });
    } else {
      // Default 'week'
      const startOfWeek = new Date(currentDate);
      startOfWeek.setDate(currentDate.getDate() - currentDate.getDay());
      daysList = Array.from({ length: 7 }).map((_, i) => {
        const d = new Date(startOfWeek);
        d.setDate(startOfWeek.getDate() + i);
        return d;
      });
    }

    // Filter weekends if showWeekends is false and we aren't in a single day view
    if (!showWeekends && viewMode !== 'day' && viewMode !== '1-day') {
      daysList = daysList.filter(d => d.getDay() !== 0 && d.getDay() !== 6);
    }

    const labels = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    return daysList.map(d => ({
      dateStr: d.toISOString().split('T')[0],
      dayNum: d.getDate(),
      dayLabel: labels[d.getDay()],
      isToday: d.toDateString() === new Date().toDateString()
    }));
  }, [currentDate, viewMode, showWeekends]);

  // Day View date string
  const dayStr = useMemo(() => {
    return currentDate.toISOString().split('T')[0];
  }, [currentDate]);

  // Toggle account lists expanded state
  const toggleAccountExpand = (accountId: string) => {
    setAccounts(prev => prev.map(acc => 
      acc.id === accountId ? { ...acc, isExpanded: !acc.isExpanded } : acc
    ));
  };

  // Toggle single calendar active state
  const toggleCalendarActive = (accountId: string, calendarId: string) => {
    setAccounts(prev => prev.map(acc => {
      if (acc.id === accountId) {
        return {
          ...acc,
          calendars: acc.calendars.map(cal => 
            cal.id === calendarId ? { ...cal, isActive: !cal.isActive } : cal
          )
        };
      }
      return acc;
    }));
  };

  // Open Details Panel to create a new event
  const handleOpenCreateForm = (targetDateStr?: string, e?: React.MouseEvent) => {
    const initialDate = targetDateStr || currentDate.toISOString().split('T')[0];
    
    // Pick the first active calendar ID
    const activeCalId = activeCalendarIds[0] || 'cal-personal';
    
    setFormTitle('');
    setFormDate(initialDate);
    setFormStartTime('10:00');
    setFormEndTime('11:00');
    setFormDescription('');
    setFormColor('blue');
    setFormLocation('');
    setFormCalendarId(activeCalId);
    setFormStatus('Next Up');
    setFormPriority('None');
    setFormIsAllDay(false);

    setSelectedEventId(null);
    setIsEditingEvent(true); // Open in Edit Mode in the details pane

    if (e) {
      const rect = e.currentTarget.getBoundingClientRect();
      const parentElement = document.getElementById('calendar-workspace-layout');
      const parentRect = parentElement?.getBoundingClientRect();
      if (parentRect) {
        setClickPosition({
          x: rect.right - parentRect.left,
          y: rect.top - parentRect.top,
          rectWidth: rect.width,
          rectLeft: rect.left - parentRect.left
        });
      }
    } else {
      setClickPosition(null);
    }
  };

  // Open Details Panel for existing event
  const handleSelectEvent = (event: CalendarEvent, e?: React.MouseEvent) => {
    setSelectedEventId(event.id);
    setIsEditingEvent(false); // Open in View Mode first

    if (e) {
      const rect = e.currentTarget.getBoundingClientRect();
      const parentElement = document.getElementById('calendar-workspace-layout');
      const parentRect = parentElement?.getBoundingClientRect();
      if (parentRect) {
        setClickPosition({
          x: rect.right - parentRect.left,
          y: rect.top - parentRect.top,
          rectWidth: rect.width,
          rectLeft: rect.left - parentRect.left
        });
      }
    } else {
      setClickPosition(null);
    }
  };

  // Save changes (creates or updates) from the Details panel
  const handleSaveForm = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!formTitle.trim()) {
      showToast('Event title is required.', 'error');
      return;
    }

    if (selectedEventId) {
      // Update existing
      setEvents(prev => prev.map(evt => evt.id === selectedEventId ? {
        ...evt,
        title: formTitle.trim(),
        date: formDate,
        startTime: formStartTime,
        endTime: formEndTime,
        description: formDescription.trim(),
        color: formColor,
        location: formLocation.trim(),
        calendarId: formCalendarId,
        status: formStatus,
        priority: formPriority,
        isAllDay: formIsAllDay
      } : evt));
      showToast('Event updated successfully.', 'success');
      setIsEditingEvent(false);
    } else {
      // Create new
      const newEvt: CalendarEvent = {
        id: `evt-${Date.now()}`,
        title: formTitle.trim(),
        date: formDate,
        startTime: formStartTime,
        endTime: formEndTime,
        description: formDescription.trim(),
        color: formColor,
        location: formLocation.trim(),
        calendarId: formCalendarId,
        status: formStatus,
        priority: formPriority,
        isAllDay: formIsAllDay
      };
      setEvents(prev => [...prev, newEvt]);
      setSelectedEventId(newEvt.id);
      setIsEditingEvent(false);
      showToast('Event created successfully.', 'success');
    }
  };

  // Delete event
  const handleDeleteEvent = (id: string) => {
    setEvents(prev => prev.filter(evt => evt.id !== id));
    setSelectedEventId(null);
    setIsEditingEvent(false);
    showToast('Event deleted permanently.', 'success');
  };

  // Helper to update RSVP state
  const handleUpdateRSVP = (eventId: string, rsvp: 'yes' | 'no' | 'maybe') => {
    setEvents(prev => prev.map(evt => {
      if (evt.id === eventId) {
        return { ...evt, rsvpStatus: rsvp };
      }
      return evt;
    }));
    showToast(`RSVP status set to: ${rsvp.toUpperCase()}`, 'success');
  };

  // Helper to dynamically calculate position for the floating details panel
  const getFloatingPanelStyle = () => {
    if (!clickPosition) {
      return {
        position: 'absolute' as const,
        top: '80px',
        right: '16px',
        zIndex: 50,
        maxHeight: 'calc(100vh - 100px)'
      };
    }

    const cardWidth = 380;
    const container = document.getElementById('calendar-workspace-layout');
    const containerRect = container?.getBoundingClientRect();
    const containerWidth = containerRect?.width || window.innerWidth;
    const containerHeight = containerRect?.height || window.innerHeight;
    
    let leftPosition = clickPosition.x + 12;
    // If it exceeds the container boundary, flip to the left
    if (leftPosition + cardWidth > containerWidth - 16) {
      leftPosition = clickPosition.rectLeft - cardWidth - 12;
    }
    
    // Ensure it doesn't go below 16px from left
    if (leftPosition < 16) {
      leftPosition = 16;
    }

    // Align vertical position y, but cap it so it doesn't go off-screen at the bottom.
    let topPosition = clickPosition.y;
    const idealHeight = 580; // approximate ideal height of card
    
    // If positioning at y would make it overflow the bottom, try to push it up
    if (topPosition + idealHeight > containerHeight - 16) {
      topPosition = containerHeight - idealHeight - 16;
    }
    
    // But never let topPosition go above the top padding (80px)
    if (topPosition < 80) {
      topPosition = 80;
    }

    // Calculate maximum height based on actual topPosition to guarantee it NEVER overflows the bottom of the container
    const maxHeightValue = Math.max(300, containerHeight - topPosition - 16);

    return {
      position: 'absolute' as const,
      top: `${topPosition}px`,
      left: `${leftPosition}px`,
      zIndex: 50,
      maxHeight: `${maxHeightValue}px`,
    };
  };

  // Helper to get props for the side panel container
  const getPanelContainerProps = () => {
    if (isMobileOrTablet) {
      return {
        className: "fixed bottom-0 inset-x-0 max-h-[75vh] bg-[#131313] border-t border-neutral-800 rounded-t-3xl shadow-2xl flex flex-col select-none overflow-hidden animate-slide-up z-50",
        style: {}
      };
    } else if (isSidebarMode) {
      return {
        className: "w-80 bg-[#0d0d0d] border-l border-border-hairline flex flex-col h-full shrink-0 select-none animate-slide-left relative z-40",
        style: {}
      };
    } else {
      const style = getFloatingPanelStyle();
      return {
        className: "absolute w-[380px] bg-[#131313]/95 backdrop-blur-md border border-neutral-800 rounded-2xl shadow-2xl flex flex-col select-none overflow-hidden animate-scale-in z-50",
        style
      };
    }
  };

  return (
    <div id="calendar-workspace-layout" className="flex-1 flex h-screen w-full bg-canvas-base text-text-primary font-sans overflow-hidden antialiased relative">
      
      {/* Mobile Sidebar Backdrop */}
      {isCalSidebarOpen && (
        <div 
          id="calendar-sidebar-mobile-backdrop" 
          className="fixed inset-0 bg-black/60 backdrop-blur-xs z-30 md:hidden"
          onClick={() => setIsCalSidebarOpen(false)}
        />
      )}

      {/* 1. LEFT SIDEBAR (Mini Calendar & Account Lists) */}
      <aside 
        id="calendar-left-sidebar" 
        className={`fixed inset-y-0 left-0 z-40 w-72 bg-[#0c0c0c] border-r border-border-hairline flex flex-col h-full shrink-0 select-none transition-transform duration-300 ease-in-out md:relative md:translate-x-0 ${
          isCalSidebarOpen ? 'translate-x-0' : '-translate-x-full'
        }`}
      >
               {/* Workspace Brand Header / Search */}
        <div className="px-4 py-3 border-b border-border-hairline flex items-center justify-between gap-2 bg-[#090909]">
          {/* Mobile Close Sidebar Button */}
          {isMobileOrTablet ? (
            <div className="flex items-center gap-2.5">
              <button
                onClick={() => setIsCalSidebarOpen(false)}
                className="p-1.5 rounded-lg bg-canvas-card hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer border border-border-hairline/65 flex items-center justify-center shrink-0"
                title="Close Sidebar"
              >
                <X className="w-3.5 h-3.5" />
              </button>
              <span className="text-xs font-bold font-mono tracking-wider uppercase text-text-primary">Kestrel Calendar</span>
            </div>
          ) : (
            <div className="flex items-center gap-2 w-full">
              <div className="relative flex-1">
                <Search className="w-3.5 h-3.5 text-text-secondary absolute left-2.5 top-2.5" />
                <input
                  id="calendar-search-input"
                  type="text"
                  placeholder="Search events..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="bg-canvas-card hover:bg-canvas-hover/40 focus:bg-canvas-hover text-text-primary text-xs rounded-lg pl-8 pr-8 py-1.5 w-full outline-none border border-border-hairline focus:border-white/20 transition-all placeholder:text-text-secondary/40"
                />
                {searchQuery && (
                  <button
                    onClick={() => setSearchQuery('')}
                    className="absolute right-2 top-2 text-text-secondary/50 hover:text-text-primary p-0.5 rounded cursor-pointer"
                  >
                    <X className="w-3 h-3" />
                  </button>
                )}
              </div>
              <button
                id="btn-new-event-sidebar"
                onClick={(e) => handleOpenCreateForm(undefined, e)}
                className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors shrink-0"
                title="Add Event"
              >
                <Plus className="w-4 h-4 stroke-[2.5]" />
              </button>
            </div>
          )}
        </div>

        {/* Sidebar Content (Scrollable) */}
        <div className="flex-1 overflow-y-auto px-4 py-4 space-y-6 scrollbar-none">
          
          {/* A. Compact Mini Calendar */}
          {!isMobileOrTablet && (
            <div id="mini-calendar" className="space-y-3 p-1">
              
              {/* Mini Calendar Header Controls */}
              <div className="flex items-center justify-between px-1">
                <span className="text-xs font-mono font-medium text-text-primary">
                  {miniMonth.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })}
                </span>
                <div className="flex items-center gap-1">
                  <button 
                    id="mini-nav-prev"
                    onClick={handleMiniMonthPrev}
                    className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
                  >
                    <ChevronLeft className="w-3.5 h-3.5" />
                  </button>
                  <button 
                    id="mini-nav-next"
                    onClick={handleMiniMonthNext}
                    className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
                  >
                    <ChevronRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              {/* Mini Calendar Grid */}
              <div className="grid grid-cols-7 text-center gap-y-1">
                
                {/* Day initials */}
                {['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'].map(d => (
                  <div key={d} className="text-[10px] font-mono text-text-secondary/50 font-medium py-0.5">
                    {d}
                  </div>
                ))}

                {/* Grid cells */}
                {miniCells.map(cell => {
                  const isToday = cell.date === new Date().toISOString().split('T')[0];
                  const isSelected = cell.date === currentDate.toISOString().split('T')[0];
                  const hasEvents = eventsByDate[cell.date] && eventsByDate[cell.date].length > 0;
                  
                  return (
                    <button
                      key={cell.key}
                      onClick={() => {
                        const d = new Date(cell.date);
                        setCurrentDate(d);
                        setMiniMonth(d);
                      }}
                      className={`h-6 w-full text-[10px] font-mono rounded flex flex-col items-center justify-center transition-all cursor-pointer relative ${
                        cell.isCurrentMonth ? 'text-text-primary hover:bg-canvas-hover' : 'text-text-secondary/30'
                      } ${
                        isToday 
                          ? 'bg-[#d15b47] text-white font-semibold shadow-sm hover:bg-[#d15b47]/90' 
                          : isSelected 
                          ? 'border border-white/20 bg-canvas-hover text-white font-semibold' 
                          : ''
                      }`}
                    >
                      <span className={isToday || isSelected ? 'font-semibold' : ''}>{cell.dayNum}</span>
                      {hasEvents && (
                        <span className={`absolute bottom-0.5 w-1 h-1 rounded-full ${
                          isToday || isSelected ? 'bg-white' : 'bg-[#d15b47]'
                        }`} />
                      )}
                    </button>
                  );
                })}

              </div>
            </div>
          )}

          {/* View Modes Selection for Mobile/Tablet in Sidebar */}
          {isMobileOrTablet && (
            <div id="sidebar-view-modes" className="space-y-3 pt-1">
              <h3 className="px-1 text-[10px] font-mono tracking-wider text-text-secondary/60 uppercase font-semibold">
                View Modes
              </h3>
              <div className="space-y-1">
                {[
                  { label: 'Month', mode: 'month', icon: CalendarDays },
                  { label: 'Week', mode: 'week', icon: CalendarIcon },
                  { label: 'Day', mode: 'day', icon: Clock },
                  { label: 'Agenda', mode: 'agenda', icon: AlignLeft },
                ].map(item => {
                  const isSelected = viewMode === item.mode;
                  const Icon = item.icon;
                  return (
                    <button
                      key={item.mode}
                      onClick={() => {
                        setViewMode(item.mode as any);
                        setIsCalSidebarOpen(false); // Close sidebar on mobile
                      }}
                      className={`w-full flex items-center justify-between px-2.5 py-2 rounded-lg text-xs font-mono transition-colors cursor-pointer ${
                        isSelected 
                          ? 'bg-white text-black font-semibold' 
                          : 'text-text-secondary hover:text-text-primary hover:bg-canvas-hover'
                      }`}
                    >
                      <div className="flex items-center gap-2.5">
                        <Icon className="w-4 h-4 shrink-0" />
                        <span>{item.label}</span>
                      </div>
                      {isSelected && <Check className="w-3.5 h-3.5 stroke-[2.5]" />}
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {/* B. Collapsible Accounts & Associated Calendars */}
          <div id="accounts-and-calendars" className="space-y-4 pt-1">
            <h3 className="px-1 text-[10px] font-mono tracking-wider text-text-secondary/60 uppercase font-semibold">
              Calendars & Accounts
            </h3>

            <div className="space-y-3">
              {accounts.map(acc => (
                <div key={acc.id} className="space-y-1.5">
                  
                  {/* Account Row */}
                  <button
                    onClick={() => toggleAccountExpand(acc.id)}
                    className="w-full flex items-center justify-between px-1 py-1 rounded text-left text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-canvas-hover/40 transition-colors cursor-pointer group"
                  >
                    <span className="truncate pr-2 font-mono text-[11px]" title={acc.email}>
                      {acc.email}
                    </span>
                    <ChevronDown className={`w-3.5 h-3.5 text-text-secondary/60 transition-transform ${acc.isExpanded ? '' : '-rotate-90'}`} />
                  </button>

                  {/* Calendars List */}
                  {acc.isExpanded && (
                    <div className="space-y-0.5 pl-1.5 animate-slide-down">
                      {acc.calendars.map(cal => {
                        const colOpt = COLOR_OPTIONS.find(c => c.name === cal.color) || COLOR_OPTIONS[0];
                        return (
                          <div
                            key={cal.id}
                            className="w-full flex items-center justify-between py-1 px-1.5 rounded text-xs transition-colors hover:bg-canvas-hover/30"
                          >
                            <button
                              onClick={() => toggleCalendarActive(acc.id, cal.id)}
                              className="flex items-center gap-2.5 text-left text-text-primary/90 hover:text-text-primary cursor-pointer flex-1 truncate"
                            >
                              {/* Colored Square Checkbox */}
                              <div 
                                className={`w-3.5 h-3.5 rounded border transition-colors flex items-center justify-center shrink-0 ${
                                  cal.isActive 
                                    ? `border-transparent` 
                                    : 'border-border-hairline bg-transparent'
                                }`}
                                style={{ backgroundColor: cal.isActive ? colOpt.hex : undefined }}
                              >
                                {cal.isActive && <Check className="w-2.5 h-2.5 text-black stroke-[3]" />}
                              </div>
                              <span className="truncate text-text-primary/80 font-medium text-xs">
                                {cal.name}
                              </span>
                              {cal.isDefault && (
                                <span className="text-[8px] font-mono text-text-secondary/40 border border-border-hairline px-1 rounded">
                                  Default
                                </span>
                              )}
                            </button>

                            {/* Eye/Toggle state */}
                            <button
                              onClick={() => toggleCalendarActive(acc.id, cal.id)}
                              className="p-0.5 rounded text-text-secondary/40 hover:text-text-secondary transition-colors cursor-pointer ml-1"
                              title={cal.isActive ? 'Hide' : 'Show'}
                            >
                              {cal.isActive ? (
                                <Eye className="w-3 h-3" />
                              ) : (
                                <EyeOff className="w-3 h-3 text-text-secondary/25" />
                              )}
                            </button>
                          </div>
                        );
                      })}
                    </div>
                  )}

                </div>
              ))}
            </div>
          </div>

        </div>

        {/* Footer info/controls */}
        <div className="p-3 border-t border-border-hairline bg-[#090909] flex flex-col gap-1 mt-auto">
          <button 
            id="btn-settings-calendar"
            onClick={() => setIsSettingsOpen(true)}
            className="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors text-left cursor-pointer font-mono"
          >
            <Settings className="w-4 h-4 text-text-secondary" />
            <span>Settings</span>
          </button>
        </div>

      </aside>

      {/* 2. CENTER AREA (Calendar Grids & Month/Week/Day Views) */}
      <main id="calendar-center-column" className="flex-1 flex flex-col h-full overflow-hidden bg-canvas-base">
        
        {/* Responsive Header Controls Bar */}
        {isMobileOrTablet ? (
          /* Mobile & Tablet Header */
          <div id="calendar-header-mobile" className="px-4 py-3 border-b border-border-hairline flex items-center justify-between gap-2 bg-[#0a0a0a] relative select-none animate-fade-in">
            
            {/* Left side: Hamburger, Month/Year + Dropdown Icon */}
            <div className="flex items-center gap-2 relative">
              <button
                onClick={() => setIsCalSidebarOpen(true)}
                className="p-1.5 rounded-lg bg-canvas-card hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer border border-border-hairline/65 flex items-center justify-center shrink-0"
                title="Open Sidebar"
              >
                <Menu className="w-4 h-4" />
              </button>

              <button
                onClick={() => setIsHeaderMonthDropdownOpen(!isHeaderMonthDropdownOpen)}
                className="flex items-center gap-1 text-sm font-semibold text-text-primary font-mono tracking-tight cursor-pointer hover:text-white p-1 rounded transition-colors"
              >
                <span>{currentDate.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })}</span>
                <ChevronDown className={`w-3.5 h-3.5 text-text-secondary transition-transform duration-200 ${isHeaderMonthDropdownOpen ? 'rotate-180' : ''}`} />
              </button>

              {isHeaderMonthDropdownOpen && (
                <>
                  <div 
                    className="fixed inset-0 z-40" 
                    onClick={() => setIsHeaderMonthDropdownOpen(false)} 
                  />
                  <div className="fixed left-4 right-4 top-16 sm:absolute sm:left-0 sm:right-auto sm:top-full sm:mt-2 sm:w-72 bg-[#131313] border border-neutral-800 rounded-xl shadow-2xl p-4 z-50 animate-scale-in space-y-3 max-w-[320px] mx-auto">
                    <div className="flex items-center justify-between px-1">
                      <span className="text-xs font-mono font-medium text-text-primary">
                        {miniMonth.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })}
                      </span>
                      <div className="flex items-center gap-1">
                        <button 
                          onClick={handleMiniMonthPrev}
                          className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
                        >
                          <ChevronLeft className="w-3.5 h-3.5" />
                        </button>
                        <button 
                          onClick={handleMiniMonthNext}
                          className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
                        >
                          <ChevronRight className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>

                    <div className="grid grid-cols-7 text-center gap-y-1">
                      {['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'].map(d => (
                        <div key={d} className="text-[10px] font-mono text-text-secondary/50 font-medium py-0.5">
                          {d}
                        </div>
                      ))}
                      {miniCells.map(cell => {
                        const isToday = cell.date === new Date().toISOString().split('T')[0];
                        const isSelected = cell.date === currentDate.toISOString().split('T')[0];
                        const hasEvents = eventsByDate[cell.date] && eventsByDate[cell.date].length > 0;
                        
                        return (
                          <button
                            key={cell.key}
                            onClick={() => {
                              const d = new Date(cell.date);
                              setCurrentDate(d);
                              setMiniMonth(d);
                              setIsHeaderMonthDropdownOpen(false);
                            }}
                            className={`h-6 w-full text-[10px] font-mono rounded flex flex-col items-center justify-center transition-all cursor-pointer relative ${
                              cell.isCurrentMonth ? 'text-text-primary hover:bg-canvas-hover' : 'text-text-secondary/30'
                            } ${
                              isToday 
                                ? 'bg-[#d15b47] text-white font-semibold shadow-sm hover:bg-[#d15b47]/90' 
                                : isSelected 
                                ? 'border border-white/20 bg-canvas-hover text-white font-semibold' 
                                : ''
                            }`}
                          >
                            <span className={isToday || isSelected ? 'font-semibold' : ''}>{cell.dayNum}</span>
                            {hasEvents && (
                              <span className={`absolute bottom-0.5 w-1 h-1 rounded-full ${
                                isToday || isSelected ? 'bg-white' : 'bg-[#d15b47]'
                              }`} />
                            )}
                          </button>
                        );
                      })}
                    </div>
                  </div>
                </>
              )}
            </div>

            {/* Right side: Search toggle and Today button */}
            <div className="flex items-center gap-2">
              {isMobileSearchOpen ? (
                <div className="flex items-center bg-canvas-card border border-border-hairline rounded-lg px-2 py-1 max-w-[120px] xs:max-w-[160px] sm:max-w-[200px] relative animate-scale-in">
                  <Search className="w-3.5 h-3.5 text-text-secondary shrink-0" />
                  <input
                    type="text"
                    placeholder="Search..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="bg-transparent text-text-primary text-xs w-full outline-none pl-1.5 pr-5 placeholder:text-text-secondary/40 font-mono"
                    autoFocus
                  />
                  {searchQuery ? (
                    <button
                      onClick={() => setSearchQuery('')}
                      className="absolute right-1 text-text-secondary/50 hover:text-text-primary p-0.5"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  ) : (
                    <button
                      onClick={() => setIsMobileSearchOpen(false)}
                      className="absolute right-1 text-text-secondary/50 hover:text-text-primary p-0.5"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  )}
                </div>
              ) : (
                <button
                  onClick={() => setIsMobileSearchOpen(true)}
                  className="p-1.5 rounded-lg hover:bg-white/5 text-text-secondary hover:text-text-primary transition-colors cursor-pointer flex items-center justify-center shrink-0 w-8 h-8"
                  title="Search Events"
                >
                  <Search className="w-4 h-4" />
                </button>
              )}

              <button
                onClick={handleToday}
                className="flex items-center justify-center hover:bg-white/5 active:bg-white/10 text-text-primary hover:text-white w-8 h-8 rounded-lg font-mono text-sm font-semibold cursor-pointer transition-colors"
                title="Go to Today"
              >
                {new Date().getDate()}
              </button>
            </div>

          </div>
        ) : (
          /* Desktop Header Controls Bar */
          <div id="calendar-header" className="px-6 py-3 border-b border-border-hairline flex flex-wrap items-center justify-between gap-4 bg-[#0a0a0a]">
          
          {/* Left Controls (Dropdown, Today, Arrows, Title) */}
          <div className="flex items-center gap-3 flex-wrap">
            {/* Toggle Sidebar Button on Mobile/Tablet */}
            <button
              onClick={() => setIsCalSidebarOpen(true)}
              className="md:hidden p-1.5 rounded-lg bg-canvas-card hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer border border-border-hairline/65 flex items-center justify-center shrink-0"
              title="Open Sidebar"
            >
              <Menu className="w-4 h-4" />
            </button>
            
            {/* View Mode Dropdown Select */}
            <div className="relative" id="calendar-view-dropdown-container">
              <button
                id="btn-calendar-view-selector"
                onClick={() => {
                  setIsViewDropdownOpen(!isViewDropdownOpen);
                  setDropdownSubmenu('none');
                }}
                className="bg-canvas-card hover:bg-canvas-hover border border-border-hairline rounded-lg px-3 py-1.5 text-xs text-text-primary flex items-center gap-1.5 transition-all font-mono font-medium cursor-pointer"
              >
                <span className="capitalize">
                  {viewMode === 'weekdays' 
                    ? 'Weekdays' 
                    : viewMode.endsWith('-day') 
                      ? viewMode.replace('-day', parseInt(viewMode) === 1 ? ' day' : ' days') 
                      : viewMode}
                </span>
                <ChevronDown className="w-3.5 h-3.5 text-text-secondary/60" />
              </button>
              
              {isViewDropdownOpen && (
                <>
                  <div 
                    className="fixed inset-0 z-40" 
                    onClick={() => setIsViewDropdownOpen(false)} 
                  />
                  <div className="absolute left-0 mt-1.5 w-52 bg-[#161616] border border-neutral-800 rounded-xl shadow-2xl py-1 z-50 animate-scale-in text-xs font-sans">
                    {dropdownSubmenu === 'none' && (
                      <div className="flex flex-col">
                        {[
                          { label: 'Day', mode: 'day', shortcut: '1 or D' },
                          { label: 'Week', mode: 'week', shortcut: '0 or W' },
                          { label: 'Month', mode: 'month', shortcut: 'M' },
                          { label: 'Agenda', mode: 'agenda', shortcut: 'A' },
                        ].map((item) => {
                          const isSelected = viewMode === item.mode;
                          return (
                            <button
                              key={item.mode}
                              onClick={() => {
                                setViewMode(item.mode as any);
                                setIsViewDropdownOpen(false);
                              }}
                              className={`w-full text-left px-3.5 py-2.5 transition-colors flex items-center justify-between cursor-pointer ${
                                isSelected 
                                  ? 'text-white bg-neutral-800/80 font-semibold' 
                                  : 'text-text-secondary hover:text-text-primary hover:bg-neutral-900'
                              }`}
                            >
                              <span>{item.label}</span>
                              <span className="text-[10px] font-mono text-text-secondary/40">{item.shortcut}</span>
                            </button>
                          );
                        })}
                        
                        <hr className="border-neutral-800/60 my-1" />
                        
                        <button
                          onClick={() => setDropdownSubmenu('days')}
                          className="w-full text-left px-3.5 py-2.5 text-text-secondary hover:text-text-primary hover:bg-neutral-900 transition-colors flex items-center justify-between cursor-pointer"
                        >
                          <span>Number of days</span>
                          <ChevronRight className="w-3.5 h-3.5 text-text-secondary/50" />
                        </button>

                        <button
                          onClick={() => setDropdownSubmenu('settings')}
                          className="w-full text-left px-3.5 py-2.5 text-text-secondary hover:text-text-primary hover:bg-neutral-900 transition-colors flex items-center justify-between cursor-pointer"
                        >
                          <span>View settings</span>
                          <ChevronRight className="w-3.5 h-3.5 text-text-secondary/50" />
                        </button>
                      </div>
                    )}

                    {dropdownSubmenu === 'days' && (
                      <div className="flex flex-col">
                        <button
                          onClick={() => setDropdownSubmenu('none')}
                          className="w-full text-left px-3 py-2 text-text-secondary hover:text-text-primary hover:bg-neutral-900 transition-colors flex items-center gap-2 cursor-pointer font-medium"
                        >
                          <ChevronLeft className="w-3.5 h-3.5" />
                          <span>Back</span>
                        </button>
                        
                        <hr className="border-neutral-800/60 my-1" />

                        {[
                          { label: '1 day', mode: 'day', shortcut: '1' },
                          { label: '2 days', mode: '2-day', shortcut: '2' },
                          { label: '3 days', mode: '3-day', shortcut: '3' },
                          { label: '4 days', mode: '4-day', shortcut: '4' },
                          { label: '5 days', mode: '5-day', shortcut: '5' },
                          { label: '6 days', mode: '6-day', shortcut: '6' },
                          { label: '7 days', mode: '7-day', shortcut: '7' },
                        ].map((item) => {
                          const isSelected = viewMode === item.mode || (item.mode === 'day' && viewMode === '1-day');
                          return (
                            <button
                              key={item.mode}
                              onClick={() => {
                                setViewMode(item.mode as any);
                                setIsViewDropdownOpen(false);
                              }}
                              className={`w-full text-left px-3.5 py-2.5 transition-colors flex items-center justify-between cursor-pointer ${
                                isSelected 
                                  ? 'text-white bg-neutral-800/80 font-semibold' 
                                  : 'text-text-secondary hover:text-text-primary hover:bg-neutral-900'
                              }`}
                            >
                              <span>{item.label}</span>
                              <span className="text-[10px] font-mono text-text-secondary/40">{item.shortcut}</span>
                            </button>
                          );
                        })}
                      </div>
                    )}

                    {dropdownSubmenu === 'settings' && (
                      <div className="flex flex-col">
                        <button
                          onClick={() => setDropdownSubmenu('none')}
                          className="w-full text-left px-3 py-2 text-text-secondary hover:text-text-primary hover:bg-neutral-900 transition-colors flex items-center gap-2 cursor-pointer font-medium"
                        >
                          <ChevronLeft className="w-3.5 h-3.5" />
                          <span>Back</span>
                        </button>
                        
                        <hr className="border-neutral-800/60 my-1" />

                        <button
                          onClick={() => {
                            const val = !showWeekends;
                            setShowWeekends(val);
                            localStorage.setItem('kestrel_cal_show_weekends', val.toString());
                            showToast(val ? 'Weekends visible' : 'Weekends hidden', 'info');
                          }}
                          className="w-full text-left px-3.5 py-2.5 text-text-secondary hover:text-text-primary hover:bg-neutral-900 transition-colors flex items-center justify-between cursor-pointer"
                        >
                          <span>Show weekends</span>
                          <div className={`w-3.5 h-3.5 rounded border border-neutral-700 flex items-center justify-center shrink-0 ${showWeekends ? 'bg-rose-500/20 border-rose-500/50' : ''}`}>
                            {showWeekends && <Check className="w-3 h-3 text-rose-400 stroke-[3]" />}
                          </div>
                        </button>

                        <button
                          onClick={() => {
                            setViewMode('weekdays');
                            setIsViewDropdownOpen(false);
                          }}
                          className={`w-full text-left px-3.5 py-2.5 transition-colors flex items-center justify-between cursor-pointer ${
                            viewMode === 'weekdays' 
                              ? 'text-white bg-neutral-800/80 font-semibold' 
                              : 'text-text-secondary hover:text-text-primary hover:bg-neutral-900'
                          }`}
                        >
                          <span>Weekdays Only</span>
                          {viewMode === 'weekdays' && <Check className="w-3.5 h-3.5 text-rose-400 stroke-[3]" />}
                        </button>

                        <button
                          onClick={() => {
                            setIsSettingsOpen(true);
                            setIsViewDropdownOpen(false);
                          }}
                          className="w-full text-left px-3.5 py-2.5 text-text-secondary hover:text-text-primary hover:bg-neutral-900 transition-colors flex items-center gap-2 cursor-pointer"
                        >
                          <Settings className="w-3.5 h-3.5 text-text-secondary/60" />
                          <span>All settings...</span>
                        </button>
                      </div>
                    )}
                  </div>
                </>
              )}
            </div>

            {/* Today Button */}
            <button 
              id="calendar-nav-today"
              onClick={handleToday}
              className="bg-canvas-card hover:bg-canvas-hover border border-border-hairline rounded-lg px-3 py-1.5 text-xs text-text-primary font-mono font-medium transition-all cursor-pointer"
            >
              Today
            </button>

            {/* Prev/Next arrows */}
            <div className="flex items-center bg-canvas-card border border-border-hairline rounded-lg p-0.5">
              <button 
                id="calendar-nav-prev"
                onClick={handlePrev}
                className="p-1 rounded-md hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
                title="Previous"
              >
                <ChevronLeft className="w-3.5 h-3.5" />
              </button>
              <button 
                id="calendar-nav-next"
                onClick={handleNext}
                className="p-1 rounded-md hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
                title="Next"
              >
                <ChevronRight className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Date Range Label */}
            <h2 id="calendar-header-label" className="text-sm font-medium text-text-primary font-mono tracking-tight ml-2">
              {getHeaderLabel()}
            </h2>

          </div>

        </div>
        )}

        {/* Dynamic Grid Space */}
        <div 
          id="calendar-grid-space" 
          className="flex-1 bg-canvas-base overflow-hidden flex flex-col"
          onTouchStart={handleTouchStart}
          onTouchEnd={handleTouchEnd}
        >
          
          {/* A. MONTH GRID */}
          {viewMode === 'month' && (
            <div className="flex-1 overflow-auto select-none">
              <div className="h-full flex flex-col min-w-[700px]">
              
              {/* Days label header */}
              <div className="grid grid-cols-7 border-b border-border-hairline bg-[#0d0d0d]/90 sticky top-0 z-10">
                {['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].map(day => (
                  <div key={day} className="py-2 text-center text-[10px] font-mono uppercase tracking-wider text-text-secondary/70 font-semibold border-r border-border-hairline last:border-r-0">
                    {day}
                  </div>
                ))}
              </div>

              {/* Grid block days */}
              <div className="grid grid-cols-7 grid-rows-6 flex-1 min-h-[550px]">
                {monthCells.map(cell => {
                  const isToday = cell.date === new Date().toISOString().split('T')[0];
                  const isSelectedDate = cell.date === currentDate.toISOString().split('T')[0];
                  const dayEvents = eventsByDate[cell.date] || [];
                  
                  return (
                    <div
                      key={cell.key}
                      id={`month-cell-${cell.date}`}
                      onClick={(e) => {
                        setCurrentDate(new Date(cell.date));
                        handleOpenCreateForm(cell.date, e);
                      }}
                      className={`group min-h-[95px] border-r border-b border-border-hairline p-1.5 flex flex-col transition-colors cursor-pointer relative hover:bg-canvas-hover/20 ${
                        cell.isCurrentMonth ? 'bg-transparent' : 'bg-[#0f0f0f]/30 opacity-40'
                      }`}
                    >
                      {/* Day Number and quick trigger */}
                      <div className="flex items-center justify-between mb-1">
                        <span 
                          className={`text-xs font-mono p-1 rounded-md flex items-center justify-center min-w-[22px] h-5.5 ${
                            isToday 
                              ? 'bg-white text-black font-semibold shadow' 
                              : isSelectedDate
                              ? 'border border-white/20 text-white font-medium bg-canvas-hover'
                              : 'text-text-primary/90'
                          }`}
                        >
                          {cell.dayNum}
                        </span>
                        
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            setCurrentDate(new Date(cell.date));
                            handleOpenCreateForm(cell.date, e);
                          }}
                          className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-all cursor-pointer"
                          title="Add Event"
                        >
                          <Plus className="w-3 h-3" />
                        </button>
                      </div>

                      {/* Events checklist list */}
                      <div className="flex-1 space-y-1 overflow-y-auto max-h-[110px] scrollbar-none">
                        {dayEvents.map(evt => {
                          const colOpt = COLOR_OPTIONS.find(c => c.name === evt.color) || COLOR_OPTIONS[0];
                          const isCurrentlySelected = selectedEventId === evt.id;
                          return (
                            <div
                              key={evt.id}
                              onClick={(e) => {
                                e.stopPropagation();
                                handleSelectEvent(evt, e);
                              }}
                              className={`px-2 py-0.5 rounded text-[10px] font-medium border transition-all truncate hover:translate-x-0.5 ${
                                isCurrentlySelected 
                                  ? 'ring-1 ring-white/40 border-white/30 font-semibold scale-[1.01]' 
                                  : ''
                              } ${colOpt.bg}`}
                              title={`${evt.startTime} - ${evt.title}`}
                            >
                              <span className="font-mono text-[9px] opacity-70 mr-1">{evt.startTime}</span>
                              <span>{evt.title}</span>
                            </div>
                          );
                        })}
                      </div>

                    </div>
                  );
                })}
              </div>

            </div>
            </div>
          )}

          {/* B. MULTI-DAY / DAY GRID (Day, 3-day, 4-day, weekdays, week views) */}
          {viewMode !== 'month' && viewMode !== 'agenda' && (
            <div className="flex-1 flex flex-col h-full overflow-x-auto overflow-y-hidden select-none">
              <div className={`flex-1 flex flex-col h-full ${isMobileOrTablet ? 'min-w-0 w-full' : 'min-w-[700px] md:min-w-0'}`}>
              {/* Sticky Week Headers */}
              <div className="grid grid-cols-[64px_1fr] border-b border-neutral-900/40 bg-canvas-base shrink-0">
                {/* Timezone label header */}
                <div className="flex items-center justify-center border-r border-neutral-900/40 text-[9px] font-mono text-text-secondary/50 p-2 text-center bg-canvas-base leading-none">
                  + {tzOffset}
                </div>
                
                {/* Columns Headers */}
                <div 
                  className="grid divide-x divide-neutral-900/40"
                  style={{ gridTemplateColumns: `repeat(${displayDays.length}, minmax(0, 1fr))` }}
                >
                  {displayDays.map(day => {
                    const dayEvents = eventsByDate[day.dateStr] || [];
                    const allDayEvents = dayEvents.filter(e => e.isAllDay);
                    return (
                      <div 
                        key={day.dateStr} 
                        onClick={(e) => {
                          setCurrentDate(new Date(day.dateStr));
                        }}
                        className={`p-2.5 flex flex-col justify-between transition-colors cursor-pointer hover:bg-neutral-900/30 min-h-[56px] ${
                          day.isToday ? 'bg-neutral-900/20' : ''
                        }`}
                      >
                        <div className="flex items-baseline gap-1.5 truncate">
                          <span className={`text-[10px] font-mono uppercase tracking-wider ${day.isToday ? 'text-white font-bold' : 'text-text-secondary/60'}`}>
                            {day.dayLabel}
                          </span>
                          <span className={`text-sm font-mono font-semibold ${day.isToday ? 'text-rose-400 font-bold' : 'text-text-primary'}`}>
                            {day.dayNum}
                          </span>
                        </div>
                        {/* Render All-Day Events if present */}
                        {allDayEvents.length > 0 && (
                          <div className="mt-1.5 space-y-1">
                            {allDayEvents.map(evt => {
                              const colOpt = COLOR_OPTIONS.find(c => c.name === evt.color) || COLOR_OPTIONS[0];
                              const isCurrentlySelected = selectedEventId === evt.id;
                              return (
                                <div
                                  key={evt.id}
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    handleSelectEvent(evt, e);
                                  }}
                                  className={`px-1.5 py-0.5 rounded text-[9px] font-medium transition-all truncate hover:translate-x-0.5 border ${
                                    isCurrentlySelected 
                                      ? 'ring-1 ring-white/45 border-white/30 font-semibold scale-[1.01]' 
                                      : 'border-transparent'
                                  } ${colOpt.bg} text-text-primary`}
                                  title={evt.title}
                                >
                                  {evt.title}
                                </div>
                              );
                            })}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Scrollable Hourly Container */}
              <div 
                ref={scrollContainerRef}
                className="flex-1 overflow-y-auto bg-[#0a0a0a] relative scrollbar-thin"
              >
                {/* Height is 24 * 60px = 1440px */}
                <div className="relative h-[1440px] grid grid-cols-[64px_1fr]">
                  
                  {/* Left Column: Time Labels */}
                  <div className="border-r border-neutral-900/40 bg-canvas-base/80 h-full relative">
                    {HOURS.map((hr, idx) => (
                      <div 
                        key={hr.hour24} 
                        className="absolute w-full pr-2 text-right text-[10px] font-mono text-text-secondary/45"
                        style={{ top: `${idx * 60 + 8}px` }}
                      >
                        {hr.label}
                      </div>
                    ))}
                  </div>

                  {/* Right: Grid columns & Background Row Lines */}
                  <div className="relative h-full">
                    {/* Horizontal grid background lines */}
                    {Array.from({ length: 24 }).map((_, idx) => (
                      <div 
                        key={idx} 
                        className="absolute left-0 right-0 border-b border-neutral-900/40 pointer-events-none"
                        style={{ top: `${idx * 60}px`, height: '60px' }}
                      />
                    ))}

                    {/* Columns of Days */}
                    <div 
                      className="absolute inset-0 grid divide-x divide-neutral-900/40"
                      style={{ gridTemplateColumns: `repeat(${displayDays.length}, minmax(0, 1fr))` }}
                    >
                      {displayDays.map(day => {
                        const dayEvents = eventsByDate[day.dateStr] || [];
                        const timedEvents = dayEvents.filter(e => !e.isAllDay && e.startTime);
                        const layouts = getTimedEventsLayout(timedEvents);

                        return (
                          <div 
                            key={day.dateStr}
                            onClick={(e) => handleColumnClick(day.dateStr, e)}
                            className="relative h-full cursor-cell hover:bg-neutral-900/5 transition-colors group"
                          >
                            {/* Render Event blocks */}
                            {timedEvents.map(evt => {
                              const layout = layouts[evt.id] || { left: 0, width: 100, start: 9, end: 10 };
                              const colOpt = COLOR_OPTIONS.find(c => c.name === evt.color) || COLOR_OPTIONS[0];
                              const isCurrentlySelected = selectedEventId === evt.id;
                              
                              return (
                                <div
                                  key={evt.id}
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    handleSelectEvent(evt, e);
                                  }}
                                  className={`absolute rounded px-2 py-1 shadow-md cursor-pointer select-none transition-all hover:scale-[1.01] hover:shadow-lg overflow-hidden flex flex-col justify-between ${
                                    isCurrentlySelected 
                                      ? 'ring-2 ring-white/45 border-white/35 font-semibold z-10' 
                                      : 'z-0 border-l-[3px]'
                                  } ${colOpt.bg}`}
                                  style={{
                                    top: `${layout.start * 60 + 2}px`,
                                    height: `${Math.max(26, (layout.end - layout.start) * 60 - 4)}px`,
                                    left: `${layout.left}%`,
                                    width: `${layout.width - 1}%`,
                                  }}
                                  title={`${evt.startTime} - ${evt.endTime}: ${evt.title}`}
                                >
                                  <div className="truncate">
                                    <h5 className="text-[10px] font-bold text-text-primary leading-tight truncate">
                                      {evt.title}
                                    </h5>
                                    {Math.abs(layout.end - layout.start) >= 0.7 && evt.location && (
                                      <p className="text-[9px] text-text-secondary/80 truncate flex items-center gap-0.5 mt-0.5">
                                        <MapPin className="w-2.5 h-2.5 shrink-0" />
                                        {evt.location}
                                      </p>
                                    )}
                                  </div>
                                  {Math.abs(layout.end - layout.start) >= 0.6 && (
                                    <span className="text-[8px] font-mono opacity-80 mt-auto leading-none self-start bg-black/10 px-1 py-0.5 rounded">
                                      {evt.startTime}
                                    </span>
                                  )}
                                </div>
                              );
                            })}
                          </div>
                        );
                      })}
                    </div>

                  </div>

                </div>
              </div>
            </div>
            </div>
          )}

          {/* C. AGENDA VIEW */}
          {viewMode === 'agenda' && (
            <div className="flex-1 flex flex-col h-full bg-[#0a0a0a] overflow-y-auto p-6 space-y-6 select-none scrollbar-thin">
              <div className="max-w-3xl mx-auto w-full space-y-6 animate-scale-in">
                <div className="flex items-center justify-between border-b border-neutral-800/40 pb-3">
                  <h3 className="text-xs font-semibold text-text-primary tracking-wider uppercase font-mono">
                    Upcoming Agenda
                  </h3>
                  <span className="text-xs font-mono text-text-secondary/50">
                    {filteredEvents.length} events found
                  </span>
                </div>
                
                {filteredEvents.length === 0 ? (
                  <div className="text-center py-16 border border-dashed border-neutral-800/60 rounded-2xl bg-canvas-card/50">
                    <CalendarIcon className="w-10 h-10 text-text-secondary/30 mx-auto mb-3" />
                    <p className="text-xs text-text-secondary/50 font-mono">No matching calendar events found.</p>
                  </div>
                ) : (
                  <div className="space-y-6 relative border-l border-neutral-800/60 pl-4 ml-2">
                    {(() => {
                      // Group filteredEvents by date, sorted chronologically
                      const sortedEvents = [...filteredEvents].sort((a, b) => {
                        const dateDiff = a.date.localeCompare(b.date);
                        if (dateDiff !== 0) return dateDiff;
                        return a.startTime.localeCompare(b.startTime);
                      });

                      const groups: { [date: string]: CalendarEvent[] } = {};
                      sortedEvents.forEach(evt => {
                        if (!groups[evt.date]) groups[evt.date] = [];
                        groups[evt.date].push(evt);
                      });

                      return Object.keys(groups).map(dateStr => {
                        const d = new Date(dateStr);
                        const isTodayStr = dateStr === new Date().toISOString().split('T')[0];
                        const dayLabel = d.toLocaleDateString(undefined, { weekday: 'long', month: 'short', day: 'numeric', year: 'numeric' });
                        
                        return (
                          <div key={dateStr} className="relative space-y-3">
                            {/* Dot indicator on timeline line */}
                            <div className={`absolute -left-[21px] top-1.5 w-2 h-2 rounded-full border bg-[#0a0a0a] ${
                              isTodayStr ? 'border-rose-400 ring-4 ring-rose-400/20' : 'border-neutral-800'
                            }`} />
                            
                            <h4 className={`text-xs font-mono font-semibold tracking-wide ${
                              isTodayStr ? 'text-rose-400' : 'text-text-primary'
                            }`}>
                              {dayLabel} {isTodayStr && <span className="ml-1.5 text-[8px] bg-rose-500/20 text-rose-300 px-1.5 py-0.2 rounded uppercase font-bold">Today</span>}
                            </h4>
                            
                            <div className="space-y-2">
                              {groups[dateStr].map(evt => {
                                const colOpt = COLOR_OPTIONS.find(c => c.name === evt.color) || COLOR_OPTIONS[0];
                                const isCurrentlySelected = selectedEventId === evt.id;
                                
                                return (
                                  <div
                                    key={evt.id}
                                    onClick={(e) => handleSelectEvent(evt, e)}
                                    className={`group flex flex-col md:flex-row md:items-start justify-between p-3 rounded-xl border border-neutral-900 bg-canvas-card/40 hover:bg-canvas-hover/40 transition-all cursor-pointer relative gap-2 ${
                                      isCurrentlySelected ? 'ring-1 ring-white/30 border-transparent bg-canvas-hover/60' : ''
                                    }`}
                                  >
                                    <div className="space-y-1.5 flex-1 min-w-0">
                                      <div className="flex items-center gap-2 flex-wrap">
                                        <div className={`w-2 h-2 rounded-full ${colOpt.dot}`} />
                                        <h5 className="text-xs font-semibold text-text-primary truncate">
                                          {evt.title}
                                        </h5>
                                        {evt.isAllDay && (
                                          <span className="text-[9px] bg-neutral-800 text-text-secondary px-1.5 py-0.5 rounded uppercase font-mono">
                                            All Day
                                          </span>
                                        )}
                                      </div>
                                      
                                      {evt.description && (
                                        <p className="text-xs text-text-secondary/80 whitespace-pre-wrap line-clamp-3">
                                          {evt.description}
                                        </p>
                                      )}
                                      
                                      {evt.location && (
                                        <div className="text-[10px] text-text-secondary/60 flex items-center gap-1">
                                          <MapPin className="w-3.5 h-3.5 text-text-secondary/40 shrink-0" />
                                          <span className="truncate">{evt.location}</span>
                                        </div>
                                      )}
                                    </div>
                                    
                                    <div className="flex items-center gap-2 md:flex-col md:items-end md:justify-center shrink-0">
                                      {!evt.isAllDay && (
                                        <span className="text-[10px] font-mono text-text-secondary bg-[#1a1a1a]/80 px-2 py-0.5 rounded border border-neutral-800/60">
                                          {evt.startTime} - {evt.endTime}
                                        </span>
                                      )}
                                    </div>
                                  </div>
                                );
                              })}
                            </div>
                          </div>
                        );
                      });
                    })()}
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Old Day Grid removed as it is handled by Multi-Day grid */}

        </div>

      </main>

      {/* 3. RIGHT PANEL (Event View & Dynamic Inline Editing Form) */}
      {selectedEvent || isEditingEvent ? (
        <>
          {/* Mobile size backdrop overlay */}
          {isMobileOrTablet && (
            <div 
              className="fixed inset-0 bg-black/60 backdrop-blur-xs z-40 animate-fade-in"
              onClick={() => {
                setSelectedEventId(null);
                setIsEditingEvent(false);
              }}
            />
          )}
          <section id="calendar-right-sidebar" {...getPanelContainerProps()}>
            {isMobileOrTablet && (
              <div className="w-12 h-1 bg-neutral-800 rounded-full mx-auto mt-2.5 mb-1 shrink-0" />
            )}
          
          {/* Top Panel Actions bar */}
          <div className="px-4 py-3 border-b border-neutral-800/60 bg-[#131313] flex items-center justify-between shrink-0">
            
            <div className="flex items-center gap-2 text-xs font-bold text-text-primary">
              <span>{isEditingEvent ? (selectedEventId ? 'Edit Event' : 'Create Event') : 'Event'}</span>
            </div>

            <div className="flex items-center gap-1.5">
              
              {/* Layout Toggle Button */}
              <button
                type="button"
                onClick={() => setIsSidebarMode(!isSidebarMode)}
                className={`p-1.5 rounded transition-colors cursor-pointer ${
                  isSidebarMode 
                    ? 'bg-neutral-800 text-white' 
                    : 'text-text-secondary hover:text-text-primary hover:bg-neutral-800'
                }`}
                title={isSidebarMode ? "Change to Floating Panel" : "Dock to Side Panel"}
              >
                <Columns className="w-3.5 h-3.5" />
              </button>
              
              {/* Edit toggle button (Only visible if we have a selectedEvent and are not already editing) */}
              {selectedEvent && !isEditingEvent && (
                <button
                  type="button"
                  onClick={() => setIsEditingEvent(true)}
                  className="p-1.5 rounded hover:bg-neutral-800 text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
                  title="Edit Event"
                >
                  <Edit2 className="w-3.5 h-3.5" />
                </button>
              )}

              {/* Delete button (slight red background/outline as requested) */}
              {selectedEvent && (
                <button
                  type="button"
                  onClick={() => handleDeleteEvent(selectedEvent.id)}
                  className="p-1.5 rounded hover:bg-red-500/10 text-red-400 hover:text-red-300 transition-colors cursor-pointer"
                  title="Delete Event"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              )}

              {/* Close panel button */}
              <button
                type="button"
                onClick={() => {
                  setSelectedEventId(null);
                  setIsEditingEvent(false);
                }}
                className="p-1.5 rounded hover:bg-neutral-800 text-text-secondary hover:text-text-primary transition-colors cursor-pointer ml-0.5"
                title="Close"
              >
                <X className="w-3.5 h-3.5" />
              </button>

            </div>
          </div>

          {/* Panel Content Body */}
          <div className="flex-1 overflow-y-auto px-5 py-5 space-y-5 scrollbar-none">
            
            {isEditingEvent ? (
              /* DYNAMIC EDIT FORM (Inline Editor) */
              <form onSubmit={handleSaveForm} className="space-y-4">
                
                {/* Event Title field - Frameless Direct Input */}
                <div className="pb-1 border-b border-neutral-800/30">
                  <input
                    type="text"
                    placeholder="Event Title"
                    value={formTitle}
                    onChange={(e) => setFormTitle(e.target.value)}
                    className="w-full bg-transparent text-lg font-bold text-text-primary outline-none border-none placeholder-text-secondary/35 p-0"
                    required
                    autoFocus
                  />
                </div>

                {/* Date Selection */}
                <div className="space-y-1">
                  <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">Date</label>
                  <input
                    type="date"
                    value={formDate}
                    onChange={(e) => setFormDate(e.target.value)}
                    className="w-full bg-neutral-900/40 rounded-xl px-3 py-2 text-xs text-text-primary font-mono outline-none focus:bg-neutral-900/70 transition-all"
                    required
                  />
                </div>

                {/* All-Day Toggle */}
                <div className="flex items-center justify-between py-1 bg-neutral-900/25 px-2.5 rounded-xl">
                  <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/70">All-Day Event</span>
                  <button
                    type="button"
                    onClick={() => setFormIsAllDay(!formIsAllDay)}
                    className="text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
                  >
                    {formIsAllDay ? (
                      <ToggleRight className="w-7 h-7 text-emerald-400 stroke-[1.5]" />
                    ) : (
                      <ToggleLeft className="w-7 h-7 text-text-secondary/40 stroke-[1.5]" />
                    )}
                  </button>
                </div>

                {/* Start / End Times (only shown if not all-day) */}
                {!formIsAllDay && (
                  <div className="grid grid-cols-2 gap-2">
                    <div className="space-y-1">
                      <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">Start Time</label>
                      <input
                        type="time"
                        value={formStartTime}
                        onChange={(e) => setFormStartTime(e.target.value)}
                        className="w-full bg-neutral-900/40 rounded-xl px-2.5 py-2 text-xs text-text-primary font-mono outline-none focus:bg-neutral-900/70 transition-all"
                        required
                      />
                    </div>
                    <div className="space-y-1">
                      <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">End Time</label>
                      <input
                        type="time"
                        value={formEndTime}
                        onChange={(e) => setFormEndTime(e.target.value)}
                        className="w-full bg-neutral-900/40 rounded-xl px-2.5 py-2 text-xs text-text-primary font-mono outline-none focus:bg-neutral-900/70 transition-all"
                        required
                      />
                    </div>
                  </div>
                )}

                {/* Calendar Selector (Ties to accounts!) */}
                <div className="space-y-1">
                  <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">Target Calendar</label>
                  <select
                    value={formCalendarId}
                    onChange={(e) => {
                      const calId = e.target.value;
                      setFormCalendarId(calId);
                      // Auto update event color based on target calendar's defined color
                      const calConfig = calendarsMap[calId];
                      if (calConfig) {
                        setFormColor(calConfig.color);
                      }
                    }}
                    className="w-full bg-neutral-900/40 rounded-xl px-2.5 py-2 text-xs text-text-primary outline-none focus:bg-neutral-900/70 transition-all"
                  >
                    {accounts.map(acc => (
                      <optgroup key={acc.id} label={acc.email} className="bg-canvas-card text-text-secondary font-mono text-[10px]">
                        {acc.calendars.map(cal => (
                          <option key={cal.id} value={cal.id} className="text-text-primary font-sans text-xs">
                            {cal.name}
                          </option>
                        ))}
                      </optgroup>
                    ))}
                  </select>
                </div>

                {/* Status & Priority Row */}
                <div className="grid grid-cols-2 gap-2">
                  <div className="space-y-1">
                    <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">Status</label>
                    <select
                      value={formStatus}
                      onChange={(e) => setFormStatus(e.target.value as any)}
                      className="w-full bg-neutral-900/40 rounded-xl px-2.5 py-2 text-xs text-text-primary outline-none focus:bg-neutral-900/70 transition-all"
                    >
                      {STATUS_OPTIONS.map(opt => (
                        <option key={opt} value={opt}>{opt}</option>
                      ))}
                    </select>
                  </div>
                  <div className="space-y-1">
                    <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">Priority</label>
                    <select
                      value={formPriority}
                      onChange={(e) => setFormPriority(e.target.value as any)}
                      className="w-full bg-neutral-900/40 rounded-xl px-2.5 py-2 text-xs text-text-primary outline-none focus:bg-neutral-900/70 transition-all"
                    >
                      {PRIORITY_OPTIONS.map(opt => (
                        <option key={opt} value={opt}>{opt}</option>
                      ))}
                    </select>
                  </div>
                </div>

                {/* Location Input */}
                <div className="space-y-1">
                  <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">Location</label>
                  <div className="relative">
                    <MapPin className="w-3.5 h-3.5 text-text-secondary/50 absolute left-3 top-2.5" />
                    <input
                      type="text"
                      placeholder="Google Meet, Conference Room A"
                      value={formLocation}
                      onChange={(e) => setFormLocation(e.target.value)}
                      className="w-full bg-neutral-900/40 rounded-xl pl-9 pr-3 py-2 text-xs text-text-primary outline-none focus:bg-neutral-900/70 transition-all"
                    />
                  </div>
                </div>

                {/* Description Input */}
                <div className="space-y-1">
                  <label className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/60 block">Notes & Agenda</label>
                  <textarea
                    placeholder="Agenda details, notes, links..."
                    value={formDescription}
                    onChange={(e) => setFormDescription(e.target.value)}
                    rows={4}
                    className="w-full bg-neutral-900/40 rounded-xl px-3 py-2 text-xs text-text-primary outline-none focus:bg-neutral-900/70 transition-all resize-none leading-relaxed"
                  />
                </div>

                {/* Save and Cancel triggers */}
                <div className="flex gap-2 pt-2">
                  <button
                    type="button"
                    onClick={() => {
                      if (selectedEvent) {
                        setIsEditingEvent(false);
                      } else {
                        setSelectedEventId(null);
                        setIsEditingEvent(false);
                      }
                    }}
                    className="flex-1 px-3 py-2 rounded-xl text-xs font-semibold bg-neutral-900/60 hover:bg-neutral-800 text-text-secondary hover:text-text-primary transition-colors cursor-pointer text-center"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className="flex-1 px-3 py-2 rounded-xl text-xs font-semibold bg-white hover:bg-neutral-200 text-black shadow transition-colors cursor-pointer text-center"
                  >
                    Save
                  </button>
                </div>

              </form>
            ) : (
              /* VIEW MODE DETAILS VIEW (Matches Image 3) */
              selectedEvent && (() => {
                // Extract any link from the description
                const linkRegex = /(https?:\/\/[^\s]+)/g;
                const matches = selectedEvent.description?.match(linkRegex);
                const extractedLink = matches ? matches[0] : null;
                // Remove the URL text from description for cleaner display
                const cleanDescription = selectedEvent.description
                  ? selectedEvent.description.replace(linkRegex, '').trim()
                  : '';
                const calInfo = calendarsMap[selectedEvent.calendarId];
                const colOpt = COLOR_OPTIONS.find(c => c.name === selectedEvent.color) || COLOR_OPTIONS[0];

                return (
                  <div className="space-y-5">
                    
                    {/* Event Title Block */}
                    <div className="space-y-1">
                      <h3 className="text-sm font-bold text-text-primary leading-snug">
                        {selectedEvent.title}
                      </h3>
                    </div>

                    {/* Planned Execution Date Block - BORDERLESS */}
                    <div className="space-y-2.5 bg-neutral-900/30 p-3.5 rounded-xl">
                      
                      <div className="flex items-start gap-2.5">
                        <Clock className="w-3.5 h-3.5 text-neutral-400 mt-0.5 flex-shrink-0" />
                        <div className="space-y-0.5">
                          <div className="text-xs text-text-primary font-bold">
                            {selectedEvent.isAllDay ? 'All Day' : `${selectedEvent.startTime} → ${selectedEvent.endTime}`}
                            {!selectedEvent.isAllDay && (
                              <span className="text-[10px] text-text-secondary/70 font-normal font-mono ml-2">
                                (()20 min(()
                              </span>
                            )}
                          </div>
                          <div className="text-[11px] text-text-secondary font-mono">
                            {new Date(selectedEvent.date).toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric', year: 'numeric' })}
                          </div>
                        </div>
                      </div>

                      {/* Propose New Time Button */}
                      <button
                        type="button"
                        onClick={() => showToast('Opening slot proposer tools in Workspace...', 'info')}
                        className="w-full mt-2 py-1.5 px-2.5 rounded-lg bg-[#1a1a1a]/60 hover:bg-neutral-800/80 text-[10px] font-semibold text-neutral-200 transition-colors cursor-pointer flex items-center justify-between"
                      >
                        <span>Propose new time</span>
                        <ExternalLink className="w-3 h-3 text-neutral-400" />
                      </button>

                      {/* Timezone details */}
                      <div className="flex items-center justify-between mt-2 pt-2 border-t border-neutral-800/20 text-[9px] text-text-secondary/70 font-mono">
                        <div className="flex items-center gap-1.5">
                          <Globe className="w-3.5 h-3.5 text-neutral-500" />
                          <span>Calcutta Time (GMT+5:30)</span>
                        </div>
                        <CornerUpLeft className="w-3 h-3 text-neutral-500" />
                      </div>

                    </div>

                    {/* Organizer & Attendees block */}
                    {(selectedEvent.organizer || (selectedEvent.attendees && selectedEvent.attendees.length > 0)) && (
                      <div className="space-y-2.5 pt-1">
                        
                        {/* Organizer */}
                        {selectedEvent.organizer && (
                          <div className="space-y-1">
                            <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/50">Organizer</span>
                            <div className="flex items-center gap-2 bg-neutral-900/30 p-2.5 rounded-xl">
                              <div className="w-7 h-7 rounded-full bg-neutral-800 text-neutral-300 flex items-center justify-center text-xs font-bold font-mono">
                                {selectedEvent.organizer.charAt(0).toUpperCase()}
                              </div>
                              <div className="flex-1 min-w-0">
                                <div className="text-xs text-text-primary font-semibold truncate">{selectedEvent.organizer}</div>
                                <div className="text-[10px] text-text-secondary/60 font-mono">Organizer</div>
                              </div>
                            </div>
                          </div>
                        )}

                        {/* Attendees List */}
                        {selectedEvent.attendees && selectedEvent.attendees.length > 0 && (
                          <div className="space-y-1">
                            <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/50">Attendees</span>
                            <div className="space-y-1.5">
                              {selectedEvent.attendees.map((att, idx) => (
                                <div key={idx} className="flex items-center gap-2 bg-neutral-900/30 p-2.5 rounded-xl">
                                  <div className="relative">
                                    <div className="w-7 h-7 rounded-full bg-emerald-800/20 border border-emerald-500/10 text-emerald-400 flex items-center justify-center text-xs font-bold font-mono">
                                      {att.name.split(' ').map(n => n[0]).join('').toUpperCase()}
                                    </div>
                                    {att.rsvp === 'yes' && (
                                      <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-emerald-500 rounded-full border border-[#131313] flex items-center justify-center">
                                        <Check className="w-2 h-2 text-white stroke-[3.5]" />
                                      </div>
                                    )}
                                  </div>
                                  <div className="flex-1 min-w-0">
                                    <div className="text-xs text-text-primary font-semibold truncate">{att.name}</div>
                                    <div className="text-[10px] text-text-secondary/60 font-mono truncate">{att.email}</div>
                                  </div>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}

                        {/* Interactive RSVP Picker Pill Container */}
                        {selectedEvent.rsvpStatus && (
                          <div className="space-y-1">
                            <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/50">Your RSVP Status</span>
                            <div className="flex items-center gap-1.5 bg-[#1a1a1a]/60 p-1.5 rounded-xl">
                              <button
                                type="button"
                                onClick={() => handleUpdateRSVP(selectedEvent.id, 'yes')}
                                className={`flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer ${
                                  selectedEvent.rsvpStatus === 'yes'
                                    ? 'bg-emerald-500/15 text-emerald-400 font-bold'
                                    : 'text-text-secondary hover:text-text-primary'
                                }`}
                              >
                                Yes
                              </button>
                              <button
                                type="button"
                                onClick={() => handleUpdateRSVP(selectedEvent.id, 'no')}
                                className={`flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer ${
                                  selectedEvent.rsvpStatus === 'no'
                                    ? 'bg-rose-500/15 text-rose-400 font-bold'
                                    : 'text-text-secondary hover:text-text-primary'
                                }`}
                              >
                                No
                              </button>
                              <button
                                type="button"
                                onClick={() => handleUpdateRSVP(selectedEvent.id, 'maybe')}
                                className={`flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer ${
                                  selectedEvent.rsvpStatus === 'maybe'
                                    ? 'bg-amber-500/15 text-amber-400 font-bold'
                                    : 'text-text-secondary hover:text-text-primary'
                                }`}
                              >
                                Maybe
                              </button>
                              <div className="w-px h-5 bg-neutral-800 mx-1" />
                              <button
                                type="button"
                                onClick={() => setIsEditingEvent(true)}
                                className="p-1.5 rounded-lg text-text-secondary hover:text-text-primary hover:bg-neutral-800 transition-colors cursor-pointer"
                                title="Edit Response"
                              >
                                <Edit2 className="w-3 h-3" />
                              </button>
                            </div>
                          </div>
                        )}

                      </div>
                    )}

                    {/* AI notes and online meeting shortcuts */}
                    <div className="space-y-2 pt-3 border-t border-neutral-800/20">
                      
                      <button
                        type="button"
                        onClick={() => showToast('AI is fetching and compiling action items...', 'info')}
                        className="w-full py-2 px-3 rounded-xl bg-gradient-to-r from-purple-500/5 to-indigo-500/5 hover:from-purple-500/10 hover:to-indigo-500/10 text-xs font-semibold text-purple-300 transition-all cursor-pointer flex items-center justify-between"
                      >
                        <div className="flex items-center gap-2">
                          <Sparkles className="w-3.5 h-3.5 text-purple-400 animate-pulse" />
                          <span>Add AI meeting notes</span>
                        </div>
                        <ChevronRight className="w-3.5 h-3.5 text-purple-400/70" />
                      </button>

                      {extractedLink && (
                        <div className="space-y-1">
                          <span className="text-[10px] font-mono uppercase tracking-wider text-text-secondary/50 block">Interview Link</span>
                          <a 
                            href={extractedLink} 
                            target="_blank" 
                            rel="noopener noreferrer" 
                            className="flex items-center justify-between p-2.5 px-3 rounded-xl bg-blue-500/10 hover:bg-blue-500/15 transition-all text-xs font-semibold text-blue-400"
                          >
                            <div className="flex items-center gap-2 truncate">
                              <Video className="w-3.5 h-3.5 text-blue-400 flex-shrink-0" />
                              <span className="truncate text-blue-400/90 font-mono text-[11px]">{extractedLink}</span>
                            </div>
                            <ExternalLink className="w-3.5 h-3.5 text-blue-400 flex-shrink-0 ml-2" />
                          </a>
                        </div>
                      )}

                    </div>

                    {/* Calendar & Interactive Provider details */}
                    {calInfo && (
                      <div className="space-y-3 pt-3 border-t border-neutral-800/20">
                        
                        {/* Swatch & Email */}
                        <div className="space-y-1.5">
                          <div className="flex items-center gap-2.5">
                            <div 
                              className="w-3.5 h-3.5 rounded border border-white/10 flex-shrink-0" 
                              style={{ backgroundColor: colOpt.hex }}
                            />
                            <span className="text-xs font-mono text-text-primary font-medium select-text break-all">
                              {calInfo.email}
                            </span>
                          </div>
                          
                          {/* Busy / Default visibility */}
                          <div className="flex items-center gap-6 text-[10px] font-mono text-text-secondary/70 pl-6">
                            <span>Busy</span>
                            <span>Default visibility</span>
                          </div>
                        </div>

                        {/* Reminders section with icon */}
                        <div className="space-y-1.5">
                          <div className="flex items-center gap-2 text-text-secondary/80">
                            <Bell className="w-3.5 h-3.5 text-text-secondary/60" />
                            <span className="text-xs font-mono font-medium">Reminders</span>
                          </div>
                          <div className="space-y-1 pl-6 font-mono text-[10px] text-text-secondary/70">
                            <div>30 min before</div>
                            <div>5 min before</div>
                          </div>
                        </div>

                      </div>
                    )}

                    {/* Other attributes (Priority, Location, etc.) */}
                    <div className="space-y-3 pt-3 border-t border-neutral-800/20">
                      
                      {/* Priority Field */}
                      <div className="flex items-center justify-between text-xs py-1.5">
                        <div className="flex items-center gap-1 text-text-secondary font-mono text-[10px]">
                          <Flag className="w-3.5 h-3.5 text-text-secondary/60" />
                          <span>Priority</span>
                        </div>
                        {selectedEvent.priority && selectedEvent.priority !== 'None' ? (
                          <span className={`px-2 py-0.5 rounded-full text-[9px] font-mono font-semibold ${
                            selectedEvent.priority === 'High' 
                              ? 'bg-rose-500/15 text-rose-400' 
                              : selectedEvent.priority === 'Medium'
                              ? 'bg-amber-500/15 text-amber-400'
                              : 'bg-blue-500/15 text-blue-400'
                          }`}>
                            {selectedEvent.priority}
                          </span>
                        ) : (
                          <span className="text-text-secondary/55 font-mono text-[10px]">None</span>
                        )}
                      </div>

                      {/* Location Field */}
                      {selectedEvent.location && (
                        <div className="space-y-1">
                          <div className="flex items-center gap-1 text-text-secondary font-mono text-[10px]">
                            <MapPin className="w-3.5 h-3.5 text-text-secondary/60" />
                            <span>Location</span>
                          </div>
                          <div className="text-xs text-text-primary bg-neutral-900/30 rounded-lg p-2.5 font-medium">
                            {selectedEvent.location}
                          </div>
                        </div>
                      )}

                    </div>

                    {/* Description / Notes text */}
                    {cleanDescription && (
                      <div className="space-y-1.5 pt-3 border-t border-neutral-800/20">
                        <div className="flex items-center gap-1 text-text-secondary font-mono text-[10px]">
                          <AlignLeft className="w-3.5 h-3.5 text-text-secondary/60" />
                          <span>Notes & Description</span>
                        </div>
                        <p className="text-xs text-text-secondary/90 leading-relaxed bg-neutral-900/20 rounded-xl p-3.5 max-h-40 overflow-y-auto whitespace-pre-line">
                          {cleanDescription}
                        </p>
                      </div>
                    )}

                    {/* CTA Action button (Manage in Workspace) */}
                    <div className="pt-2">
                      <button
                        type="button"
                        onClick={() => showToast('Redirecting to Google Workspace sync services...', 'info')}
                        className="w-full py-2 rounded-lg bg-neutral-900/80 hover:bg-neutral-800/80 text-xs font-semibold text-text-primary hover:text-white transition-colors cursor-pointer flex items-center justify-center gap-2"
                      >
                        <ExternalLink className="w-3.5 h-3.5" />
                        <span>Manage in Workspace</span>
                      </button>
                    </div>

                  </div>
                );
              })()
            )}

          </div>

        </section>
        </>
      ) : isSidebarMode ? (
        /* Empty side panel prompt on Desktop - ONLY when docked/sidebar mode is true */
        <aside id="calendar-right-sidebar-empty" className="hidden lg:flex w-80 bg-[#0d0d0d] border-l border-border-hairline flex-col h-full items-center justify-center p-6 text-center select-none shrink-0 relative z-40">
          <Info className="w-6 h-6 text-text-secondary/30 mb-2" />
          <p className="text-[11px] font-mono text-text-secondary/40">
            Select any event to view plan execution and status details.
          </p>
        </aside>
      ) : null}

      {/* 4. CALENDAR SETTINGS MODAL OVERLAY */}
      {isSettingsOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs animate-fade-in">
          <div className="fixed inset-0 cursor-pointer" onClick={() => setIsSettingsOpen(false)} />
          
          <div className="relative w-full max-w-md bg-[#131313] border border-neutral-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden animate-scale-in z-10">
            {/* Modal Header */}
            <div className="px-5 py-4 border-b border-neutral-800/60 flex items-center justify-between bg-[#181818]">
              <div className="flex items-center gap-2">
                <Settings className="w-4 h-4 text-rose-400 animate-spin-slow" />
                <span className="text-xs font-mono font-semibold text-text-primary uppercase tracking-wider">
                  Calendar Settings
                </span>
              </div>
              <button 
                onClick={() => setIsSettingsOpen(false)}
                className="p-1 rounded-md hover:bg-neutral-800 text-text-secondary hover:text-text-primary transition-all cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Modal Body */}
            <div className="p-5 space-y-6 overflow-y-auto max-h-[80vh] scrollbar-thin">
              
              {/* Option A: Default Calendar */}
              <div className="space-y-2.5">
                <label className="block text-[10px] font-mono text-text-secondary/60 uppercase tracking-wider">
                  Default Calendar
                </label>
                <div className="space-y-2">
                  {[
                    { id: 'cal-personal', name: 'Personal' },
                    { id: 'cal-work', name: 'Work' },
                    { id: 'cal-birthdays', name: 'Birthdays' }
                  ].map(cal => {
                    const isSelected = defaultCalendarId === cal.id;
                    return (
                      <button
                        key={cal.id}
                        onClick={() => {
                          setDefaultCalendarId(cal.id);
                          localStorage.setItem('kestrel_default_cal_id', cal.id);
                          showToast(`Default calendar changed to: ${cal.name}`, 'info');
                        }}
                        className={`w-full flex items-center justify-between p-3 rounded-xl border text-xs transition-all cursor-pointer ${
                          isSelected 
                            ? 'bg-[#1a1a1a] border-rose-500/50 text-text-primary font-medium' 
                            : 'bg-[#101010]/50 border-neutral-800/40 text-text-secondary hover:text-text-primary hover:bg-[#151515]'
                        }`}
                      >
                        <span>{cal.name}</span>
                        <div className={`w-4 h-4 rounded-full border flex items-center justify-center shrink-0 ${
                          isSelected ? 'border-rose-400' : 'border-neutral-700'
                        }`}>
                          {isSelected && <div className="w-2 h-2 rounded-full bg-rose-400" />}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Option B: Daily Start Hour */}
              <div className="space-y-2.5">
                <label className="block text-[10px] font-mono text-text-secondary/60 uppercase tracking-wider">
                  Scroll Daily Start Hour
                </label>
                <div className="relative">
                  <select
                    value={startHour}
                    onChange={(e) => {
                      const val = Number(e.target.value);
                      setStartHour(val);
                      localStorage.setItem('kestrel_cal_start_hour', val.toString());
                      showToast(`Start hour adjusted to ${val % 12 === 0 ? 12 : val % 12} ${val >= 12 ? 'PM' : 'AM'}`, 'info');
                    }}
                    className="w-full bg-[#101010] border border-neutral-800 hover:border-neutral-700/80 rounded-xl px-3.5 py-2.5 text-xs text-text-primary outline-none focus:border-rose-500/50 transition-all cursor-pointer font-mono"
                  >
                    {Array.from({ length: 24 }).map((_, i) => {
                      const label = i === 0 ? '12:00 AM (Midnight)' : i === 12 ? '12:00 PM (Noon)' : i > 12 ? `${i - 12}:00 PM` : `${i}:00 AM`;
                      return (
                        <option key={i} value={i}>
                          {label}
                        </option>
                      );
                    })}
                  </select>
                </div>
                <p className="text-[10px] font-mono text-text-secondary/40 leading-relaxed">
                  Configures the default vertical grid position when loading or switching views.
                </p>
              </div>

              {/* Option C: Show Weekends Toggle */}
              <div className="flex items-center justify-between p-3 rounded-xl bg-[#101010]/30 border border-neutral-800/40">
                <div className="space-y-1">
                  <span className="block text-xs font-semibold text-text-primary">
                    Show Weekends
                  </span>
                  <span className="block text-[10px] font-mono text-text-secondary/40">
                    Include Saturday & Sunday in grids.
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => {
                    const val = !showWeekends;
                    showWeekends;
                    setShowWeekends(val);
                    localStorage.setItem('kestrel_cal_show_weekends', val.toString());
                    showToast(val ? 'Weekends visible' : 'Weekends hidden', 'info');
                  }}
                  className="text-text-primary hover:text-white transition-colors cursor-pointer"
                >
                  {showWeekends ? (
                    <ToggleRight className="w-8 h-8 text-rose-400" />
                  ) : (
                    <ToggleLeft className="w-8 h-8 text-text-secondary/40" />
                  )}
                </button>
              </div>

            </div>

            {/* Modal Footer */}
            <div className="px-5 py-3 border-t border-neutral-800/60 bg-[#181818] flex items-center justify-end">
              <button
                onClick={() => setIsSettingsOpen(false)}
                className="w-full py-2 rounded-xl bg-white hover:bg-neutral-200 text-black text-xs font-semibold shadow transition-all cursor-pointer"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Floating Add Event Button on Mobile/Tablet */}
      {isMobileOrTablet && (
        <button
          onClick={(e) => handleOpenCreateForm(undefined, e)}
          className="fixed bottom-6 right-6 z-30 w-12 h-12 rounded-full bg-white text-black flex items-center justify-center shadow-2xl hover:bg-neutral-200 active:scale-95 transition-all cursor-pointer border border-white/10"
          title="Add Event"
        >
          <Plus className="w-6 h-6 stroke-[2.5]" />
        </button>
      )}

    </div>
  );
}
