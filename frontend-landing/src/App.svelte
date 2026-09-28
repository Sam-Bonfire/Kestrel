<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '@kestrel/shared/components/Button.svelte';
  import ProviderBadge from '@kestrel/shared/components/ProviderBadge.svelte';
  import ThreadList from 'frontend-mail/src/lib/components/ThreadList.svelte';
  import WeekGrid from 'frontend-calendar/src/lib/components/WeekGrid.svelte';
  import MonthGrid from 'frontend-calendar/src/lib/components/MonthGrid.svelte';
  import { site, platforms } from './site.js';

  let theme: 'light' | 'dark' =
    typeof document !== 'undefined' && document.documentElement.dataset.theme === 'light'
      ? 'light'
      : 'dark';
  let previewTab: 'mail' | 'calendar' = $state('mail');
  let menuOpen: boolean = $state(false);

  // Release tag baked in at build time; empty when unknown (never a stale version).
  const version: string = __KESTREL_VERSION__;

  // --- Direct download links, resolved from the latest GitHub release ---
  // Buttons fall back to the release page until (or unless) the API answers.
  let dl: Record<string, string> = $state({});

  interface ReleaseAsset {
    name: string;
    browser_download_url: string;
  }

  function pickAsset(assets: ReleaseAsset[], re: RegExp): string | null {
    return assets.find((a) => re.test(a.name))?.browser_download_url ?? null;
  }

  function dlUrl(os: string, app: 'mail' | 'cal'): string {
    const prefix = os === 'Windows' ? 'win' : os === 'Linux' ? 'linux' : null;
    const fallback = app === 'mail' ? site.mailDownload : site.calendarDownload;
    if (!prefix) return fallback;
    return dl[`${prefix}-${app}`] ?? fallback;
  }

  function resolveDownloads(): void {
    fetch(`https://api.github.com/repos/${site.repo}/releases/latest`)
      .then((r) => (r.ok ? r.json() : null))
      .then((rel: { assets?: ReleaseAsset[] } | null) => {
        if (!rel || !Array.isArray(rel.assets)) return;
        const assets = rel.assets;
        const next: Record<string, string> = {};
        const put = (key: string, re: RegExp): void => {
          const url = pickAsset(assets, re);
          if (url) next[key] = url;
        };
        put('win-mail', /^kestrel\.mail.*\.exe$/i);
        put('win-cal', /^kestrel\.calendar.*\.exe$/i);
        put('linux-mail', /^kestrel\.mail.*\.appimage$/i);
        put('linux-cal', /^kestrel\.calendar.*\.appimage$/i);
        dl = next;
      })
      .catch(() => {
        /* offline or rate-limited — release-page links stay */
      });
  }

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
    resolveDownloads();

    const els = document.querySelectorAll(
      '.shot, .demo, .dl, .final, .stats, .faq details, .feat li, .card, .step, .cmp-wrap, .hero-mock, .connect',
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
    {
      id: 't4',
      sender: 'Design',
      senderEmail: 'design@example.com',
      subject: 'New empty-state mockups are ready',
      snippet: 'Three directions for the zero-thread view. Vote with an emoji by EOD.',
      date: 'Tue',
      isUnread: false,
      isStarred: false,
      hasAttachment: true,
      labels: [],
    },
    {
      id: 't5',
      sender: 'Priya Nair',
      senderEmail: 'priya@example.com',
      subject: 'Interview loop — Thursday panel',
      snippet: 'You are on the 14:00 panel. Scorecard attached, please review beforehand.',
      date: 'Mon',
      isUnread: false,
      isStarred: true,
      hasAttachment: true,
      labels: ['careers'],
    },
    {
      id: 't6',
      sender: 'Calendar',
      senderEmail: 'calendar@example.com',
      subject: 'Design review moved to 15:00',
      snippet: 'Moved by Ana. The poll winner is locked in — see you at three.',
      date: 'Mon',
      isUnread: false,
      isStarred: false,
      hasAttachment: false,
      labels: [],
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

  const features = [
    {
      icon: 'inbox',
      title: 'One inbox, both providers',
      body: 'Gmail and Outlook side by side in a single unified inbox. Same addresses, same history — no forwarding rules, no migration.',
    },
    {
      icon: 'kbd',
      title: 'Keyboard-first triage',
      body: 'Move with j/k, archive with e, snooze with s. Clearing the inbox becomes a ten-second loop you can try live below.',
    },
    {
      icon: 'cal',
      title: 'Scheduling without threads',
      body: 'Polls collect votes and lock the winner, with free/busy across Google and Outlook calendars before any invite goes out.',
    },
    {
      icon: 'off',
      title: 'Offline outbox',
      body: 'Drafts queue locally and replay on reconnect. Tunnel commute, flaky café wifi — zero lost sends.',
    },
    {
      icon: 'search',
      title: 'Cross-provider search',
      body: 'Threads, labels, and conversations searchable across every connected account from one command palette.',
    },
    {
      icon: 'link',
      title: 'Mail meets calendar',
      body: 'Every event traces back to the thread that caused it. The recap practically files itself.',
    },
  ];

  const iconPaths: Record<string, string> = {
    inbox:
      'M22 12h-6l-2 3h-4l-2-3H2 M5.5 5.1 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.5-6.9A2 2 0 0 0 16.7 4H7.3a2 2 0 0 0-1.8 1.1z',
    kbd: 'M4 17V7a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm5-6h1m2 0h1m2 0h1m-8 4h8',
    cal: 'M8 2v4m8-4v4M3 9h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z',
    off: 'M8.7 16.7 3 12l9-9 5.5 5.5M5 5l16 16M17 8.5V7h1.5M22 12l-3-3m-3.5 10.5c-1 .5-2.4.8-3.5.8a8 8 0 0 1-8-8c0-1.1.3-2.5.8-3.5',
    search: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zm10 2-4.3-4.3',
    link: 'M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7',
  };

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
      q: 'Do I need a new email address?',
      a: 'No. Kestrel connects to the Gmail and Outlook accounts you already have. Your addresses, history, and folders stay exactly where they are — Kestrel is the client you read and triage them in.',
    },
    {
      q: 'How do my accounts connect?',
      a: 'With OAuth through Google and Microsoft — the same “sign in with” flow every mail client uses. Kestrel never sees your password, and you can revoke access from your Google or Microsoft account page at any time.',
    },
    {
      q: 'Is this another inbox I have to maintain?',
      a: 'No. There is nothing extra to file, forward, or sync-manage. Kestrel reads your existing mailboxes and writes back actions (archive, snooze, labels) so everything stays in step on every device.',
    },
    {
      q: 'Does it work offline?',
      a: 'Yes. Threads stay readable offline, and anything you send queues in a local outbox that replays automatically when the connection returns.',
    },
    {
      q: 'Are Mail and Calendar separate downloads?',
      a: 'Yes. Each app ships as its own installer per platform in every tagged release — install one or both, and they share your connected accounts.',
    },
    {
      q: 'How much does it cost?',
      a: 'Kestrel is free and open source. Download the apps, connect your accounts, done — no seat pricing, no subscription.',
    },
  ];
</script>

<svelte:window onkeydown={demoKey} />

<div class="announce">
  {#if version !== ''}Kestrel {version} — {/if}Free mail + calendar client for Gmail &amp; Outlook. <a href="#download">Get the apps →</a>
</div>
<nav class="nav">
  <div class="wrap nav-inner">
    <a class="brand" href="#top">
      <img src="/logo.svg" alt="Kestrel logo" width="28" height="28" />
      Kestrel
    </a>
    <div class="nav-links" class:open={menuOpen}>
      <a href="#features" onclick={() => (menuOpen = false)}>Features</a>
      <a href="#product" onclick={() => (menuOpen = false)}>Product</a>
      <a href="#compare" onclick={() => (menuOpen = false)}>Compare</a>
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
      <Button variant="primary" size="sm" onclick={() => go('#download')}>Download free</Button>
    </div>
  </div>
</nav>

<div class="wrap" id="top">
  <header class="hero">
    <div class="hero-copy">
      <span class="eyebrow"><span class="dot-live"></span>For Gmail &amp; Outlook accounts</span>
      <h1>Your inbox, cleared before standup.</h1>
      <p class="sub">
        Kestrel is a fast, keyboard-driven mail + calendar client for the
        accounts you already have. Connect Gmail and Outlook, triage everything
        in one inbox, and settle meeting times without the thread.
      </p>
      <div class="hero-cta">
        <Button variant="primary" onclick={() => go('#download')}>Download free</Button>
        <a class="ghost-link" href="#demo">Try the 10-second triage ↓</a>
      </div>
      <p class="trust-line">Free &amp; open source · Keep your addresses · No new inbox to maintain</p>
      <div class="hero-accts">
        <ProviderBadge provider="gmail" />
        <ProviderBadge provider="outlook" />
        <span class="hero-accts-note">Connects with OAuth — no password sharing</span>
      </div>
    </div>
    <div class="hero-mock" aria-label="Kestrel Mail preview">
      <div class="mock-bar">
        <span class="mock-dots"><i></i><i></i><i></i></span>
        <span class="mono-dim">kestrel mail — inbox</span>
      </div>
      <div class="mock-body">
        <div class="mock-side">
          <span class="mock-acct"><i class="g"></i>ana@gmail.com</span>
          <span class="mock-acct"><i class="o"></i>ana@outlook.com</span>
          <span class="mock-nav on">Inbox <b>4</b></span>
          <span class="mock-nav">Starred</span>
          <span class="mock-nav">Snoozed</span>
          <span class="mock-nav">Sent</span>
        </div>
        <div class="mock-list">
          <div class="mock-row unread"><i class="bar g"></i><div><b>CI Pipeline</b><span>Release v0.1.0 — all platforms green</span></div><em>09:41</em></div>
          <div class="mock-row unread"><i class="bar o"></i><div><b>Ana Ruiz ★</b><span>Q3 budget review — your sign-off needed</span></div><em>08:15</em></div>
          <div class="mock-row"><i class="bar g"></i><div><b>Ops</b><span>Maintenance window confirmed for Saturday</span></div><em>Tue</em></div>
          <div class="mock-row"><i class="bar o"></i><div><b>Design</b><span>New empty-state mockups</span></div><em>Mon</em></div>
          <div class="mock-keys"><span class="kbd">j</span><span class="kbd">k</span> move · <span class="kbd">e</span> archive · <span class="kbd">s</span> snooze</div>
        </div>
      </div>
    </div>
  </header>

  <div class="stats" aria-label="Kestrel at a glance">
    <span><b>2</b> providers, one inbox</span>
    <span><b>~10s</b> triage loop in the demo</span>
    <span><b>5</b> platforms, native apps</span>
    <span><b>$0</b> — free &amp; open source</span>
  </div>

  <section class="block" id="features">
    <div class="sec-index">Features</div>
    <h2>Everything the default clients make you click through.</h2>
    <p class="sec-sub">
      Six reasons to switch clients without switching addresses. Each one maps
      to a morning you get back.
    </p>
    <ul class="feat">
      {#each features as f}
        <li>
          <span class="feat-ic" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d={iconPaths[f.icon]} /></svg>
          </span>
          <b>{f.title}</b>
          <span>{f.body}</span>
        </li>
      {/each}
    </ul>
  </section>

  <section class="block band" id="product">
    <div class="sec-index">Product tour</div>
    <h2>The actual app, running right here.</h2>
    <p class="sec-sub">
      Not a mockup — these are Kestrel's real ThreadList and WeekGrid components
      with sample data. Switch tabs.
    </p>
    <div class="shot">
      <div class="shot-tabs" role="tablist" aria-label="Product preview">
        <button role="tab" aria-selected={previewTab === 'mail'} class:active={previewTab === 'mail'} onclick={() => (previewTab = 'mail')}>
          Kestrel Mail — thread list
        </button>
        <button role="tab" aria-selected={previewTab === 'calendar'} class:active={previewTab === 'calendar'} onclick={() => (previewTab = 'calendar')}>
          Kestrel Calendar — week view
        </button>
      </div>
      <div class="frame" class:narrow={previewTab === 'mail'}>
        {#if previewTab === 'mail'}
          <ThreadList threads={threads} currentView="inbox" />
        {:else}
          <WeekGrid events={calEvents} viewMode="week" />
        {/if}
      </div>
    </div>
  </section>

  <section class="block" id="demo">
    <div class="sec-index">Try it · Mail</div>
    <h2>Reach inbox zero before standup.</h2>
    <p class="sec-sub">
      Mail's real triage loop with sample data. Select with <span class="kbd">j</span>/<span class="kbd">k</span>,
      archive with <span class="kbd">e</span>, snooze with <span class="kbd">s</span> — or use the buttons.
    </p>
    <div class="demo">
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

  <section class="block band" id="calendar">
    <div class="sec-index">Try it · Calendar</div>
    <h2>Run the day from one view.</h2>
    <p class="sec-sub">
      A week-first calendar over the same connected accounts. Scheduling polls
      collect votes and lock the winner — no five-message chains for thirty minutes.
    </p>
    <div class="shot" style="margin-top: 28px;">
      <div class="shot-bar"><span class="mono-dim">kestrel calendar — month view, live component</span></div>
      <div class="frame">
        <MonthGrid events={monthEvents} />
      </div>
    </div>
    <div class="cal-proof">
      <ProviderBadge provider="outlook" />
      <span>Calendars stay in step with Mail's accounts — connect once, use both.</span>
    </div>
    <div class="connect">
      <div class="connect-step"><div class="step-n">1</div><b>Download the app</b><span>Native apps for Windows, macOS, Linux, Android, and iOS. Install Mail, Calendar, or both.</span></div>
      <div class="connect-step"><div class="step-n">2</div><b>Connect with OAuth</b><span>Sign in to Gmail and Outlook the usual way. No passwords shared, revokable anytime.</span></div>
      <div class="connect-step"><div class="step-n">3</div><b>Triage and schedule</b><span>One inbox, keyboard triage, scheduling polls, offline outbox. Done in minutes.</span></div>
    </div>
  </section>

  <section class="block" id="compare">
    <div class="sec-index">Compare</div>
    <h2>Why not just use the default apps?</h2>
    <p class="sec-sub">Honest version — including where Kestrel loses.</p>
    <div class="cmp-wrap">
      <table class="cmp">
        <thead><tr><th></th><th>Kestrel</th><th>Gmail / Outlook apps</th><th>Superhuman-class</th></tr></thead>
        <tbody>
          <tr><td>Gmail + Outlook in one inbox</td><td class="y">Yes</td><td>Separate apps / tabs</td><td class="y">Yes</td></tr>
          <tr><td>Keyboard triage (j/k/e/s)</td><td class="y">Built in</td><td>Partial / add-ons</td><td class="y">Yes</td></tr>
          <tr><td>Scheduling polls + free/busy</td><td class="y">Built in</td><td>Extensions needed</td><td>Partial</td></tr>
          <tr><td>Offline outbox</td><td class="y">Yes</td><td>Limited</td><td>Limited</td></tr>
          <tr><td>Cost</td><td class="y">Free, open source</td><td>Free with ads / M365 sub</td><td>~$30/mo per seat</td></tr>
          <tr><td>Setup effort</td><td>Install + OAuth</td><td class="y">Already there</td><td class="y">Onboarding call</td></tr>
        </tbody>
      </table>
    </div>
    <p class="sec-sub" style="margin-top: 14px; font-size: 0.9rem;">
      Trade-off, stated plainly: one install buys back speed, offline resilience,
      and freedom from ads. If staying put beats everything, stay put.
    </p>
  </section>

  <section class="block band" id="principles">
    <div class="sec-index">Product principles</div>
    <h2>Decisions we'd defend in an interview.</h2>
    <p class="sec-sub">Every trade-off on this page was a choice. Three that shaped Kestrel:</p>
    <div class="cards">
      <div class="card">
        <div class="card-tag">No migration tax</div>
        <b>Keep your addresses</b>
        <span>Switching clients fails when it demands a new identity. So connect wins over import — history intact, zero setup weekend.</span>
      </div>
      <div class="card">
        <div class="card-tag">Speed is a feature</div>
        <b>Keys over clicks</b>
        <span>Triage is a loop (j/k/e/s), not a page. Ten seconds in the demo above is the acceptance test every inbox change must pass.</span>
      </div>
      <div class="card">
        <div class="card-tag">Network is a detail</div>
        <b>Offline first</b>
        <span>Outbox queues, calendar replays. The app must be useful in a tunnel — connectivity is eventual, work is not.</span>
      </div>
    </div>
  </section>

  <section class="block" id="roadmap">
    <div class="sec-index">Now · Next · Later</div>
    <h2>Shipped, shipping, scoped.</h2>
    <div class="cards">
      <div class="card"><div class="card-tag">Now — v0.1</div><b>Connected triage</b><span>Gmail/Outlook sync, keyboard inbox, polls, native apps on every platform. What you're looking at.</span></div>
      <div class="card"><div class="card-tag">Next</div><b>Send + notify</b><span>Real send via provider APIs, push notifications, auto-update. The un-shipped list is public in the repo roadmap.</span></div>
      <div class="card"><div class="card-tag">Later</div><b>Rich compose + contacts</b><span>Rich-text compose, address book, store-signed mobile builds. Cut until Now/Next land.</span></div>
    </div>
    <p class="sec-sub" style="margin-top: 16px; font-size: 0.9rem;">
      Full status lives in <a href={site.github}>the repo roadmap</a> — done, in-progress, and planned, no marketing fog.
    </p>
  </section>

  <section class="block band" id="download">
    <div class="sec-index">Download</div>
    <h2>Get the apps. Free, forever.</h2>
    <p class="sec-sub">
      Mail and Calendar ship as separate installers in every tagged release. Where a direct
      installer exists, one click starts the download — everything else opens the latest release.
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
              <a class="dl-link primary" href={dlUrl(p.os, 'mail')}>Mail ↓</a>
              <a class="dl-link" href={dlUrl(p.os, 'cal')}>Calendar ↓</a>
            </div>
          {/if}
        </div>
      {/each}
    </div>
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
    <h2>Clear the inbox. Keep the address.</h2>
    <p class="sec-sub">Free, open-source apps for the accounts you already have. Connected in minutes.</p>
    <div class="hero-cta" style="justify-content: center;">
      <Button variant="primary" onclick={() => go('#download')}>Download free</Button>
    </div>
    <p class="trust-line" style="text-align: center;">Free &amp; open source · OAuth only · Works offline</p>
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
        <p>A fast mail + calendar client for your existing Gmail and Outlook accounts. One inbox, keyboard triage, scheduling without threads.</p>
      </div>
      <div class="foot-col">
        <h4>Product</h4>
        <a href="#features">Features</a>
        <a href="#demo">Mail demo</a>
        <a href="#calendar">Calendar</a>
        <a href="#compare">Compare</a>
        <a href="#download">Download</a>
        <a href="#faq">FAQ</a>
      </div>
      <div class="foot-col">
        <h4>Resources</h4>
        <a href={site.github}>GitHub</a>
        <a href={site.releaseNotes}>Release notes</a>
        {#if site.contactEmail !== ''}
          <a href="mailto:{site.contactEmail}">Contact</a>
        {/if}
      </div>
    </div>
    <div class="foot-base">
      <span>Kestrel{#if version !== ''} {version}{/if} · free &amp; open source</span>
      {#if site.buildNotes !== ''}<a href={site.buildNotes}>Build notes</a>{/if}
    </div>
  </div>
</footer>
