with open("frontend-calendar/src/routes/+page.svelte", "r") as f:
    data = f.read()

# Fix the spacing
data = data.replace("{isDetailsDocked && !clickPosition ? 'lg:mr-80' : ''}", "{isDetailsDocked ? 'lg:mr-80' : ''}")

with open("frontend-calendar/src/routes/+page.svelte", "w") as f:
    f.write(data)
