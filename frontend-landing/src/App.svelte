<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '@kestrel/shared/components/Button.svelte';
  import ProviderBadge from '@kestrel/shared/components/ProviderBadge.svelte';
  import ThreadList from 'frontend-mail/src/lib/components/ThreadList.svelte';
  import WeekGrid from 'frontend-calendar/src/lib/components/WeekGrid.svelte';
  import MonthGrid from 'frontend-calendar/src/lib/components/MonthGrid.svelte';
  import { site, platforms } from './site.js';

  let serverOnline: boolean | null = $state(null);
  let theme: 'light' | 'dark' =
    typeof document !== 'undefined' && document.documentElement.dataset.theme === 'light'
      ? 'light'
      : 'dark';
  let previewTab: 'mail' | 'calendar' = $state('mail');
  let menuOpen: boolean = $state(false);

  // Release tag baked in at build time; empty when unknown (never a stale version).
  const version: string = __KESTREL_VERSION__;

  function toggleTheme(): void {
    theme = theme === 'light' ? 'dark' : 'light';
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem('kestrel-theme', theme);
    } catch {
      /* storage unavailable — theme still applies for the session */
    }
  }

  function go(url: string): void {
    window.location.href = url;
  }

  onMount(() => {
    fetch('/api/health')
      .then((r) => (serverOnline = r.ok))
      .catch(() => (serverOnline = false));

    const els = document.querySelectorAll(
      '.shot, .demo, .dl, .arch, .final, .stats, .faq details, .feat-list li, .code',
    );
    if ('IntersectionObserver' in window) {
      const io = new IntersectionObserver(
        (entries) => {
          for (const e of entries) {
            if (e.isIntersecting) {
              e.target.classList.add('in');
              io.unobserve(e.target);
            }
          }
        },
        { threshold: 0.12 },
      );
      els.forEach((el) => {
        el.classList.add('reveal');
        io.observe(el);
      });
    }
  });

  // --- Real ThreadList data (same shape the Mail app uses) ---
  const threads = [
    {
      id: 't1',
      sender: 'CI Pipeline',
      senderEmail: 'ci@example.com',
      subject: 'Release v0.1.0 — all platforms green',
      snippet: 'Windows, Linux, macOS, Android and iOS builds passed. Docker image pushed.',
      date: '09:41',
      isUnread: true,
      isStarred: false,
      hasAttachment: false,
      labels: ['devops'],
    },
    {
      id: 't2',
      sender: 'Ana Ruiz',
      senderEmail: 'ana@example.com',
      subject: 'Q3 budget review — your sign-off needed',
      snippet: 'Three open threads carry cost decisions. Sheet attached, deadline Friday.',
      date: '08:15',
      isUnread: true,
      isStarred: true,
      hasAttachment: true,
      labels: ['finance'],
    },
    {
      id: 't3',
      sender: 'Ops',
      senderEmail: 'ops@example.com',
      subject: 'Maintenance window confirmed for Saturday',
      snippet: 'Sync pauses 02:00–02:30 UTC. The outbox holds all sends automatically.',
      date: 'Tue',
      isUnread: false,
      isStarred: false,
      hasAttachment: false,
      labels: ['urgent'],
    },
  ];

  // --- Real WeekGrid data (same shape the Calendar app uses) ---
  function dayStr(offset: number): string {
    const d = new Date();
    const dow = (d.getDay() + 6) % 7; // Monday = 0
    d.setDate(d.getDate() - dow + offset);
    return d.toISOString().slice(0, 10);
  }

  const calEvents = [
    { id: 'e1', title: 'Sprint planning', date: dayStr(0), startTime: '10:00', endTime: '10:30', color: 'blue', calendarId: 'work' },
    { id: 'e2', title: 'Standup', date: dayStr(1), startTime: '09:30', endTime: '09:45', color: 'rose', calendarId: 'work' },
    { id: 'e3', title: 'Design review', date: dayStr(1), startTime: '14:00', endTime: '15:00', color: 'purple', calendarId: 'work' },
    { id: 'e4', title: '1:1 with Ana', date: dayStr(2), startTime: '11:00', endTime: '11:30', color: 'green', calendarId: 'work' },
    { id: 'e5', title: 'Release cut', date: dayStr(4), startTime: '15:00', endTime: '15:30', color: 'amber', calendarId: 'work' },
  ] as { id: string; title: string; date: string; startTime: string; endTime: string; color: string; calendarId: string }[];

  const monthColors = ['#0078D4', '#D15B47', '#34d399', '#c084fc', '#E5B722'];
  const monthEvents = calEvents.map((e, i) => ({
    ...e,
    color: monthColors[i % monthColors.length],
    dayIndex: (i * 2 + 1) % 7,
  }));

  const mailBenefits = [
    { title: 'Clear the inbox in minutes', body: 'Move with j/k, archive with e, snooze with s. Triage becomes a ten-second loop instead of a morning lost to clicking.' },
    { title: 'Threads stay together', body: 'Conversations, labels, and cross-provider search narrow thousands of threads to the few that need you.' },
    { title: 'Never lose a send', body: 'The offline outbox queues drafts and replays them on reconnect. Bad network, zero lost mail.' },
  ];

  const calBenefits = [
    { title: 'Vote on times, skip the thread', body: 'Scheduling polls collect votes and lock the winner — no five-message chains to find thirty minutes.' },
    { title: 'See availability first', body: 'Free/busy overlays across Google and Outlook calendars before any invite goes out.' },
    { title: 'Meetings linked to mail', body: 'Every event traces back to the thread that caused it. The recap files itself.' },
  ];

  // --- Interactive triage demo (Mail's real loop, sample data) ---
  interface DemoThread {
    id: number;
    from: string;
    subject: string;
    state: 'inbox' | 'archived' | 'snoozed';
  }

  const demoSeed: DemoThread[] = [
    { id: 1, from: 'CI Pipeline', subject: 'Nightly build passed', state: 'inbox' },
    { id: 2, from: 'Ana Ruiz', subject: 'Q3 budget review', state: 'inbox' },
    { id: 3, from: 'Ops', subject: 'Saturday maintenance window', state: 'inbox' },
    { id: 4, from: 'Design', subject: 'New empty-state mockups', state: 'inbox' },
  ];

  let demo: DemoThread[] = $state(demoSeed.map((t) => ({ ...t })));
  let cursor: number = $state(0);

  let inboxCount = $derived(demo.filter((t) => t.state === 'inbox').length);
  let archivedCount = $derived(demo.filter((t) => t.state === 'archived').length);
  let snoozedCount = $derived(demo.filter((t) => t.state === 'snoozed').length);

  function visibleThreads(): DemoThread[] {
    return demo.filter((t) => t.state === 'inbox');
  }

  function move(dir: 1 | -1): void {
    const n = visibleThreads().length;
    if (n === 0) return;
    cursor = (cursor + dir + n) % n;
  }

  function act(action: 'archived' | 'snoozed'): void {
    const list = visibleThreads();
    if (list.length === 0) return;
    list[cursor].state = action;
    const remaining = visibleThreads().length;
    cursor = Math.min(cursor, Math.max(0, remaining - 1));
  }

  function resetDemo(): void {
    demo = demoSeed.map((t) => ({ ...t }));
    cursor = 0;
  }

  function demoKey(e: KeyboardEvent): void {
    const el = e.target as HTMLElement | null;
    if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) return;
    if (e.key === 'j') move(1);
    else if (e.key === 'k') move(-1);
    else if (e.key === 'e') act('archived');
    else if (e.key === 's') act('snoozed');
  }

  const faqs = [
    {
      q: 'What do I need to run it?',
      a: 'Docker and one open port. Copy the example env file, run compose up, and point the apps at your server URL. SQLite is built in — PostgreSQL is supported for larger setups. If you can run one container, you can run Kestrel.',
    },
    {
      q: 'Do I have to give up Gmail or Outlook?',
      a: 'No. Kestrel syncs your existing Gmail and Outlook accounts through sandboxed provider plugins. Your addresses stay the same — only where the mail lives changes.',
    },
    {
      q: 'What happens to my mail if I stop using Kestrel?',
      a: 'Nothing bad. Your mail still exists at your providers, and anything Kestrel stored can be exported. There is no lock-in because there is nothing to lock you into — the code is open source and the data is yours.',
    },
    {
      q: 'Does it work offline?',
      a: 'Yes. Mail caches threads and queues sends in a local outbox with automatic retry. Calendar keeps your schedule readable and replays changes when the connection returns.',
    },
    {
      q: 'Are Mail and Calendar separate downloads?',
      a: 'Yes. Each app ships as its own installer per platform in every tagged release — install one or both.',
    },
    {
      q: 'How much does it cost?',
      a: 'The software is free to self-host. You only pay for whatever hardware or VPS you run it on.',
    },
  ];
</script>

<svelte:window onkeydown={demoKey} />

<div class="announce">
  {#if version !== ''}Kestrel {version} — {/if}Mail + Calendar are ready to self-host. <a href="#download">Get the builds →</a>
</div>
<nav class="nav">
  <div class="wrap nav-inner">
    <a class="brand" href="#top">
      <img src="/logo.svg" alt="Kestrel logo" width="28" height="28" />
      Kestrel
    </a>
    <div class="nav-links" class:open={menuOpen}>
      <a href="#product" onclick={() => (menuOpen = false)}>Product</a>
      <a href="#why" onclick={() => (menuOpen = false)}>Why Kestrel</a>
      <a href="#download" onclick={() => (menuOpen = false)}>Download</a>
      <a href="#faq" onclick={() => (menuOpen = false)}>FAQ</a>
    </div>
    <div class="nav-actions">
      <button class="nav-toggle" onclick={() => (menuOpen = !menuOpen)} aria-label="Toggle navigation menu" aria-expanded={menuOpen}>
        {menuOpen ? '✕' : '☰'}
      </button>
      <button class="theme-btn" onclick={toggleTheme} aria-label="Toggle color theme">
        {theme === 'light' ? '◑' : '◐'}
      </button>
      <a class="nav-github" href={site.github}>GitHub</a>
      <Button variant="primary" size="sm" onclick={() => go('#download')}>Download Kestrel</Button>
    </div>
  </div>
</nav>

<div class="wrap" id="top">
  <header class="hero">
    <span class="eyebrow">Self-hosted mail + calendar</span>
    <h1>Email and calendar that live on your server.</h1>
    <p class="sub">
      Your Gmail and Outlook accounts, synced to a server you own. One Docker image
      on your hardware, two fast apps everywhere else.
    </p>
    <div class="hero-cta">
      <Button variant="primary" onclick={() => go('#download')}>Download Kestrel</Button>
      <a class="ghost-link" href="#product">See how it works ↓</a>
    </div>
    <p class="trust-line">Free to self-host · No account · No credit card · Open source</p>

    <div class="shot hero-shot" id="product">
      <div class="shot-tabs" role="tablist" aria-label="Product preview">
        <button role="tab" aria-selected={previewTab === 'mail'} class:active={previewTab === 'mail'} onclick={() => (previewTab = 'mail')}>
          Kestrel Mail — live thread list
        </button>
        <button role="tab" aria-selected={previewTab === 'calendar'} class:active={previewTab === 'calendar'} onclick={() => (previewTab = 'calendar')}>
          Kestrel Calendar — live week view
        </button>
      </div>
      <div class="frame" class:narrow={previewTab === 'mail'}>
        {#if previewTab === 'mail'}
          <ThreadList threads={threads} currentView="inbox" />
        {:else}
          <WeekGrid events={calEvents} viewMode="week" />
        {/if}
      </div>
      <div class="showcase-note">
        Not a mockup — the actual ThreadList and WeekGrid components from the apps, running with sample data.
      </div>
    </div>

    <div class="stats">
      <span><b>2</b> apps</span>
      <span><b>6</b> platforms</span>
      <span><b>2</b> providers synced</span>
      <span><b>1</b> Docker image</span>
    </div>
  </header>

  <section class="block band" id="why">
    <div class="sec-index">Why Kestrel</div>
    <h2>Your email lives on someone else&rsquo;s computer.</h2>
    <div class="problem-grid">
      <p><b>Your mail is someone else&rsquo;s product.</b> Every message sits on servers you don&rsquo;t control, scanned to sell ads and one outage away from unreachable.</p>
      <p class="answer"><span>→ Kestrel:</span> your mail lives on your server. No ads, no tracking, no one reading over your shoulder — the code is open source, so you don&rsquo;t take that on faith.</p>
      <p><b>Switching feels like a second job.</b> New address, lost history, a setup weekend — so the inbox keeps growing and the unease about who reads it never leaves.</p>
      <p class="answer"><span>→ Kestrel:</span> keep your providers and addresses. Gmail and Outlook sync in, history intact — only where the mail lives changes.</p>
      <p><b>The clients eat your mornings.</b> Click-heavy triage in one tab, scheduling ping-pong in another, and nothing works when the network drops.</p>
      <p class="answer"><span>→ Kestrel:</span> keyboard triage that clears the inbox in minutes, scheduling polls instead of threads, and an offline outbox that never loses a send.</p>
    </div>
    <p class="sec-sub" style="margin-top: 28px;">
      That&rsquo;s the whole pitch. Here&rsquo;s what it looks like in each app.
    </p>
  </section>

  <section class="block" id="mail">
    <div class="sec-index">Kestrel Mail</div>
    <h2>Reach inbox zero before standup.</h2>
    <p class="sec-sub">
      A keyboard-driven client for your existing Gmail and Outlook accounts. Same addresses,
      same history — triage in minutes, then close it.
    </p>
    <ul class="feat-list">
      {#each mailBenefits as f}
        <li><b>{f.title}</b><span>{f.body}</span></li>
      {/each}
    </ul>
    <div class="demo">
      <div class="demo-title">Try Mail&rsquo;s triage loop</div>
      <p class="demo-sub">
        The real loop, sample data. Select with <span class="kbd">j</span>/<span class="kbd">k</span>,
        archive with <span class="kbd">e</span>, snooze with <span class="kbd">s</span> — or use the buttons.
      </p>
      <div class="demo-counts">
        <span>Inbox <b>{inboxCount}</b></span>
        <span>Archived <b>{archivedCount}</b></span>
        <span>Snoozed <b>{snoozedCount}</b></span>
        <button class="link-btn" onclick={resetDemo}>Reset</button>
      </div>
      {#each visibleThreads() as t, i}
        <button class="demo-row" class:selected={i === cursor} onclick={() => (cursor = i)}>
          <span class="demo-from">{t.from}</span>
          <span class="demo-subject">{t.subject}</span>
          <span class="demo-hint">{i === cursor ? 'e archive · s snooze' : ''}</span>
        </button>
      {/each}
      {#if visibleThreads().length === 0}
        <div class="demo-empty">Inbox zero. That took about ten seconds. <button class="link-btn" onclick={resetDemo}>Run it again</button></div>
      {/if}
      <div class="demo-keys">
        <button class="kbd-btn" onclick={() => move(-1)}><span class="kbd">k</span> up</button>
        <button class="kbd-btn" onclick={() => move(1)}><span class="kbd">j</span> down</button>
        <button class="kbd-btn" onclick={() => act('archived')}><span class="kbd">e</span> archive</button>
        <button class="kbd-btn" onclick={() => act('snoozed')}><span class="kbd">s</span> snooze</button>
      </div>
    </div>
    <div class="hero-cta" style="margin-top: 22px;">
      <Button variant="secondary" onclick={() => go(site.mailDownload)}>Download Kestrel Mail ↓</Button>
    </div>
  </section>

  <section class="block band" id="calendar">
    <div class="sec-index">Kestrel Calendar</div>
    <h2>Run the day from one view.</h2>
    <p class="sec-sub">
      A week-first calendar over the same accounts Mail syncs. The live week view above is the
      app itself — here is what it does for you.
    </p>
    <ul class="feat-list">
      {#each calBenefits as f}
        <li><b>{f.title}</b><span>{f.body}</span></li>
      {/each}
    </ul>
    <div class="shot" style="margin-top: 28px;">
      <div class="shot-bar"><span class="mono-dim">kestrel calendar — month view, live component</span></div>
      <div class="frame">
        <MonthGrid events={monthEvents} />
      </div>
      <div class="showcase-note">The app&rsquo;s own MonthGrid, running with sample data.</div>
    </div>
    <div class="cal-proof">
      <ProviderBadge provider="outlook" />
      <span>Synced calendars stay in step with Mail&rsquo;s accounts — connect once, use both.</span>
    </div>
    <div class="hero-cta" style="margin-top: 22px;">
      <Button variant="secondary" onclick={() => go(site.calendarDownload)}>Download Kestrel Calendar ↓</Button>
    </div>
  </section>

  <section class="block band" id="selfhost">
    <div class="sec-index">Self-hosting</div>
    <h2>Yours in three steps.</h2>
    <p class="sec-sub">No accounts, no subscriptions, no data leaving your network unless you say so.</p>
    <div class="arch" aria-label="Deployment diagram">
      <div class="arch-node"><b>Mail + Calendar apps</b><span>Windows · macOS · Linux · Android · iOS</span></div>
      <div class="arch-arrow" aria-hidden="true">→</div>
      <div class="arch-node highlight"><b>Your Kestrel server</b><span>One Docker image · SQLite included</span></div>
      <div class="arch-arrow" aria-hidden="true">→</div>
      <div class="arch-node"><b>Gmail · Outlook</b><span>Sandboxed sync plugins</span></div>
    </div>
    <div class="code">cp .env.example .env
docker compose up -d</div>
    <p class="sec-sub" style="margin-top: 12px;">
      Expose it with a Cloudflare Tunnel for public access or keep it on Tailscale for private use.
      This page is served by that same backend — no separate hosting.
      <span class="status-inline" style="margin-left: 8px;">
        <span class="dot" class:on={serverOnline === true}></span>
        {serverOnline === null ? 'checking…' : serverOnline ? 'this server is online' : 'offline preview'}
      </span>
    </p>
  </section>

  <section class="block" id="download">
    <div class="sec-index">Download</div>
    <h2>Get the latest builds.</h2>
    <p class="sec-sub">
      Mail and Calendar ship as separate installers in every tagged release. The buttons
      below open the latest release.
    </p>
    <div class="dl">
      {#each platforms as p}
        <div class="dl-row">
          <div>
            <span class="dl-os">{p.os}</span>
            <span class="dl-format">{p.format}</span>
          </div>
          {#if p.os === 'iOS'}
            <div class="dl-btns">
              {#if site.iosTestFlight !== ''}
                <a class="dl-link primary" href={site.iosTestFlight}>Join TestFlight →</a>
              {:else}
                <span class="dl-note">TestFlight · invite only for now</span>
              {/if}
            </div>
          {:else}
            <div class="dl-btns">
              <a class="dl-link primary" href={site.mailDownload}>Mail ↓</a>
              <a class="dl-link" href={site.calendarDownload}>Calendar ↓</a>
            </div>
          {/if}
        </div>
      {/each}
      <div class="dl-row">
        <div>
          <span class="dl-os">Server</span>
          <span class="dl-format">Docker</span>
          <div class="dl-note">Self-host the backend that serves this page</div>
        </div>
        <div class="dl-btns">
          <a class="dl-link" href={site.releaseNotes}>Release notes</a>
        </div>
      </div>
    </div>
    <div class="code">docker pull {site.dockerImage}</div>
  </section>

  <section class="block band" id="faq">
    <div class="sec-index">FAQ</div>
    <h2>Questions, answered.</h2>
    <div class="faq">
      {#each faqs as f}
        <details>
          <summary>{f.q}</summary>
          <p>{f.a}</p>
        </details>
      {/each}
    </div>
  </section>

  <section class="final">
    <h2>Take back your inbox.</h2>
    <p class="sec-sub">Free to self-host. Install both apps in under five minutes.</p>
    <div class="hero-cta" style="justify-content: center;">
      <Button variant="primary" onclick={() => go('#download')}>Download Kestrel</Button>
    </div>
    <p class="trust-line" style="text-align: center;">Free to self-host · No account · No credit card · Open source</p>
  </section>
</div>

<footer>
  <div class="wrap">
    <div class="foot-grid">
      <div class="foot-brand">
        <a class="brand" href="#top">
          <img src="/logo.svg" alt="Kestrel logo" width="28" height="28" />
          Kestrel
        </a>
        <p>A private, self-hosted mail and calendar suite. Your accounts, synced to a server you own — under your control.</p>
      </div>
      <div class="foot-col">
        <h4>Product</h4>
        <a href="#mail">Kestrel Mail</a>
        <a href="#calendar">Kestrel Calendar</a>
        <a href="#download">Download</a>
        <a href="#faq">FAQ</a>
      </div>
      <div class="foot-col">
        <h4>Resources</h4>
        <a href={site.github}>GitHub</a>
        <a href={site.releaseNotes}>Release notes</a>
        <a href="/api/health">API status</a>
      </div>
    </div>
    <div class="foot-base">
      <span>Kestrel{#if version !== ''} {version}{/if} · served by its own backend</span>
      {#if site.buildNotes !== ''}<a href={site.buildNotes}>Build notes</a>{/if}
    </div>
  </div>
</footer>
