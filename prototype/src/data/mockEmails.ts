import { Email } from '../types';

export const mockEmails: Email[] = [
  {
    id: 'email-1',
    sender: 'Vercel Deployment',
    senderEmail: 'noreply@vercel.com',
    to: 'user@workspace.io',
    subject: 'Production Deployment Successful: workspace-dashboard',
    timestamp: '2026-07-18T08:25:00-07:00', // Today
    isUnread: true,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Updates',
    labels: ['Updates [Gmail]', 'DevOps'],
    avatar: 'https://images.unsplash.com/photo-1618401471353-b98aedd07871?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #1a1a1a; color: #e5e2e1; border-radius: 8px;">
        <div style="text-align: center; border-bottom: 1px solid #353534; padding-bottom: 20px; margin-bottom: 20px;">
          <h2 style="font-size: 18px; font-weight: 600; margin-top: 10px; color: #ffffff;">Vercel Deployment</h2>
        </div>
        
        <p style="font-size: 15px; color: #c4c7c8;">Your latest deployment for project <strong>workspace-dashboard</strong> is now live!</p>
        
        <div style="background-color: #111111; border: 1px solid #353534; border-radius: 6px; padding: 16px; margin-bottom: 16px;">
          <div style="font-size: 12px; font-family: monospace; text-transform: uppercase; color: #a0a0a0; margin-bottom: 4px;">Deployment Status</div>
          <h3 style="font-size: 16px; font-weight: 500; margin: 0 0 8px 0; color: #4ade80;">READY (Production)</h3>
          <p style="font-size: 13px; color: #a0a0a0; margin: 0 0 12px 0;">Branch: <strong>main</strong> | Commit: <code>8f7a2d1</code></p>
          <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8; margin: 0 0 16px 0;">
            The deployment was built successfully in 1m 24s. All serverless endpoints are active and routing properly. Domain SSL certificates have been updated.
          </p>
          <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; text-decoration: none; font-size: 13px; font-weight: 500; padding: 8px 16px; border-radius: 4px;">View Deployment</a>
        </div>
        
        <p style="font-size: 11px; color: #a0a0a0; text-align: center; margin-top: 30px; border-top: 1px solid #353534; padding-top: 15px;">
          You can configure notifications and webhooks under your Vercel Project Settings.
        </p>
      </div>
    `
  },
  {
    id: 'email-2',
    sender: 'Sarah Chen (Linear)',
    senderEmail: 'sarah@linear.app',
    to: 'user@workspace.io',
    subject: 'New issue assigned: [ENG-4029] Optimize database query latency',
    timestamp: '2026-07-18T06:12:00-07:00', // Today
    isUnread: true,
    isArchived: false,
    isStarred: true,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Primary',
    labels: ['Inbox [Gmail]', 'Work', 'Urgent'],
    avatar: 'https://images.unsplash.com/photo-1494790108377-be9c29b29330?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <div style="margin-bottom: 20px;">
          <h2 style="font-size: 20px; color: #ffffff; margin-bottom: 4px;">Linear Ticket Update</h2>
          <p style="font-size: 13px; color: #a0a0a0; margin: 0;">Sarah Chen assigned you a high-priority ticket</p>
        </div>
        
        <div style="padding: 16px; background-color: #1c1b1b; border: 1px solid #353534; border-radius: 6px;">
          <h3 style="font-size: 16px; font-weight: 500; color: #ffffff; margin-top: 0;">[ENG-4029] Optimize database query latency</h3>
          <p style="font-size: 14px; font-weight: 500; color: #c4c7c8; margin: 4px 0;">Priority: <strong>High</strong> | Assignee: <strong>Developer</strong></p>
          
          <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8; margin: 12px 0 16px 0;">
            The analytics queries for the billing report are exceeding the 2.5s execution threshold during peak loads. We need to analyze indices on the transactions table and potentially introduce a caching layer.
          </p>
          
          <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; text-decoration: none; font-size: 13px; font-weight: 500; padding: 10px 20px; border-radius: 4px;">Open in Linear</a>
        </div>
      </div>
    `
  },
  {
    id: 'email-3',
    sender: 'GitHub Security',
    senderEmail: 'noreply@github.com',
    to: 'user@workspace.io',
    subject: '[GitHub] Security Alert: vulnerability found in express dependency',
    timestamp: '2026-07-17T17:45:00-07:00', // Yesterday
    isUnread: true,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Updates',
    labels: ['GitHub', 'Urgent'],
    avatar: 'https://images.unsplash.com/photo-1618401471353-b98aedd07871?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: monospace; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #0d0d0d; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <div style="font-size: 18px; color: #ffb4ab; margin-bottom: 8px;">[GitHub Alert] Prototype Pollution in dependency</div>
        <p style="font-size: 13px; color: #c8c6c5;">A high-severity vulnerability has been detected in a dependency of your repository <strong>workspace/dashboard</strong>.</p>
        <table style="width: 100%; border-collapse: collapse; font-size: 12px; margin: 15px 0;">
          <tr style="border-bottom: 1px solid #353534;">
            <td style="padding: 6px 0; color: #a0a0a0;">Vulnerability:</td>
            <td style="padding: 6px 0; color: #ffffff; text-align: right;">CVE-2026-9821</td>
          </tr>
          <tr style="border-bottom: 1px solid #353534;">
            <td style="padding: 6px 0; color: #a0a0a0;">Package:</td>
            <td style="padding: 6px 0; color: #ffffff; text-align: right;">express-body-parser</td>
          </tr>
          <tr style="border-bottom: 1px solid #353534;">
            <td style="padding: 6px 0; color: #a0a0a0;">Severity:</td>
            <td style="padding: 6px 0; color: #ffb4ab; text-align: right; font-weight: bold;">High (8.5)</td>
          </tr>
        </table>
        <p style="font-size: 13px; color: #a0a0a0;">Recommendation: Upgrade the relevant nested dependency version immediately to patch this issue.</p>
        <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; text-decoration: none; font-size: 13px; font-weight: 500; padding: 8px 16px; border-radius: 4px;">View Advisory</a>
      </div>
    `
  },
  {
    id: 'email-4',
    sender: 'Stripe Billing',
    senderEmail: 'billing@stripe.com',
    to: 'user@workspace.io',
    subject: 'Your monthly billing statement is ready - June 2026',
    timestamp: '2026-07-17T09:15:00-07:00', // Yesterday
    isUnread: false,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: true,
    category: 'Updates',
    labels: ['Statements', 'Finance', 'Billing'],
    avatar: 'https://images.unsplash.com/photo-1563013544-824ae1d704d3?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: monospace; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #0d0d0d; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1; line-height: 1.6;">
        <h2 style="font-size: 16px; border-bottom: 1px solid #353534; padding-bottom: 10px; color: #ffffff;">STRIPE BILLING STATEMENT</h2>
        
        <table style="width: 100%; font-size: 12px; margin-bottom: 20px; border-collapse: collapse;">
          <tr>
            <td style="color: #a0a0a0; padding: 4px 0;">INVOICE ID:</td>
            <td style="text-align: right; color: #ffffff;">INV-2026-84920</td>
          </tr>
          <tr>
            <td style="color: #a0a0a0; padding: 4px 0;">STATEMENT DATE:</td>
            <td style="text-align: right; color: #ffffff;">15-JUL-2026</td>
          </tr>
          <tr>
            <td style="color: #a0a0a0; padding: 4px 0;">TOTAL AMOUNT PAID:</td>
            <td style="text-align: right; color: #4ade80; font-weight: bold;">$149.00 USD</td>
          </tr>
          <tr>
            <td style="color: #a0a0a0; padding: 4px 0;">METHOD:</td>
            <td style="text-align: right; color: #ffffff;">Visa ending in 4242</td>
          </tr>
        </table>

        <div style="background-color: #131313; border: 1px solid #353534; padding: 12px; border-radius: 4px; font-size: 12px; margin-bottom: 20px;">
          <span style="font-weight: bold; color: #ffffff;">[ATTACHMENT DETECTED]</span><br/>
          Filename: <span style="color: #c8c6c5;">Stripe_Invoice_INV-2026-84920.pdf</span>
        </div>

        <p style="font-size: 11px; color: #a0a0a0; border-top: 1px solid #353534; padding-top: 10px;">
          This is an auto-generated billing receipt. If you have any inquiries, please head to the Support Center in your developer dashboard.
        </p>
      </div>
    `
  },
  {
    id: 'email-5',
    sender: 'Alex River (Design Team)',
    senderEmail: 'alex@workspace.io',
    to: 'user@workspace.io',
    subject: 'Review requested: Figma design specs for billing dashboard',
    timestamp: '2026-07-16T15:10:00-07:00', // 2 days ago
    isUnread: false,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Primary',
    labels: ['Inbox [Gmail]', 'Work'],
    avatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <h3 style="font-size: 16px; color: #ffffff;">Billing Dashboard Redesign</h3>
        <p style="font-size: 14px; color: #c4c7c8; margin: 0;">Design Specs & Feedback Session</p>
        <p style="font-size: 13px; color: #a0a0a0; margin: 12px 0;">
          Hi Team, I have finished polishing the Figma design files for the new team billing dashboard. It centers simpler invoice list navigation and clearer usage tracking charts. Please leave your comments directly on the specs when you get a chance!
        </p>
        <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; text-decoration: none; font-size: 13px; font-weight: 500; padding: 8px 16px; border-radius: 4px;">Open Figma File</a>
      </div>
    `
  },
  {
    id: 'email-6',
    sender: 'Postgres Cloud',
    senderEmail: 'alerts@postgres-cloud.com',
    to: 'user@workspace.io',
    subject: '[WARNING] Storage usage on production cluster has exceeded 85%',
    timestamp: '2026-07-16T11:40:00-07:00', // 2 days ago
    isUnread: true,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Updates',
    labels: ['Updates [Gmail]', 'Urgent', 'DevOps'],
    avatar: '',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <h2 style="font-size: 18px; color: #ffb4ab; margin-top: 0;">Storage Usage Warning</h2>
        <p style="font-size: 14px; line-height: 1.5; color: #e5e2e1;">
          Your Postgres cluster <strong>production-main-db</strong> is reaching capacity bounds.
        </p>
        <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8;">
          Current disk utilization is at <strong>85.4%</strong>. To prevent transactional write operations from failing or blocking, we strongly recommend enabling storage auto-scaling or upgrading your database instance storage volume.
        </p>
        <div style="margin-top: 20px;">
          <a href="#" style="display: inline-block; background-color: #ffb4ab; color: #690005; font-size: 13px; font-weight: 600; text-decoration: none; padding: 10px 18px; border-radius: 4px;">Upgrade Instance Storage</a>
        </div>
      </div>
    `
  },
  {
    id: 'email-7',
    sender: 'Notion Team',
    senderEmail: 'welcome@notion.so',
    to: 'user@workspace.io',
    subject: 'Welcome to Notion! Connect your workspace in 1-click.',
    timestamp: '2026-07-16T10:05:00-07:00', // 2 days ago
    isUnread: false,
    isArchived: false,
    isStarred: true,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Primary',
    labels: ['Inbox [Gmail]', 'Notion Mail'],
    avatar: 'https://images.unsplash.com/photo-1516321318423-f06f85e504b3?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <div style="text-align: center; margin-bottom: 20px;">
          <span style="font-size: 40px;">✏️</span>
          <h2 style="font-size: 20px; font-weight: 600; color: #ffffff; margin: 10px 0 0 0;">Welcome to Notion</h2>
        </div>
        <p style="font-size: 14px; line-height: 1.6; color: #c4c7c8;">
          Welcome to Notion—your shared team workspace designed specifically for documentation, product plans, and knowledge bases.
        </p>
        <ul style="font-size: 13px; line-height: 1.6; color: #c4c7c8; padding-left: 20px;">
          <li><strong>Wikis & Docs:</strong> Real-time editing and clear page organization hierarchies.</li>
          <li><strong>Keyboard First:</strong> Press <kbd style="background-color: #353534; padding: 2px 5px; border-radius: 3px; font-size: 11px;">/</kbd> to invoke commands and insert media.</li>
          <li><strong>App integrations:</strong> Sync your development cards and task boards natively.</li>
        </ul>
        <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; font-size: 13px; font-weight: 500; text-decoration: none; padding: 10px 18px; border-radius: 4px; margin-top: 15px;">Launch Getting Started Guide</a>
      </div>
    `
  },
  {
    id: 'email-8',
    sender: 'Slack Notifications',
    senderEmail: 'notification@slack.com',
    to: 'user@workspace.io',
    subject: 'Weekly digest: unread mentions in #engineering',
    timestamp: '2026-07-15T14:30:00-07:00', // 3 days ago
    isUnread: false,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Social',
    labels: ['Inbox [Gmail]'],
    avatar: 'https://images.unsplash.com/photo-1563986768609-322da13575f3?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <h3 style="font-size: 16px; color: #ffffff;">Slack Notification Summary</h3>
        <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8;">
          You have unread mentions in your workspace:
        </p>
        <div style="background-color: #1c1b1b; border: 1px solid #353534; border-radius: 6px; padding: 12px; margin-top: 10px;">
          <span style="font-weight: 600; font-size: 13px; color: #ffffff;">#engineering</span>
          <p style="font-size: 12px; color: #a0a0a0; margin: 4px 0 0 0;">"Are we good to merge the main API router test specs before tomorrow's staging release?"</p>
        </div>
        <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; text-decoration: none; font-size: 13px; font-weight: 500; padding: 8px 16px; border-radius: 4px; margin-top: 15px;">Open Slack Workspace</a>
      </div>
    `
  },
  {
    id: 'email-9',
    sender: 'GitHub',
    senderEmail: 'noreply@github.com',
    to: 'user@workspace.io',
    subject: '[GitHub] Pull Request #145: Add end-to-end integration tests',
    timestamp: '2026-07-14T11:20:00-07:00', // 4 days ago
    isUnread: false,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Primary',
    labels: ['GitHub'],
    avatar: 'https://images.unsplash.com/photo-1618401471353-b98aedd07871?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <h4 style="margin: 0; font-size: 14px; color: #a0a0a0;">workspace/dashboard</h4>
        <h3 style="margin: 8px 0; font-size: 16px; color: #ffffff;">Pull Request #145: Add end-to-end integration tests</h3>
        <p style="font-size: 13px; color: #c4c7c8;"><strong>reviewer</strong> approved these changes:</p>
        <div style="background-color: #0d0d0d; border-left: 3px solid #ffffff; padding: 10px 15px; margin: 15px 0; font-family: monospace; font-size: 13px; border-radius: 0 4px 4px 0;">
          "The integration suite is well organized and mocks network failures beautifully. Let's merge!"
        </div>
        <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; font-size: 13px; font-weight: 500; text-decoration: none; padding: 8px 16px; border-radius: 4px;">Merge Pull Request</a>
      </div>
    `
  },
  {
    id: 'email-10',
    sender: 'SaaS Weekly',
    senderEmail: 'newsletter@saasweekly.com',
    to: 'user@workspace.io',
    subject: 'SaaS Weekly #384: The rise of local-first web applications',
    timestamp: '2026-07-13T16:50:00-07:00', // 5 days ago
    isUnread: false,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Promotions',
    labels: ['Updates [Gmail]'],
    avatar: '',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <h3 style="font-size: 16px; color: #ffffff;">SaaS Weekly #384</h3>
        <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8;">
          This week we explore why local-first architecture is gaining dramatic adoption among top SaaS creators. We dive into sync engines, operational transformation logic, and standard offline database patterns.
        </p>
        <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; text-decoration: none; font-size: 13px; font-weight: 500; padding: 8px 16px; border-radius: 4px;">Read Full Issue</a>
      </div>
    `
  },
  {
    id: 'email-11',
    sender: 'AWS Alerts',
    senderEmail: 'no-reply@amazon.com',
    to: 'user@workspace.io',
    subject: 'AWS Billing Alert: Monthly budget threshold exceeded',
    timestamp: '2026-07-13T14:15:00-07:00', // 5 days ago
    isUnread: false,
    isArchived: false,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Updates',
    labels: ['Billing', 'Finance'],
    avatar: '',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <h2 style="font-size: 18px; color: #ffffff; margin-top: 0;">AWS Budget Alert</h2>
        <p style="font-size: 14px; line-height: 1.5; color: #e5e2e1;">
          You have exceeded 80% of your configured monthly AWS budget.
        </p>
        <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8;">
          Current cost projections estimate an end-of-month spend of $120.00 USD, surpassing your budget limit of $100.00 USD.
        </p>
        <a href="#" style="display: inline-block; background-color: #ffffff; color: #0d0d0d; text-decoration: none; font-size: 13px; font-weight: 500; padding: 8px 16px; border-radius: 4px;">Open AWS Cost Console</a>
      </div>
    `
  },
  {
    id: 'email-12',
    sender: 'Vercel Support',
    senderEmail: 'support@vercel.com',
    to: 'user@workspace.io',
    subject: 'Re: [Ticket #84291] Custom domain SSL renewal failure',
    timestamp: '2026-07-13T09:10:00-07:00', // 5 days ago
    isUnread: false,
    isArchived: true,
    isStarred: false,
    isDraft: false,
    isSpam: false,
    isTrash: false,
    hasAttachment: false,
    category: 'Primary',
    labels: ['Replied', 'Work'],
    avatar: 'https://images.unsplash.com/photo-1618401471353-b98aedd07871?auto=format&fit=crop&w=100&h=100&q=80',
    body: `
      <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #131313; border: 1px solid #353534; border-radius: 8px; color: #e5e2e1;">
        <p style="font-size: 14px; line-height: 1.5; color: #e5e2e1;">
          Hi Developer,
        </p>
        <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8;">
          We have reviewed your request regarding Ticket #84291 and successfully triggered manual certification renewal on our edge CDN nodes. The DNS entries are now resolving correctly over HTTPS.
        </p>
        <p style="font-size: 13px; line-height: 1.5; color: #c4c7c8;">
          Please let us know if you experience any further handshake latency!
        </p>
        <p style="font-size: 13px; line-height: 1.5; color: #a0a0a0;">
          Best regards,<br/>Vercel Support Team
        </p>
      </div>
    `
  }
];
