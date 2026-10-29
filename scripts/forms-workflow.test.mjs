// Checks for the n8n forms receiver (deploy/n8n/forms-ingest.json).
// The export is the artefact operators import, so its wiring and its two Code
// nodes are asserted here instead of only in a live instance.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

const workflow = JSON.parse(
  readFileSync(new URL('../deploy/n8n/forms-ingest.json', import.meta.url), 'utf8'),
);

const byName = (name) => {
  const found = workflow.nodes.find((entry) => entry.name === name);
  assert.ok(found, `workflow has no node named ${name}`);
  return found;
};

function runCode(name, { input, items = [], nodes = {}, staticData = {}, env = {} }) {
  const body = byName(name).parameters.jsCode;
  const fn = new Function('$', '$input', '$getWorkflowStaticData', '$env', body);
  const $ = (nodeName) => ({
    first: () => {
      assert.ok(nodeName in nodes, `no stubbed output for ${nodeName}`);
      return { json: nodes[nodeName] };
    },
  });
  const $input = {
    first: () => ({ json: input }),
    all: () => items.map((json) => ({ json })),
  };
  return fn($, $input, () => staticData, env);
}

const request = (overrides = {}) => ({
  body: {
    app: 'landing',
    form: 'interest',
    fields: { _hp: '', email: 'someone@example.com' },
    meta: { sentAt: '2026-10-28T15:20:00.000Z' },
    ...overrides,
  },
  headers: { 'x-forwarded-for': '203.0.113.9', 'content-type': 'application/json' },
});

const registryRow = (overrides = {}) => ({
  app: 'landing',
  form: 'interest',
  databaseId: 'db-123',
  titleProperty: 'Name',
  propertyMap: '',
  token: '',
  notifyUrl: '',
  ...overrides,
});

const authorize = (req, rows, env = {}) => {
  const guard = runCode('Guard', { input: req, staticData: {} })[0].json;
  return runCode('Authorize', { items: rows, nodes: { Guard: guard }, env })[0].json;
};

test('wiring only references nodes that exist and every node is reachable', () => {
  const names = new Set(workflow.nodes.map((entry) => entry.name));
  assert.equal(names.size, workflow.nodes.length, 'node names must be unique');

  const ids = new Set(workflow.nodes.map((entry) => entry.id));
  assert.equal(ids.size, workflow.nodes.length, 'node ids must be unique');

  const targets = new Set();
  for (const [source, outputs] of Object.entries(workflow.connections)) {
    assert.ok(names.has(source), `connection from unknown node ${source}`);
    for (const branch of outputs.main ?? []) {
      for (const link of branch) {
        assert.ok(names.has(link.node), `connection to unknown node ${link.node}`);
        targets.add(link.node);
      }
    }
  }

  for (const entry of workflow.nodes) {
    const isTrigger = entry.type.endsWith('.webhook');
    assert.ok(isTrigger || targets.has(entry.name), `node ${entry.name} is unreachable`);
  }
});

test('the webhook only accepts POST and answers through Respond nodes', () => {
  const webhook = byName('Webhook').parameters;
  assert.equal(webhook.httpMethod, 'POST');
  assert.equal(webhook.path, 'forms-ingest');
  assert.equal(webhook.responseMode, 'responseNode');
});

test('guard rejects unknown apps and malformed payloads', () => {
  const [noFields] = runCode('Guard', { input: { body: { app: 'landing', form: 'x' }, headers: {} } });
  assert.deepEqual({ status: noFields.json.status, ok: noFields.json.ok }, { status: 400, ok: false });

  const [unknown] = runCode('Guard', { input: request({ app: 'nope' }) });
  assert.equal(unknown.json.status, 400);

  const [oversize] = runCode('Guard', {
    input: request({ fields: { _hp: '', email: 'a@b.c', message: 'x'.repeat(40_000) } }),
  });
  assert.equal(oversize.json.status, 413);
});

test('guard answers 200 without work for a filled honeypot', () => {
  const [result] = runCode('Guard', {
    input: request({ fields: { _hp: 'bot', email: 'bot@example.com' } }),
  });
  assert.equal(result.json.ok, false);
  assert.equal(result.json.status, 200);
  assert.equal(result.json.error, '');
});

test('guard limits each ip and form to five submissions a minute', () => {
  const staticData = {};
  const send = () => runCode('Guard', { input: request(), staticData })[0].json;

  for (let attempt = 0; attempt < 5; attempt += 1) {
    assert.equal(send().ok, true, `attempt ${attempt + 1} should pass`);
  }
  assert.equal(send().status, 429);

  // A different form has its own budget.
  const [other] = runCode('Guard', {
    input: request({ form: 'beta' }),
    staticData,
  });
  assert.equal(other.json.ok, true);
});

test('guard reads the token from either accepted header', () => {
  const [bearer] = runCode('Guard', {
    input: { ...request(), headers: { authorization: 'Bearer secret-1', 'x-forwarded-for': '198.51.100.1' } },
    staticData: {},
  });
  assert.equal(bearer.json.token, 'secret-1');

  const [xToken] = runCode('Guard', {
    input: { ...request(), headers: { 'x-token': 'secret-2', 'x-forwarded-for': '198.51.100.2' } },
    staticData: {},
  });
  assert.equal(xToken.json.token, 'secret-2');
});

test('authorize rejects unknown forms, wrong tokens and attachments', () => {
  assert.equal(authorize(request(), []).status, 400);

  const wrongToken = authorize(request(), [registryRow({ token: 'expected' })]);
  assert.deepEqual({ status: wrongToken.status, error: wrongToken.error }, {
    status: 401,
    error: 'unauthorized',
  });

  const attachment = authorize(
    request({ fields: { _hp: '', email: 'a@b.c', attachment: 'data:application/pdf;base64,AA==' } }),
    [registryRow()],
  );
  assert.equal(attachment.status, 415);

  const noDatabase = authorize(request(), [registryRow({ databaseId: '' })]);
  assert.equal(noDatabase.status, 500);

  const badMap = authorize(request(), [registryRow({ propertyMap: 'not json' })]);
  assert.equal(badMap.status, 500);
});

test('authorize accepts a blank registry token as a public form', () => {
  const result = authorize(request(), [registryRow({ token: '' })]);
  assert.equal(result.authorized, true);
  assert.equal(result.databaseId, 'db-123');
});

test('authorize requires a verification token once a Turnstile secret is set', () => {
  const missing = authorize(request(), [registryRow()], { TURNSTILE_SECRET: 'sec' });
  assert.equal(missing.status, 400);

  const present = authorize(
    request({ meta: { turnstile: 'token-1' } }),
    [registryRow()],
    { TURNSTILE_SECRET: 'sec' },
  );
  assert.equal(present.authorized, true);
  assert.equal(present.needsTurnstile, true);
  assert.equal(present.turnstileToken, 'token-1');
});

test('authorize builds a Notion page with mapped property types and a title fallback', () => {
  const result = authorize(
    request({
      fields: { _hp: '', email: 'someone@example.com', name: 'Sam', seats: '4', plan: 'pro' },
    }),
    [
      registryRow({
        propertyMap: JSON.stringify({
          name: { name: 'Name', type: 'title' },
          seats: { name: 'Seats', type: 'number' },
          plan: { name: 'Plan', type: 'select' },
          email: { name: 'Email', type: 'email' },
        }),
      }),
    ],
  );

  assert.equal(result.authorized, true);
  assert.deepEqual(result.page.parent, { database_id: 'db-123' });
  assert.deepEqual(result.page.properties, {
    Seats: { number: 4 },
    Plan: { select: { name: 'pro' } },
    Email: { email: 'someone@example.com' },
    Name: { title: [{ text: { content: 'Sam' } }] },
  });
  assert.equal(result.page.properties._hp, undefined);
});

test('authorize falls back to the email as the title when nothing is mapped', () => {
  const result = authorize(request(), [registryRow({ titleProperty: '' })]);
  assert.deepEqual(result.page.properties.Name, {
    title: [{ text: { content: 'someone@example.com' } }],
  });
});
