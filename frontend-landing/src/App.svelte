<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '@kestrel/shared/components/Button.svelte';
  import LabelPill from '@kestrel/shared/components/LabelPill.svelte';
  import ProviderBadge from '@kestrel/shared/components/ProviderBadge.svelte';
  import { site, platforms } from './site.js';

  let serverOnline: boolean | null = $state(null);
  let theme: 'light' | 'dark' =
    typeof document !== 'undefined' && document.documentElement.dataset.theme === 'light'
      ? 'light'
      : 'dark';

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
  });

  // --- Kestrel Mail data ---
  interface ThreadSample {
    initials: string;
    sender: string;
    subject: string;
    tag: string;
    snippet: string;
    provider: string;
    date: string;
  }

  const threads: ThreadSample[] = [
    {
      initials: 'CI',
      sender: 'CI Pipeline',
      subject: 'Release v0.1.0 — all platforms green',
      tag: 'devops',
      snippet: 'Windows, Linux, macOS, Android and iOS builds passed. Docker image pushed.',
      provider: 'gmail',
      date: '09:41',
    },
    {
      initials: 'AR',
      sender: 'Ana Ruiz',
      subject: 'Q3 budget review — your sign-off needed',
      tag: 'finance',
      snippet: 'Three open threads carry cost decisions. Sheet attached, deadline Friday.',
      provider: 'outlook',
      date: '08:15',
    },
    {
      initials: 'OP',
      sender: 'Ops',
      subject: 'Maintenance window confirmed for Saturday',
      tag: 'urgent',
      snippet: 'Sync pauses 02:00–02:30 UTC. The outbox holds all sends automatically.',
      provider: 'gmail',
      date: 'Tue',
    },
  ];

  const mailFeatures = [
    { title: 'Keyboard-first triage', body: 'Move with j/k, archive with e, snooze with s. A cheat sheet lives one keypress away.' },
    { title: 'Threads + labels', body: 'Conversations stay together. Cross-provider labels and search narrow thousands of threads to the few that matter.' },
    { title: 'Offline outbox', body: 'Sends queue locally and replay on reconnect. No lost drafts on bad networks.' },
    { title: 'Provider badges', body: 'Gmail and Outlook accounts side by side, always marked — you always know where a thread lives.' },
  ];

  // --- Interactive triage demo (Mail) ---
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

  // --- Kestrel Calendar data ---
  interface DaySchedule {
    d: string;
    items: { time: string; title: string; sub: string }[];
  }

  const week: DaySchedule[] = [
    { d: 'Mon', items: [{ time: '10:00 – 10:30', title: 'Sprint planning', sub: 'Engineering · video link attached' }] },
    {
      d: 'Tue',
      items: [
        { time: '09:30 – 09:45', title: 'Standup', sub: 'Engineering · video link attached' },
        { time: '14:00 – 15:00', title: 'Design review', sub: '4 attendees · recap filed to thread' },
        { time: '16:30 – 17:00', title: 'Release sign-off', sub: 'Go / no-go for the weekend cut' },
      ],
    },
    { d: 'Wed', items: [{ time: '11:00 – 11:30', title: '1:1 with Ana', sub: 'Budget thread linked' }] },
    { d: 'Thu', items: [] },
    { d: 'Fri', items: [{ time: '15:00 – 15:30', title: 'Release cut', sub: 'Poll closed · time won by vote' }] },
  ];

  let selectedDay: number = $state(1);

  const calFeatures = [
    { title: 'Week, day, month', body: 'Three views over the same data. Drag to reschedule; everything stays in sync with providers.' },
    { title: 'Scheduling polls', body: 'Propose times, collect votes, lock the winner — no five-message threads to find thirty minutes.' },
    { title: 'Team availability', body: 'Free/busy overlays across Google and Outlook calendars before any invite goes out.' },
    { title: 'Events linked to mail', body: 'Every meeting traces back to the thread that caused it. The recap files itself.' },
  ];

  const sharedPoints = [
    { k: 'One backend', v: 'Rust + Axum serves both apps, the API, and this page' },
    { k: 'One component library', v: 'Shared Svelte primitives keep both apps consistent' },
    { k: 'One deploy', v: 'Single Docker image · SQLite included · runs on a NAS' },
  ];

  const faqs = [
    {
      q: 'What do I need to self-host?',
      a: 'Docker and one open port. Copy the example env file, run compose up, and point the apps at your server URL. SQLite is the default — PostgreSQL is supported for larger setups.',
    },
    {
      q: 'Which email and calendar providers are supported?',
      a: 'Gmail and Outlook via sandboxed WASM provider plugins. Each account is connected once and shared by both Mail and Calendar.',
    },
    {
      q: 'Do the apps work offline?',
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

<nav class="nav">
  <div class="wrap nav-inner">
    <a class="brand" href="#top">
      <img src="/logo.svg" alt="Kestrel logo" width="28" height="28" />
      Kestrel
    </a>
    <div class="nav-links">
      <a href="#mail">Mail</a>
      <a href="#calendar">Calendar</a>
      <a href="#selfhost">Self-host</a>
      <a href="#download">Download</a>
      <a href="#faq">FAQ</a>
    </div>
    <div class="nav-actions">
      <button class="theme-btn" onclick={toggleTheme} aria-label="Toggle color theme">
        {theme === 'light' ? '◑' : '◐'}
      </button>
      <Button variant="secondary" size="sm" onclick={() => go(site.github)}>GitHub</Button>
      <Button variant="primary" size="sm" onclick={() => go('#download')}>Get Kestrel</Button>
    </div>
  </div>
</nav>

<div class="wrap" id="top">
  <header class="hero">
    <span class="eyebrow">Self-hosted · Two apps, one server</span>
    <h1>Email and calendar, minus the cloud.</h1>
    <p class="sub">
      Kestrel is a private mail and calendar suite that runs on your hardware. Two focused
      apps — Mail for triage, Calendar for scheduling — sharing one backend and one sync engine.
    </p>
    <div class="hero-cta">
      <Button variant="primary" onclick={() => go('#download')}>Download the apps</Button>
      <Button variant="secondary" onclick={() => go('#mail')}>Explore the apps</Button>
    </div>
    <div class="hero-facts">
      <span>Rust backend</span>
      <span>6 platforms</span>
      <span>Offline-first</span>
      <span class="status-inline">
        <span class="dot" class:on={serverOnline === true}></span>
        {serverOnline === null ? 'checking server…' : serverOnline ? 'server online' : 'offline preview'}
      </span>
    </div>
  </header>

  <!-- ============ KESTREL MAIL ============ -->
  <section class="block" id="mail">
    <div class="sec-index">Kestrel Mail</div>
    <h2>Reach inbox zero before standup.</h2>
    <p class="sec-sub">
      A keyboard-driven mail client for Gmail and Outlook. Triage in minutes, then close it —
      the outbox and sync handle the rest.
    </p>
    <div class="app-grid">
      <div class="shot">
        <div class="shot-bar"><span class="mono-dim">kestrel mail — inbox</span></div>
        {#each threads as t}
          <div class="thread">
            <div class="initials">{t.initials}</div>
            <div class="thread-body">
              <div class="thread-subject">
                {t.subject}
                <LabelPill tag={t.tag} />
              </div>
              <div class="thread-snippet">{t.sender} — {t.snippet}</div>
            </div>
            <div class="thread-side">
              <ProviderBadge provider={t.provider} />
              <span class="thread-date">{t.date}</span>
            </div>
          </div>
        {/each}
        <div class="showcase-note">Rendered with Mail&rsquo;s own shared components (@kestrel/shared)</div>
      </div>
      <div>
        <ul class="feat-list">
          {#each mailFeatures as f}
            <li><b>{f.title}</b><span>{f.body}</span></li>
          {/each}
        </ul>
        <div class="hero-cta" style="margin-top: 18px;">
          <Button variant="primary" onclick={() => go(site.mailDownload)}>Download Mail ↓</Button>
        </div>
      </div>
    </div>

    <div class="demo">
      <div class="demo-title">Try Mail&rsquo;s triage loop</div>
      <p class="demo-sub">
        Select with <span class="kbd">j</span>/<span class="kbd">k</span>, archive with
        <span class="kbd">e</span>, snooze with <span class="kbd">s</span> — or use the buttons.
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
  </section>

  <!-- ============ KESTREL CALENDAR ============ -->
  <section class="block" id="calendar">
    <div class="sec-index">Kestrel Calendar</div>
    <h2>Run the day from one view.</h2>
    <p class="sec-sub">
      A week-first calendar with polls and availability built in. Pick a day below — the agenda
      on the right is live.
    </p>
    <div class="app-grid">
      <div>
        <ul class="feat-list">
          {#each calFeatures as f}
            <li><b>{f.title}</b><span>{f.body}</span></li>
          {/each}
        </ul>
        <div class="hero-cta" style="margin-top: 18px;">
          <Button variant="primary" onclick={() => go(site.calendarDownload)}>Download Calendar ↓</Button>
        </div>
      </div>
      <div class="shot">
        <div class="shot-bar"><span class="mono-dim">kestrel calendar — september · click a day</span></div>
        <div class="cal-strip">
          {#each week as day, i}
            <button class="cal-day" class:today={day.d === 'Tue'} class:selected={i === selectedDay} onclick={() => (selectedDay = i)}>
              <b>{day.d}</b>
              <span class="cal-count">{day.items.length === 0 ? 'free' : `${day.items.length} event${day.items.length > 1 ? 's' : ''}`}</span>
            </button>
          {/each}
        </div>
        <div class="agenda">
          {#if week[selectedDay].items.length === 0}
            <div class="demo-empty">Nothing scheduled — a free {week[selectedDay].d}.</div>
          {:else}
            {#each week[selectedDay].items as e}
              <div class="evt">
                <span class="evt-time">{e.time}</span>
                <div>
                  <div class="evt-title">{e.title}</div>
                  <div class="evt-sub">{e.sub}</div>
                </div>
              </div>
            {/each}
          {/if}
        </div>
        <div class="showcase-note">
          <ProviderBadge provider="outlook" />
          <span style="margin-left: 8px;">synced calendars stay in step with Mail&rsquo;s accounts</span>
        </div>
      </div>
    </div>
  </section>

  <!-- ============ SHARED PLATFORM ============ -->
  <section class="block" id="platform">
    <div class="sec-index">Shared platform</div>
    <h2>Two apps, zero duplication.</h2>
    <p class="sec-sub">Mail and Calendar are separate clients on top of one backend, one sync engine, and one UI library.</p>
    <dl class="stack">
      {#each sharedPoints as f}
        <div class="stack-item">
          <dt>{f.k}</dt>
          <dd>{f.v}</dd>
        </div>
      {/each}
    </dl>
  </section>

  <section class="block" id="selfhost">
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
    </p>
  </section>

  <section class="block" id="download">
    <div class="sec-index">Download</div>
    <h2>Get the latest builds.</h2>
    <p class="sec-sub">
      Mail and Calendar ship as separate installers in every tagged release. This page is served
      by a Kestrel backend — pick your app below.
    </p>
    <div class="dl">
      {#each platforms as p}
        <div class="dl-row">
          <div>
            <span class="dl-os">{p.os}</span>
            <span class="dl-format">{p.format}</span>
          </div>
          <div class="dl-btns">
            <a class="dl-link primary" href={site.mailDownload}>Mail ↓</a>
            <a class="dl-link" href={site.calendarDownload}>Calendar ↓</a>
          </div>
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

  <section class="block" id="faq">
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
      <Button variant="secondary" onclick={() => go(site.github)}>Browse the source</Button>
    </div>
  </section>
</div>

<footer>
  <div class="wrap foot">
    <span>Kestrel · private mail + calendar</span>
    <span>
      <a href={site.github}>GitHub</a>
      &nbsp;·&nbsp;
      <a href="mailto:{site.email}">Contact</a>
      &nbsp;·&nbsp;
      <a href="/api/health">API status</a>
      {#if site.buildNotes !== ''}&nbsp;·&nbsp;<a href={site.buildNotes}>Build notes</a>{/if}
    </span>
  </div>
</footer>
