<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '@kestrel/shared/components/Button.svelte';
  import ProviderBadge from '@kestrel/shared/components/ProviderBadge.svelte';
  import ThreadList from 'frontend-mail/src/lib/components/ThreadList.svelte';
  import WeekGrid from 'frontend-calendar/src/lib/components/WeekGrid.svelte';
  import MonthGrid from 'frontend-calendar/src/lib/components/MonthGrid.svelte';
  import { site, platforms } from './site.js';

  let previewTab: 'mail' | 'calendar' = $state('mail');
  let menuOpen: boolean = $state(false);
  let activeSection: string = $state('');
  let navScrolled: boolean = $state(false);

  const version: string = __KESTREL_VERSION__;
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

  // True only when a direct installer asset was resolved — otherwise the
  // button points at the release page and must not claim a direct download.
  function isDirect(os: string, app: 'mail' | 'cal'): boolean {
    const prefix = os === 'Windows' ? 'win' : os === 'Linux' ? 'linux' : null;
    if (!prefix) return false;
    return `${prefix}-${app}` in dl;
  }

  let stars: number | null = $state(null);

  function fmtStars(n: number): string {
    return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : `${n}`;
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
        /* offline — release-page links stay */
      });
  }

  function go(url: string): void {
    window.location.href = url;
  }

  onMount(() => {
    // Landing is dark-only (the apps have no light theme yet).
    document.documentElement.dataset.theme = 'dark';
    try {
      localStorage.setItem('kestrel-theme', 'dark');
    } catch {
      /* storage unavailable */
    }
    resolveDownloads();
    fetch(`https://api.github.com/repos/${site.repo}`)
      .then((r) => (r.ok ? r.json() : null))
      .then((repo: { stargazers_count?: number } | null) => {
        if (repo && typeof repo.stargazers_count === 'number') stars = repo.stargazers_count;
      })
      .catch(() => {
        /* offline — plain GitHub link stays */
      });

    // Page-wide wheel routing — this landing page has NO legitimate inner
    // scrolling (every embed and table is a showcase), so every wheel
    // scrolls the page. Per-element chaining heuristics proved untrustworthy
    // across builds (overflow clip/hidden + ancestor overscroll-behavior +
    // nested app scroll regions), and traps appeared even over plain rows.
    // One path for the whole page also fixes the speed mismatch: deltas
    // accumulate and flush once per animation frame. Untouched: clicks,
    // keyboard, pinch-zoom, and horizontal pans inside the compare table.
    let fX = 0;
    let fY = 0;
    let fQueued = false;
    const flushWheel = (): void => {
      fQueued = false;
      if (fX === 0 && fY === 0) return;
      window.scrollBy({ top: fY, left: fX, behavior: 'instant' as ScrollBehavior });
      fX = 0;
      fY = 0;
    };
    const queueWheel = (dx: number, dy: number, mode: number): void => {
      const unit = mode === 1 ? 16 : mode === 2 ? window.innerHeight : 1;
      fX += dx * unit;
      fY += dy * unit;
      if (!fQueued) {
        fQueued = true;
        requestAnimationFrame(flushWheel);
      }
    };
    document.addEventListener(
      'wheel',
      ((e: WheelEvent): void => {
        if (e.ctrlKey) return; // pinch-zoom: let the browser handle it
        const t = e.target as HTMLElement | null;
        const inTable = t?.closest?.('.cmp-wrap');
        if (inTable && Math.abs(e.deltaX) > Math.abs(e.deltaY)) return; // pan the table
        e.preventDefault();
        queueWheel(e.deltaX, e.deltaY, e.deltaMode);
      }) as EventListener,
      { passive: false },
    );

    // Scrollspy + condensed nav + hero parallax — one rAF-throttled listener.
    let ticking = false;
    const onScroll = (): void => {
      if (ticking) return;
      ticking = true;
      requestAnimationFrame(() => {
        navScrolled = window.scrollY > 8;
        // Gentle hero parallax (transform; entrance uses `translate`, so they compose).
        if (!window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
          const mock = document.querySelector('.hero-mock') as HTMLElement | null;
          if (mock) mock.style.transform = `translateY(${Math.min(window.scrollY * 0.09, 140)}px)`;
        }
        ticking = false;
      });
    };
    window.addEventListener('scroll', onScroll, { passive: true });
    onScroll();

    // Scrollspy — highlights the nav link for the section in view.
    if ('IntersectionObserver' in window) {
      const spy = new IntersectionObserver(
        (entries) => {
          for (const e of entries) {
            if (e.isIntersecting) activeSection = (e.target as HTMLElement).id;
          }
        },
        { rootMargin: '-40% 0px -55% 0px' },
      );
      document.querySelectorAll('section.block').forEach((s) => spy.observe(s));
    }
    const els = document.querySelectorAll(
      '.hero-mock, .stats, .feat li, .shot, .connect-step, .dl, .faq details, .final',
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

  // Anchor week: local Monday 09:00, passed explicitly to WeekGrid so the
  // previewed week and the sample events always coincide (viewer-date
  // and timezone independent — never UTC-shifted out of the visible week).
  function localISO(d: Date): string {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  const weekMonday: Date = (() => {
    const d = new Date();
    d.setDate(d.getDate() - ((d.getDay() + 6) % 7));
    d.setHours(9, 0, 0, 0);
    return d;
  })();

  function dayStr(offset: number): string {
    const d = new Date(weekMonday);
    d.setDate(d.getDate() + offset);
    return localISO(d);
  }

  const threads = [
    { id: 't1', sender: 'CI Pipeline', senderEmail: 'ci@example.com', subject: 'Release v0.1.0 — all green', snippet: 'All builds passed. Docker image pushed.', date: '09:41', isUnread: true, isStarred: false, hasAttachment: false, labels: ['devops'] },
    { id: 't2', sender: 'Ana Ruiz', senderEmail: 'ana@example.com', subject: 'Q3 budget review — sign-off needed', snippet: 'Sheet attached, deadline Friday.', date: '08:15', isUnread: true, isStarred: true, hasAttachment: true, labels: ['finance'] },
    { id: 't3', sender: 'Ops', senderEmail: 'ops@example.com', subject: 'Maintenance window Saturday', snippet: 'Sync pauses 02:00–02:30 UTC. Outbox holds sends.', date: 'Tue', isUnread: false, isStarred: false, hasAttachment: false, labels: ['urgent'] },
    { id: 't4', sender: 'Design', senderEmail: 'design@example.com', subject: 'New empty-state mockups', snippet: 'Three directions. Vote by EOD.', date: 'Tue', isUnread: false, isStarred: false, hasAttachment: true, labels: [] },
  ];

  const calEvents = [
    { id: 'e1', title: 'Sprint planning', date: dayStr(0), startTime: '10:00', endTime: '10:30', color: 'blue', calendarId: 'work' },
    { id: 'e2', title: 'Standup', date: dayStr(1), startTime: '09:30', endTime: '09:45', color: 'rose', calendarId: 'work' },
    { id: 'e3', title: 'Design review', date: dayStr(1), startTime: '14:00', endTime: '15:00', color: 'purple', calendarId: 'work' },
    { id: 'e4', title: '1:1 with Ana', date: dayStr(2), startTime: '11:00', endTime: '11:30', color: 'green', calendarId: 'work' },
    { id: 'e5', title: 'Release cut', date: dayStr(4), startTime: '15:00', endTime: '15:30', color: 'amber', calendarId: 'work' },
  ] as { id: string; title: string; date: string; startTime: string; endTime: string; color: string; calendarId: string }[];

  // MonthGrid matches events by weekday, so entries repeat weekly by design.
  // Keep three that read as genuine recurring meetings instead of five
  // identical rows that scream placeholder.
  const monthEvents = [
    { ...calEvents[0], title: 'Sprint planning', color: '#0078D4', dayIndex: 1 },
    { ...calEvents[3], title: '1:1 with Ana', color: '#c084fc', dayIndex: 2 },
    { ...calEvents[2], title: 'Design review', color: '#34d399', dayIndex: 4 },
  ];

  const features = [
    { icon: 'inbox', title: 'One inbox, every account', body: 'Starting with Gmail and Outlook, more on the way. No forwarding rules.' },
    { icon: 'kbd', title: 'Keyboard-first triage', body: 'The full loop without the mouse. Seconds, not minutes.' },
    { icon: 'cal', title: 'Scheduling without threads', body: 'Polls over real free/busy. Votes lock the winner.' },
    { icon: 'off', title: 'Offline outbox', body: 'Queues locally, replays on reconnect. Zero lost sends.' },
  ];

  const iconPaths: Record<string, string> = {
    inbox: 'M22 12h-6l-2 3h-4l-2-3H2 M5.5 5.1 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.5-6.9A2 2 0 0 0 16.7 4H7.3a2 2 0 0 0-1.8 1.1z',
    kbd: 'M4 17V7a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm5-6h1m2 0h1m2 0h1m-8 4h8',
    cal: 'M8 2v4m8-4v4M3 9h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z',
    off: 'M8.7 16.7 3 12l9-9 5.5 5.5M5 5l16 16M17 8.5V7h1.5M22 12l-3-3m-3.5 10.5c-1 .5-2.4.8-3.5.8a8 8 0 0 1-8-8c0-1.1.3-2.5.8-3.5',
  };

  const faqs = [
    { q: 'Do I need a new email address?', a: 'No. Connect what you have — starting with Gmail and Outlook. History stays put.' },
    { q: 'How do accounts connect?', a: 'OAuth, starting with Google and Microsoft. No passwords, revokable anytime.' },
    { q: 'Does it work offline?', a: 'Yes. Reading works offline; sends queue and replay on reconnect.' },
    { q: 'Mail and Calendar — one download?', a: 'Separate installers, shared accounts. Install one or both.' },
    { q: 'How much?', a: 'Free and open source (MIT). No seats, no subscription.' },
    { q: 'How is it free — what is the catch?', a: 'There is no paid tier and no Kestrel cloud account. The apps connect to your providers directly; the Docker sync server is optional, for those who want mail flowing through their own hardware.' },
  ];
</script>

<div class="announce">
  {#if version !== ''}Kestrel {version} — {/if}Free, private mail + calendar for your accounts. <a href="#download">Get the apps →</a>
</div>

<nav class="nav" class:scrolled={navScrolled}>
  <div class="wrap nav-inner">
    <a class="brand" href="#top"><img src="/logo.svg" alt="Kestrel logo" width="28" height="28" />Kestrel</a>
    <div class="nav-links" class:open={menuOpen}>
      <a href="#features" class:active={activeSection === 'features'} onclick={() => (menuOpen = false)}>Features</a>
      <a href="#product" class:active={activeSection === 'product'} onclick={() => (menuOpen = false)}>Product</a>
      <a href="#compare" class:active={activeSection === 'compare'} onclick={() => (menuOpen = false)}>Compare</a>
      <a href="#download" class:active={activeSection === 'download'} onclick={() => (menuOpen = false)}>Download</a>
      <a href="#faq" class:active={activeSection === 'faq'} onclick={() => (menuOpen = false)}>FAQ</a>
    </div>
    <div class="nav-actions">
      <button class="nav-toggle" onclick={() => (menuOpen = !menuOpen)} aria-label="Menu" aria-expanded={menuOpen}>{menuOpen ? '✕' : '☰'}</button>
      <a class="nav-github" href={site.github}>GitHub{#if stars !== null && stars > 0} · ★ {fmtStars(stars)}{/if}</a>
      <Button variant="primary" size="sm" onclick={() => go('#download')}>Download free</Button>
    </div>
  </div>
</nav>

<div class="wrap" id="top">
  <header class="hero">
    <div class="hero-copy">
      <span class="eyebrow"><span class="dot-live"></span>For your existing accounts</span>
      <h1>Own your inbox.</h1>
      <p class="sub">
        A fast, keyboard-driven mail + calendar client for the
        accounts you already have. Private by design, self-hostable
        if you want it — free and open source.
      </p>
      <div class="hero-cta">
        <Button variant="primary" onclick={() => go('#download')}>Download free</Button>
        <a class="ghost-link" href="#product">See it live ↓</a>
      </div>
      <p class="trust-line">Free &amp; open source · Private by design · Keep your addresses</p>
      <div class="hero-accts">
        <ProviderBadge provider="gmail" />
        <ProviderBadge provider="outlook" />
        <span class="hero-accts-note">Connects with OAuth — more providers on the way</span>
      </div>
    </div>
    <div class="hero-mock" aria-label="Kestrel Mail preview">
      <div class="hero-frame">
        <div inert><ThreadList threads={threads} currentView="inbox" /></div>
      </div>
    </div>
  </header>

  <div class="stats" aria-label="Kestrel at a glance">
    <span><b>One</b> inbox for every account</span>
    <span><b>Keyboard-first</b> triage loop</span>
    <span><b>Private</b> by design, yours to host</span>
    <span><b>$0</b> — free &amp; open source</span>
  </div>

  <section class="block" id="features">
    <div class="sec-index">Features</div>
    <h2>Less clicking. More done.</h2>
    <p class="sec-sub">Four reasons to switch clients without switching addresses.</p>
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
    <h2>The actual app, live.</h2>
    <p class="sec-sub">Real components. Switch tabs.</p>
    <div class="shot">
      <div class="shot-tabs" role="tablist" aria-label="Product preview">
        <button role="tab" aria-selected={previewTab === 'mail'} class:active={previewTab === 'mail'} onclick={() => (previewTab = 'mail')}>Kestrel Mail — thread list</button>
        <button role="tab" aria-selected={previewTab === 'calendar'} class:active={previewTab === 'calendar'} onclick={() => (previewTab = 'calendar')}>Kestrel Calendar — week view</button>
      </div>
      <div class="frame" class:narrow={previewTab === 'mail'}>
        {#key previewTab}
          <div class="frame-fade" inert>
            {#if previewTab === 'mail'}
              <ThreadList threads={threads} currentView="inbox" />
            {:else}
              <WeekGrid events={calEvents} viewMode="week" selectedDate={weekMonday} />
            {/if}
          </div>
        {/key}
      </div>
    </div>
  </section>

  <section class="block band" id="calendar">
    <div class="sec-index">Calendar</div>
    <h2>One view for the day.</h2>
    <p class="sec-sub">Polls collect votes and lock the winner. No long threads.</p>
    <div class="shot" style="margin-top: 56px;">
      <div class="shot-bar"><span class="mono-dim">Month view</span></div>
      <div class="frame"><div inert><MonthGrid events={monthEvents} /></div></div>
    </div>
    <div class="cal-proof">
      <ProviderBadge provider="outlook" />
      <span>Connect once, use both.</span>
    </div>
    <div class="connect">
      <div class="connect-step"><div class="step-n">1</div><b>Download</b><span>Mail, Calendar, or both.</span></div>
      <div class="connect-step"><div class="step-n">2</div><b>Connect</b><span>OAuth, starting with Gmail and Outlook.</span></div>
      <div class="connect-step"><div class="step-n">3</div><b>Done</b><span>Triage and schedule in minutes.</span></div>
    </div>
  </section>

  <section class="block" id="compare">
    <div class="sec-index">Compare</div>
    <h2>Why switch?</h2>
    <p class="sec-sub">The honest version.</p>
    <div class="cmp-wrap">
      <table class="cmp">
        <thead><tr><th></th><th>Kestrel</th><th>Default apps</th><th>Superhuman-class</th></tr></thead>
        <tbody>
          <tr><td>Unified inbox</td><td class="y">Yes</td><td>Switch between accounts</td><td class="y">Yes</td></tr>
          <tr><td>Keyboard-first</td><td class="y">Built in</td><td>Partial</td><td class="y">Yes</td></tr>
          <tr><td>Scheduling polls</td><td class="y">Built in</td><td>Extensions</td><td>Partial</td></tr>
          <tr><td>Cost</td><td class="y">Free</td><td>Free / sub</td><td>~$30/mo</td></tr>
        </tbody>
      </table>
    </div>
  </section>

  <section class="block band" id="download">
    <div class="sec-index">Download</div>
    <h2>Get the apps. Free.</h2>
    <p class="sec-sub">Separate Mail and Calendar installers, every release.</p>
    <div class="dl">
      {#each platforms as p}
        <div class="dl-row">
          <div><span class="dl-os">{p.os}</span><span class="dl-format">{p.format}</span></div>
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
              <a class="dl-link primary" href={dlUrl(p.os, 'mail')}>{isDirect(p.os, 'mail') ? 'Mail ↓' : 'Mail →'}</a>
              <a class="dl-link" href={dlUrl(p.os, 'cal')}>{isDirect(p.os, 'cal') ? 'Calendar ↓' : 'Calendar →'}</a>
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
    <p class="sec-sub">Free, open source, connected in minutes.</p>
    <div class="hero-cta" style="justify-content: center;">
      <Button variant="primary" onclick={() => go('#download')}>Download free</Button>
    </div>
  </section>
</div>

<footer>
  <div class="wrap">
    <div class="foot-grid">
      <div class="foot-brand">
        <a class="brand" href="#top"><img src="/logo.svg" alt="Kestrel logo" width="28" height="28" />Kestrel</a>
        <p>Mail + calendar for the accounts you have. One inbox, keyboard-first.</p>
      </div>
      <div class="foot-col">
        <h4>Product</h4>
        <a href="#features">Features</a>
        <a href="#product">Product</a>
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
