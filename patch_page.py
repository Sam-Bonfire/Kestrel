import re

with open('frontend-mail/src/routes/+page.svelte', 'r') as f:
    content = f.read()

# I am having trouble replacing exactly the lines via patch format. Let's do it carefully with re.

content = content.replace("bind:searchQuery", "searchQuery={searchQuery}")

content = content.replace("selectedThreadId = id;", "tabsStore.updateActiveTab({ threadId: id });")
content = content.replace("onFilterChange={() => tabsStore.updateActiveTab({ view: currentView, threadId: null })}", "onFilterChange={() => { tabsStore.updateActiveTab({ view: currentView, threadId: null }); }}")

content = content.replace("currentView = 'inbox';", "tabsStore.updateActiveTab({ view: 'inbox', threadId: null });")
content = content.replace("currentView = v;", "tabsStore.updateActiveTab({ view: v, threadId: null });")
content = content.replace("currentView = cmd.replace('view-', '');", "tabsStore.updateActiveTab({ view: cmd.replace('view-', ''), threadId: null });")
content = content.replace("currentView = 'all-mail';", "tabsStore.updateActiveTab({ view: 'all-mail' });")
content = content.replace("selectedThreadId = null;", "tabsStore.updateActiveTab({ threadId: null });")

with open('frontend-mail/src/routes/+page.svelte', 'w') as f:
    f.write(content)
