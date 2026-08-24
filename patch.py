with open("frontend-calendar/src/routes/+page.svelte", "r") as f:
    data = f.read()

data = data.replace("{:else if isDetailsDocked && !isMobileOrTablet}", "{:else if isDetailsDocked && !isMobileOrTablet && !selectedEvent}")

with open("frontend-calendar/src/routes/+page.svelte", "w") as f:
    f.write(data)
