/**
 * Checks if the user is currently typing in an input field.
 * This should be used to guard global keyboard shortcuts.
 */
export function isTyping(event: KeyboardEvent): boolean {
    const target = event.target as HTMLElement;
    if (!target) return false;
    
    return (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable
    );
}
