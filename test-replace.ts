console.log("- [ ] Task 1".replace(/^(\s*)[-*]\s+\[([ xX])\]\s+(.*)$/gm, (match, indent, checked, text) => {
    return `<input type="checkbox">`;
}));
