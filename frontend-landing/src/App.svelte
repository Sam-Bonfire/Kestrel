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
      snippet: 'Sync pauses 02:00–02:30 UTC. Outbox queue holds all sends automatically.',
      provider: 'gmail',
      date: 'Tue',
    },
  ];

  interface EventSample {
    time: string;
    title: string;
    sub: string;
  }

  const events: EventSample[] = [
    { time: '09:30 – 09:45', title: 'Standup', sub: 'Engineering · video link attached' },
    { time: '14:00 – 15:00', title: 'Design review', sub: '4 attendees · recap auto-filed to thread' },
    { time: '16:30 – 17:00', title: 'Release sign-off', sub: 'Go / no-go for the weekend cut' },
  ];

  const processSteps = [
    {
      n: '01',
      title: 'Frame the problem',
      body: 'Start from observed behavior, not feature requests. Define the smallest decision this work must improve.',
    },
    {
      n: '02',
      title: 'Place a small bet',
      body: 'Scope the thinnest slice that tests the riskiest assumption. Write the PRD around trade-offs, not just scope.',
    },
    {
      n: '03',
      title: 'Ship the slice',
      body: 'Build with the team, keep the diff reviewable, and put it in front of users behind existing flows.',
    },
    {
      n: '04',
      title: 'Measure, then decide',
      body: 'One metric per bet, decided up front. Double down, reshape, or kill — explicitly, in writing.',
    },
  ];

  const stackFacts = [
    { k: 'Backend', v: 'Rust + Axum on Tokio, SQLite or PostgreSQL' },
    { k: 'Clients', v: 'Svelte 5 + Tailwind, shipped via Tauri v2' },
    { k: 'Providers', v: 'Sandboxed WASM plugins for Gmail and Outlook' },
    { k: 'Sync', v: 'Background sync daemon, offline outbox queue' },
    { k: 'Deploy', v: 'Single Docker image on your server or NAS' },
    { k: 'Access', v: 'Cloudflare Tunnel (public) or Tailscale (private)' },
  ];

  const skills = [
    'Roadmapping',
    'PRDs & specs',
    'User discovery',
    'Prioritization',
    'Experimentation',
    'Analytics',
    'Go-to-market',
    'Stakeholder leadership',
  ];
</script>

<nav class="nav">
  <div class="wrap nav-inner">
    <a class="brand" href="#top">
      <img src="/logo.svg" alt="Kestrel logo" width="28" height="28" />
      Kestrel
    </a>
    <div class="nav-links">
      <a href="#work">Work</a>
      <a href="#process">Process</a>
      <a href="#about">About</a>
      <a href="#download">Download</a>
    </div>
    <div class="nav-actions">
      <button class="theme-btn" onclick={toggleTheme} aria-label="Toggle color theme">
        {theme === 'light' ? '◑' : '◐'}
      </button>
      <Button variant="secondary" size="sm" onclick={() => go('#download')}>Get the apps</Button>
    </div>
  </div>
</nav>

<div class="wrap" id="top">
  <header class="hero">
    <span class="eyebrow">{site.role} · Mail &amp; Calendar</span>
    <h1>I turn inbox and calendar chaos into calm software.</h1>
    <p class="sub">
      I&rsquo;m {site.name}, a product manager. Kestrel is my end-to-end portfolio piece: a
      private, self-hosted mail and calendar suite. I set the strategy, wrote the specs, and
      shipped it with the team — strategy through release, documented below.
    </p>
    <div class="hero-cta">
      <Button variant="primary" onclick={() => go('#work')}>View selected work</Button>
      <Button variant="secondary" onclick={() => go('#download')}>Download the apps</Button>
    </div>
    <div class="hero-facts">
      <span>Rust backend</span>
      <span>Svelte clients</span>
      <span>6 platforms</span>
      <span>Self-hosted</span>
    </div>
  </header>

  <section class="block" id="work">
    <div class="sec-index">01 — Selected work</div>
    <h2>Two products, one system.</h2>
    <p class="sec-sub">
      Kestrel Mail and Kestrel Calendar share one backend, one component library, and one set
      of product principles. Each case below states the problem, the calls I made, and what
      shipped. The panels render the apps&rsquo; real shared components with sample data.
    </p>

    <article class="case">
      <div class="case-head">
        <h3>Kestrel Mail — triage at keyboard speed</h3>
        <span class="case-meta">Role: PM · Scope: inbox, threading, offline</span>
      </div>
      <div class="case-grid">
        <div>
          <h4>Problem</h4>
          <p>
            Knowledge workers lose the first hour of the day to triage. Webmail is click-heavy,
            and every provider behaves differently, so habits never transfer.
          </p>
        </div>
        <div>
          <h4>Decisions</h4>
          <ul>
            <li><strong>Keyboard-first list:</strong> every triage action reachable without a pointer.</li>
            <li><strong>One component library:</strong> Mail and Calendar share primitives, so behavior stays consistent.</li>
            <li><strong>Offline outbox:</strong> sends queue locally and replay — no lost drafts on bad networks.</li>
          </ul>
        </div>
        <div>
          <h4>Shipped</h4>
          <p>
            Shortcut system with on-screen cheat sheet, cross-provider labels, and an offline
            outbox with retry. Full client UI complete; provider sync via WASM plugins.
          </p>
        </div>
      </div>
      <div class="showcase">
        <div class="showcase-bar">kestrel mail — inbox (live shared components)</div>
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
        <div class="showcase-note">LabelPill + ProviderBadge from @kestrel/shared, same build the apps use</div>
      </div>
    </article>

    <article class="case">
      <div class="case-head">
        <h3>Kestrel Calendar — scheduling without the thread</h3>
        <span class="case-meta">Role: PM · Scope: week/day/month, polls, availability</span>
      </div>
      <div class="case-grid">
        <div>
          <h4>Problem</h4>
          <p>
            Scheduling still happens over email: five messages to find thirty minutes. Availability
            lives in people&rsquo;s heads instead of the tool.
          </p>
        </div>
        <div>
          <h4>Decisions</h4>
          <ul>
            <li><strong>Events link to threads:</strong> every meeting traces back to the mail that caused it.</li>
            <li><strong>Polls over threads:</strong> time proposals collect votes instead of replies.</li>
            <li><strong>Free/busy first:</strong> availability query before any invite goes out.</li>
          </ul>
        </div>
        <div>
          <h4>Shipped</h4>
          <p>
            Week, day, and month views with event creation, polls with voting, and team
            availability overlays across Google and Outlook calendars.
          </p>
        </div>
      </div>
      <div class="showcase">
        <div class="showcase-bar">kestrel calendar — today (live shared components)</div>
        <div class="day">
          {#each events as e}
            <div class="evt">
              <span class="evt-time">{e.time}</span>
              <div>
                <div class="evt-title">{e.title}</div>
                <div class="evt-sub">{e.sub}</div>
              </div>
            </div>
          {/each}
          <div style="margin-top: 10px; display: flex; gap: 8px; align-items: center;">
            <ProviderBadge provider="outlook" />
            <LabelPill tag="devops" label="team overlay on" />
          </div>
        </div>
        <div class="showcase-note">ProviderBadge + LabelPill from @kestrel/shared, same build the apps use</div>
      </div>
    </article>
  </section>

  <section class="block" id="process">
    <div class="sec-index">02 — How I work</div>
    <h2>Small bets, written down.</h2>
    <p class="sec-sub">The operating loop behind both apps — boring on purpose, so surprises are cheap.</p>
    <div class="steps">
      {#each processSteps as s}
        <div class="step">
          <div class="step-num">{s.n}</div>
          <h3>{s.title}</h3>
          <p>{s.body}</p>
        </div>
      {/each}
    </div>
  </section>

  <section class="block" id="stack">
    <div class="sec-index">03 — Under the hood</div>
    <h2>Lightweight by design.</h2>
    <p class="sec-sub">
      Premium feel, minimal footprint: your data lives on your server, and this page is served
      by that same backend — no separate hosting.
    </p>
    <dl class="stack">
      {#each stackFacts as f}
        <div class="stack-item">
          <dt>{f.k}</dt>
          <dd>{f.v}</dd>
        </div>
      {/each}
    </dl>
    <span class="status">
      <span class="dot" class:on={serverOnline === true}></span>
      {#if serverOnline === null}Checking server status…
      {:else if serverOnline}Backend online — downloads below come from this server&rsquo;s release
      {:else}Offline preview — connect to a running backend for live status{/if}
    </span>
  </section>

  <section class="block" id="about">
    <div class="sec-index">04 — About</div>
    <h2>Product manager who ships.</h2>
    <div class="about">
      <div>
        <p>
          I work across the full product surface: discovery and positioning, specs the team can
          build from, launch checklists, and the metrics review after. Kestrel is where I
          practice that end to end — I own the roadmap, the trade-off calls, and the release.
        </p>
        <p>
          Before Kestrel: cross-functional teams, B2B and consumer workflows, and a bias for
          writing things down. The best way to evaluate me is the product above — install it,
          break it, and tell me what you find.
        </p>
      </div>
      <ul class="skills">
        {#each skills as s}
          <li>{s}</li>
        {/each}
      </ul>
    </div>
  </section>

  <section class="block" id="download">
    <div class="sec-index">05 — Download</div>
    <h2>Get the latest builds.</h2>
    <p class="sec-sub">
      Every tagged release ships Mail and Calendar installers for all platforms, plus the server
      image. This page is served by your Kestrel backend; the links below open the latest
      GitHub release.
    </p>
    <div class="dl">
      {#each platforms as p}
        <div class="dl-row">
          <div>
            <span class="dl-os">{p.os}</span>
            <span class="dl-format">{p.format}</span>
            <div class="dl-note">{p.note}</div>
          </div>
          <a class="dl-link" href={site.repoReleases}>Download</a>
        </div>
      {/each}
      <div class="dl-row">
        <div>
          <span class="dl-os">Server</span>
          <span class="dl-format">Docker</span>
          <div class="dl-note">Self-host the backend that serves this page</div>
        </div>
        <a class="dl-link" href={site.repoReleases}>Release notes</a>
      </div>
    </div>
    <div class="code">docker pull {site.dockerImage}</div>
  </section>
</div>

<footer>
  <div class="wrap foot">
    <span>Kestrel · {site.name} · {site.role}</span>
    <span>
      <a href="mailto:{site.email}">Email</a>
      &nbsp;·&nbsp;
      <a href={site.github}>GitHub</a>
      &nbsp;·&nbsp;
      <a href={site.linkedin}>LinkedIn</a>
      &nbsp;·&nbsp;
      <a href="/api/health">API status</a>
    </span>
  </div>
</footer>
