export interface EventTemplate {
  id: string;
  name: string;
  title: string;
  durationMins: number;
  category: string;
  color: string;
  description?: string;
}

export const builtInEventTemplates: EventTemplate[] = [
  { id: 'standup', name: 'Standup', title: 'Standup', durationMins: 15, category: 'Work', color: 'green' },
  { id: 'one-on-one', name: '1:1', title: '1:1', durationMins: 30, category: 'Work', color: 'blue' },
  { id: 'focus', name: 'Focus block', title: 'Focus time', durationMins: 60, category: 'Personal', color: 'purple' },
  { id: 'lunch', name: 'Lunch', title: 'Lunch', durationMins: 60, category: 'Personal', color: 'orange' },
];

export interface AppliedTemplate {
  title: string;
  endTime: string;
  category: string;
  color: string;
  description: string;
  spansNextDay: boolean;
}

function addMinutes(time: string, mins: number): { endTime: string; spansNextDay: boolean } {
  const [h, m] = time.split(':').map(Number);
  const total = h * 60 + m + mins;
  const wrapped = total % (24 * 60);
  return {
    endTime: `${String(Math.floor(wrapped / 60)).padStart(2, '0')}:${String(wrapped % 60).padStart(2, '0')}`,
    spansNextDay: total >= 24 * 60,
  };
}

/** Fill a new-event form from a template, anchoring duration at the chosen start. */
export function applyEventTemplate(template: EventTemplate, startTime: string): AppliedTemplate {
  const { endTime, spansNextDay } = addMinutes(startTime, template.durationMins);
  return {
    title: template.title,
    endTime,
    category: template.category,
    color: template.color,
    description: template.description ?? '',
    spansNextDay,
  };
}
