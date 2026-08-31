export const mailStore = $state({
    unreadFilterOnly: false,

    toggleUnreadFilter() {
        this.unreadFilterOnly = !this.unreadFilterOnly;
    },

    setUnreadFilter(value: boolean) {
        this.unreadFilterOnly = value;
    }
});
