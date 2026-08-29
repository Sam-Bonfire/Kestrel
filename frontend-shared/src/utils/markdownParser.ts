export function parseChecklists(html: string): string {
    // Basic markdown list format: "- [ ] task" or "* [x] task"
    // Note: Emails might contain HTML list items like "<li>[ ] task</li>" too.

    // Replace text-based markdown checkboxes
    let parsed = html.replace(/^(\s*)[-*]\s+\[([ xX])\]\s+(.*)$/gm, (match, indent, checked, text, offset) => {
        const isChecked = checked.trim().toLowerCase() === 'x';
        const checkboxHtml = `<div class="kestrel-task-item" data-task-id="task-${offset}" style="display: flex; align-items: flex-start; gap: 8px; margin: 4px 0;">
            <input type="checkbox" class="kestrel-task-checkbox" ${isChecked ? 'checked' : ''} style="margin-top: 4px; cursor: pointer;">
            <span class="kestrel-task-text" style="${isChecked ? 'text-decoration: line-through; opacity: 0.7;' : ''}">${text}</span>
        </div>`;
        return `${indent}${checkboxHtml}`;
    });

    // Replace html-based list checkboxes (<li>[ ] task</li>)
    parsed = parsed.replace(/<li([^>]*)>\s*\[([ xX])\]\s+(.*?)<\/li>/gis, (match, attrs, checked, text, offset) => {
        const isChecked = checked.trim().toLowerCase() === 'x';
        const checkboxHtml = `<li${attrs} class="kestrel-task-list-item" style="list-style: none;">
            <div class="kestrel-task-item" data-task-id="task-${offset}" style="display: flex; align-items: flex-start; gap: 8px; margin: 4px 0;">
                <input type="checkbox" class="kestrel-task-checkbox" ${isChecked ? 'checked' : ''} style="margin-top: 4px; cursor: pointer;">
                <span class="kestrel-task-text" style="${isChecked ? 'text-decoration: line-through; opacity: 0.7;' : ''}">${text}</span>
            </div>
        </li>`;
        return checkboxHtml;
    });

    return parsed;
}
