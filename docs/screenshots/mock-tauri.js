// Stand-in for the Tauri API with demo data, so the pages can be rendered in a
// plain browser for screenshots. Loaded before the page script by make.py.
// The URL hash selects the scenario, see SCENARIOS at the bottom.
;(() => {
  const ALL = '*'
  // Catalog language and dates follow the browser; keep the demo in English.
  Object.defineProperty(navigator, 'language', { get: () => 'en-US' })
  const now = Date.now()
  const accounts = [
    { id: 'acc-gmail', label: 'Gmail', server: 'imap.gmail.com', port: 993, security: 'ssl', allow_invalid_certs: false, user: 'alex.doe@gmail.com', mailbox: 'INBOX', tag: 'gmail' },
    { id: 'acc-work', label: 'Work', server: 'imap.example-corp.com', port: 993, security: 'ssl', allow_invalid_certs: false, user: 'a.doe@example-corp.com', mailbox: 'INBOX', tag: 'work' },
    { id: 'acc-proton', label: 'Proton', server: '127.0.0.1', port: 1143, security: 'starttls', allow_invalid_certs: true, user: 'alex.doe@proton.me', mailbox: 'INBOX', tag: 'proton' },
  ]
  const statuses = [
    { account_id: 'acc-gmail', state: 'ok', message: 'Connected – watching INBOX', timestamp: now / 1000 - 3600 },
    { account_id: 'acc-work', state: 'ok', message: 'Mailbox checked – 1 new', timestamp: now / 1000 - 240 },
    { account_id: 'acc-proton', state: 'ok', message: 'Connected – watching INBOX', timestamp: now / 1000 - 3500 },
  ]
  const rule = (id, label, tags, r) => ({ id, label, tags, enabled: true, sender: '', subject: '', body: '', code_pattern: '[0-9]{4,8}', link_prefix: '', kind: 'code', ...r })
  let rules = [
    rule('github', 'GitHub (English)', [ALL], { sender: 'noreply@github.com', subject: '[GitHub] Please verify your device', body: 'Verification code: {code}', code_pattern: '[0-9]{6}', catalog: { app: 'github', lang: 'en' } }),
    rule('spotify', 'Spotify (English)', ['gmail'], { sender: '*@alerts.spotify.com', subject: '{code} * your Spotify login code', catalog: { app: 'spotify', lang: 'en' } }),
    rule('claude', 'Claude AI (English)', ['proton'], { kind: 'link', sender: '*mail.anthropic.com', subject: 'Secure link to log in to Claude.ai', link_prefix: 'https://claude.ai/magic-link', catalog: { app: 'claude-ai', lang: 'en' } }),
    rule('vpn', 'Company VPN', ['work'], { sender: 'security@example-corp.com', subject: 'Your VPN sign-in code is {code}', code_pattern: '[0-9]{6}' }),
  ]
  const ok = (stage, detail) => ({ stage, ok: true, detail })
  const fail = (stage, detail) => ({ stage, ok: false, detail })
  const mail = (seq, minutesAgo, account_id, from, subject, outcome, kind, value, traces) => {
    const acc = accounts.find(a => a.id === account_id)
    return { seq, timestamp: now - minutesAgo * 60000, account_id, account: acc.label, type: 'mail', uid: 1000 + seq,
             from, subject, outcome, kind, value, traces, timings: { headers: 84, body: kind === 'link' || outcome === 'partial' ? 212 : null, done: 230 } }
  }
  const log = [
    mail(1, 52, 'acc-gmail', 'Newsletter <news@shop.example>', 'Our autumn sale starts today', 'no_match', null, null,
      [{ rule: 'GitHub (English)', rule_id: 'github', stages: [fail('sender', 'does not match "noreply@github.com"')], value: null },
       { rule: 'Spotify (English)', rule_id: 'spotify', stages: [fail('sender', 'does not match "*@alerts.spotify.com"')], value: null }]),
    mail(2, 31, 'acc-proton', 'Anthropic <no-reply@mail.anthropic.com>', 'Secure link to log in to Claude.ai', 'found', 'link', 'https://claude.ai/magic-link',
      [{ rule: 'Claude AI (English)', rule_id: 'claude', stages: [ok('sender', 'matches "*mail.anthropic.com"'), ok('subject', 'matches "Secure link to log in to Claude.ai"'), ok('link', 'found link starting with "https://claude.ai/magic-link"')], value: 'https://claude.ai/magic-link' }]),
    mail(3, 12, 'acc-work', 'IT Security <security@example-corp.com>', 'Your VPN sign-in code is 731905', 'found', 'code', '731905',
      [{ rule: 'Company VPN', rule_id: 'vpn', stages: [ok('sender', 'matches "security@example-corp.com"'), ok('subject', 'matches "Your VPN sign-in code is {code}", code in subject')], value: '731905' }]),
    mail(4, 2, 'acc-gmail', 'GitHub <noreply@github.com>', '[GitHub] Please verify your device', 'found', 'code', '482913',
      [{ rule: 'GitHub (English)', rule_id: 'github', stages: [ok('sender', 'matches "noreply@github.com"'), ok('subject', 'matches "[GitHub] Please verify your device"'), ok('body', '"Verification code: {code}" found in the text')], value: '482913' }]),
  ]

  // Rough imitation of the backend rule test, good enough for the demo sample.
  function testRule({ rule: r, from, subject, body }) {
    const stages = [ok('sender', r.sender ? `matches "${r.sender}"` : 'any sender')]
    if (r.subject.includes('{code}')) {
      stages.push(ok('subject', `matches "${r.subject}", code in subject`))
      return { rule: r.label, rule_id: r.id, stages, value: (subject.match(/[0-9]{4,8}/) || [null])[0] }
    }
    stages.push(ok('subject', r.subject ? `matches "${r.subject}"` : 'any subject'))
    if (!body) return { rule: r.label, rule_id: r.id, stages, value: null }
    const value = (body.match(/[0-9]{6}/) || [null])[0]
    stages.push(r.kind === 'link' ? ok('link', `found link starting with "${r.link_prefix}"`) : ok('body', `"${r.body}" found in the text`))
    return { rule: r.label, rule_id: r.id, stages, value }
  }

  // The Maillet window (app.html): mails in the inbox, tasks, a fresh code.
  const sec = minutesAgo => Math.floor(now / 1000) - minutesAgo * 60
  const accName = id => accounts.find(a => a.id === id).label
  let mid = 0
  const m = (minutesAgo, account_id, from_name, from_addr, subject, preview, o = {}) => ({
    id: `${account_id}:1:${++mid}`, account_id, account: accName(account_id), uid: mid, from_name, from_addr, subject, preview,
    date: sec(minutesAgo), has_attachments: !!o.att, seen: !!o.seen, flagged: !!o.pinned, message_id: `<demo-${mid}@example>`,
    bulk: !!o.bulk, findings: o.code ? [{ feature: 'otp', kind: 'code', value: o.code }] : [] })
  const inbox = [
    m(14, 'acc-work', 'Mira Hansen', 'mira.hansen@example-corp.com', 'Offer for the spring campaign – your feedback?', 'Hi Alex, attached is the revised offer. Could you look at the two open points before Friday?', { att: true }),
    m(38, 'acc-proton', 'Jonas Weber', 'jonas.weber@example.org', 'Harz weekend: who brings the stove?', 'Train is booked for 8:32. Ada has the map, I can bring the gas – we still need someone for the stove.'),
    m(71, 'acc-work', 'Sam Okafor', 'sam.okafor@example-corp.com', 'Release notes 2.4 – ready for review', 'Draft is in the shared folder. Two items still need a screenshot, the rest is done.', { seen: true }),
    m(55, 'acc-gmail', 'Northwind Outdoor', 'news@northwind.example', 'Our autumn sale starts today', '', { bulk: true }),
    m(190, 'acc-work', 'Product Weekly', 'digest@productweekly.example', 'Five ideas for better onboarding', '', { bulk: true, seen: true }),
    m(260, 'acc-gmail', 'Parcel Service', 'noreply@parcel.example', 'Your parcel arrives tomorrow between 10 and 12', '', { bulk: true }),
  ]
  const pinned = [m(1500, 'acc-gmail', 'Lena Brandt', 'lena.brandt@example.org', 'Travel booking: Hamburg, 18–20 Nov', '', { pinned: true, seen: true })]
  const task = (status, minutesAgo, account_id, name, addr, subject, preview) => ({ status, mail: m(minutesAgo, account_id, name, addr, subject, preview, { seen: true }) })
  const tasks = [
    task('open', 300, 'acc-work', 'Mira Hansen', 'mira.hansen@example-corp.com', 'Contract draft: comments by Friday', 'Please check clauses 4 and 7, legal has added two notes.'),
    task('open', 1500, 'acc-work', 'Facility Team', 'facility@example-corp.com', 'New access card – pick up at reception', 'Your card is ready. Bring your ID.'),
    task('progress', 1600, 'acc-work', 'Sam Okafor', 'sam.okafor@example-corp.com', 'Budget sheet Q1 – numbers for marketing', 'I need the campaign figures to finish the sheet.'),
    task('progress', 4400, 'acc-gmail', 'Lena Brandt', 'lena.brandt@example.org', 'Workshop agenda, second draft', 'Changed the order of the sessions as discussed.'),
    task('done', 5800, 'acc-gmail', 'Jonas Weber', 'jonas.weber@example.org', 'Train tickets for the Harz weekend', ''),
  ]
  const unread = id => inbox.filter(x => x.account_id === id && !x.seen).length
  const mailData = {
    accounts: accounts.map(a => ({ account_id: a.id, account: a.label, unread_total: unread(a.id), inbox_total: inbox.filter(x => x.account_id === a.id).length })),
    mails: inbox, pinned, status: statuses.map(s => ({ account_id: s.account_id, state: s.state })),
  }
  const appStatus = {
    version: '0.7.1', accounts: accounts.map(a => ({ account_id: a.id, account: a.label, state: 'ok', unread: unread(a.id), timestamp: sec(1) })),
    api: { state: 'off', message: '' }, shortcut: { keys: 'Ctrl+Alt+M', error: null },
    actions: ['archive', 'task', 'file', 'pin', 'noise'], categories: [], window_form: 'popup', inbox_days: 7, mark_read_on_open: true,
  }

  const commands = {
    get_i18n: () => ({ strings: window.__DEMO_I18N__ || {}, language: 'en', languages: [{ code: 'en', name: 'English' }] }),
    list_mails: () => mailData,
    list_tasks: () => tasks,
    get_app_status: () => appStatus,
    get_accounts: () => accounts,
    get_rules: () => rules,
    get_status: () => statuses,
    get_log: () => [...log],
    get_catalog: () => ({ catalog: window.__DEMO_CATALOG__, offline_reason: null }),
    get_api_settings: () => ({ token: 'demo-token', config: { enabled: false, port: 6870, allowed_links: [] }, status: { state: 'off', message: '' } }),
    get_app_info: () => ({ name: 'Maillet', tagline: 'your one-time password helper', version: '0.4.0' }),
    test_rule: testRule,
    save_rule: ({ rule: r }) => rules,
    get_last_code: () => location.pathname.endsWith('app.html') && location.hash === '#home'
      ? JSON.stringify({ code: '482913', kind: 'code', account: 'Gmail', rule: 'GitHub', timestamp: now / 1000 - 20 }) : null,
    // ?theme=fjord&style=stage tries a colour theme and style; without, the windows follow light/dark of the system
    get_appearance: () => { const q = new URLSearchParams(location.search); return q.get('theme') ? { theme: q.get('theme'), style: q.get('style') || 'calm' } : null },
  }

  async function invoke(cmd, args = {}) {
    const fn = commands[cmd]
    return fn ? fn(args) : null
  }

  window.__TAURI__ = {
    core: { invoke },
    event: { listen: async (name, cb) => { if (name === 'show-code') window.__showCode = p => cb({ payload: p }) } },
  }

  const popup = (kind, code, account, rule, from, subject) =>
    ({ code, kind, account, account_id: 'x', uid: 1, rule, from, subject })
  const SCENARIOS = {
    accounts: () => go('accounts'),
    rules: () => go('rules'),
    'rule-editor': async () => {
      go('rules')
      await openRule('github', { sample: { from: 'noreply@github.com', subject: '[GitHub] Please verify your device',
        body: 'Hey alex-doe!\n\nVerification code: 482913\n\nIf you did not attempt to sign in, change your password.' } })
    },
    catalog: () => go('catalog'),
    home: () => {},
    tasks: () => document.getElementById('navTasks').click(),
    log: async () => {
      go('log')
      await new Promise(r => setTimeout(r, 200))
      logOpen = 4
      renderLog()
    },
    code: () => window.__showCode(popup('code', '482913', 'Gmail', 'GitHub (English)', 'GitHub <noreply@github.com>', '[GitHub] Please verify your device')),
    link: () => window.__showCode(popup('link', 'https://claude.ai/magic-link#token', 'Proton', 'Claude AI (English)', 'Anthropic <no-reply@mail.anthropic.com>', 'Secure link to log in to Claude.ai')),
  }

  window.addEventListener('load', () => setTimeout(async () => {
    const run = SCENARIOS[location.hash.slice(1)]
    if (run) await run()
    // The settings page starts with go('accounts'); the scenario may switch away.
  }, 100))
})()
