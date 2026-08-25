export interface CalendarSet {
    id: string;
    name: string;
    calendarIds: string[];
}

export function createCalendarSets() {
    let sets = $state<CalendarSet[]>([]);

    if (typeof window !== 'undefined') {
        const saved = localStorage.getItem('kestrel_calendar_sets');
        if (saved) {
            try {
                sets = JSON.parse(saved);
            } catch (e) {
                console.error("Failed to parse calendar sets", e);
            }
        }
    }

    return {
        get sets() { return sets; },
        set sets(value: CalendarSet[]) { sets = value; },
        save: () => {
            if (typeof window !== 'undefined') {
                localStorage.setItem('kestrel_calendar_sets', JSON.stringify(sets));
            }
        },
        addSet: (name: string, calendarIds: string[]) => {
            sets.push({
                id: crypto.randomUUID(),
                name,
                calendarIds
            });
            if (typeof window !== 'undefined') {
                localStorage.setItem('kestrel_calendar_sets', JSON.stringify(sets));
            }
        },
        removeSet: (id: string) => {
            sets = sets.filter(s => s.id !== id);
            if (typeof window !== 'undefined') {
                localStorage.setItem('kestrel_calendar_sets', JSON.stringify(sets));
            }
        },
        renameSet: (id: string, name: string) => {
            const set = sets.find(s => s.id === id);
            if (set) {
                set.name = name;
                if (typeof window !== 'undefined') {
                    localStorage.setItem('kestrel_calendar_sets', JSON.stringify(sets));
                }
            }
        },
        updateSetCalendars: (id: string, calendarIds: string[]) => {
            const set = sets.find(s => s.id === id);
            if (set) {
                set.calendarIds = calendarIds;
                if (typeof window !== 'undefined') {
                    localStorage.setItem('kestrel_calendar_sets', JSON.stringify(sets));
                }
            }
        }
    };
}

export const calendarSetsStore = createCalendarSets();
