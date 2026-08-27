export interface IcsEvent {
  uid: string;
  summary: string;
  dtstart?: string;
  dtend?: string;
  organizer?: string;
  location?: string;
}

export function parseIcs(content: string): IcsEvent | null {
  const lines = content.split(/\r?\n/);

  const event: Partial<IcsEvent> = {};

  for (let line of lines) {
    if (line.startsWith('SUMMARY:')) {
      event.summary = line.substring(8).trim();
    } else if (line.startsWith('UID:')) {
      event.uid = line.substring(4).trim();
    } else if (line.startsWith('LOCATION:')) {
      event.location = line.substring(9).trim();
    } else if (line.startsWith('DTSTART')) {
      const idx = line.indexOf(':');
      if (idx !== -1) event.dtstart = line.substring(idx + 1).trim();
    } else if (line.startsWith('DTEND')) {
      const idx = line.indexOf(':');
      if (idx !== -1) event.dtend = line.substring(idx + 1).trim();
    } else if (line.startsWith('ORGANIZER')) {
      const match = /mailto:(.+)/i.exec(line);
      if (match && match[1]) {
        event.organizer = match[1].trim();
      }
    }
  }

  if (event.uid && event.summary) {
    return event as IcsEvent;
  }

  return null;
}
