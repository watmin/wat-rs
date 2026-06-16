export const meta = {
  name: 'vigilia',
  description: 'Generic vigilia — cast a CHOSEN set of datamancy wards against a target home. Each worker FETCHES its spell LIVE from the signed datamancy MCP (current discipline, SHA-256-verified per read — never embedded, so spell updates are always picked up). circumspicere cast LAST if requested, discovering its own surround. Reusable across any home: args = { target: [files], wards: [names], circumspicere?: bool }.',
  phases: [
    { title: 'Guard', detail: 'the chosen inward wards in parallel — each fetches its spell live' },
    { title: 'Perimeter', detail: 'circumspicere last (if requested) — discovers its own surround' },
  ],
}

const FINDINGS_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    ward: { type: 'string' },
    spellFetched: { type: 'boolean' },
    convergence: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        properties: {
          severity: { type: 'string', enum: ['L1', 'L2', 'L3'] },
          file: { type: 'string' },
          line: { type: 'string' },
          finding: { type: 'string' },
          quote: { type: 'string' },
          suggestion: { type: 'string' },
        },
        required: ['severity', 'file', 'line', 'finding', 'quote', 'suggestion'],
      },
    },
  },
  required: ['ward', 'spellFetched', 'convergence', 'findings'],
}

// ── args contract ───────────────────────────────────────────────────────────
// { target: ['src/foo/mod.rs', ...], wards: ['intueri','solvere',...], circumspicere?: true }
// The orchestrator picks the roster per vigilia's selection rule (universal-always +
// conditional-by-trigger + kind-scoped) and passes the NAMES; workers fetch the live spell.
// The harness may deliver `args` as a JSON STRING rather than a parsed object — normalize
// defensively. THIS is what "args don't wire" actually was: the object arrived stringified.
// Parse-if-string and they wire fine; this makes the tool robust to either delivery shape.
const V = (typeof args === 'string') ? JSON.parse(args) : (args || {})
if (!Array.isArray(V.target) || V.target.length === 0 || !Array.isArray(V.wards) || V.wards.length === 0) {
  throw new Error('vigilia: args must be { target: [files], wards: [names], circumspicere?: bool } — got: ' + JSON.stringify(args))
}
const filesList = V.target.map((f) => '- ' + f).join('\n')

// Live fetch — never embed. The MCP SHA-256-verifies every read, so a worker reading the
// signed channel directly gets the CURRENT, verified discipline (spell updates are picked up).
function fetchClause(ward) {
  return [
    'FETCH YOUR SPELL FIRST — LIVE from the signed datamancy MCP; do NOT recite it from memory (it may have changed):',
    '1. If the MCP resource tool is not loaded: call ToolSearch with query "select:ReadMcpResourceTool".',
    '2. Call ReadMcpResourceTool with server: "datamancy", uri: "https://datamancy.dev/' + ward + '/SKILL.md".',
    '3. Read the returned text IN FULL — that IS your ward (current + SHA-256-verified at the server).',
    'If after trying you genuinely CANNOT reach the datamancy MCP, set spellFetched=false, convergence="SPELL UNREACHABLE", findings=[] and STOP — an unread spell is an invalid cast, not a finding. Otherwise set spellFetched=true.',
  ].join('\n')
}

function inwardPrompt(ward) {
  return [
    'You are casting the datamancy ward **' + ward + '** — ONE defect class — as part of a vigilia (the full guard standing). Grimoire ethos: ground every claim against the disk; cast, do not narrate; failure is data.',
    '',
    fetchClause(ward),
    '',
    'TARGET — read each file IN FULL with the Read tool, on the current git branch (cwd /home/watmin/work/holon/wat-rs):',
    filesList,
    '',
    'Cast ' + ward + ' on these files. Apply ONLY the concern this ward owns — sibling concerns belong to other wards in this vigilia; do not poach. For every finding:',
    '- ground it in an actual file:line you READ this run; put the offending text in "quote"; no speculation.',
    '- "severity": L1 = correctness lie, L2 = structural mumble, L3 = taste (noted, never gating).',
    '- respect any rune:' + ward + '(...) exemption in the code (skip it).',
    'If the concern does not apply or the code is clean, return convergence "CONVERGED" with empty findings. Do NOT invent findings to look thorough.',
    '',
    'Return structured output: ward, spellFetched, convergence, findings[].',
  ].join('\n')
}

phase('Guard')
const inward = (await parallel(
  V.wards.map((w) => () => agent(inwardPrompt(w), { label: 'ward:' + w, phase: 'Guard', schema: FINDINGS_SCHEMA }))
)).filter(Boolean)

let perim = null
if (V.circumspicere) {
  phase('Perimeter')
  const coverage = inward.map((r) => '- ' + r.ward + ': ' + (r.spellFetched ? r.convergence : 'SPELL UNREACHABLE')).join('\n')
  perim = await agent(
    [
      'You are casting **circumspicere** LAST in a vigilia — after the inward guard reported. Your quarry is the SURROUND the inward lenses turned their backs on: default-behaviour egress, claims-vs-code, unenforced load-bearing invariants, and negative space (a surface/failure-class NO inward ward examined). DISCOVER the surround yourself — read whatever surrounding sites (callers, gates, docs) you need; do not wait to be told where to look.',
      '',
      fetchClause('circumspicere'),
      '',
      'TARGET — read each file IN FULL on the current git branch (cwd /home/watmin/work/holon/wat-rs):',
      filesList,
      '',
      'WHAT THE INWARD GUARD COVERED (find the COMPLEMENT — do not re-walk):',
      coverage,
      '',
      'Ground every finding in a file:line (in "quote"). A finding that contradicts a shipped CLAIM (module doc vs what the code enforces) ranks highest (L1).',
      '',
      'Return structured output: ward "circumspicere", spellFetched, convergence, findings[].',
    ].join('\n'),
    { label: 'ward:circumspicere', phase: 'Perimeter', schema: FINDINGS_SCHEMA }
  )
}

const all = [...inward, ...(perim ? [perim] : [])].filter(Boolean)
const flat = all.flatMap((r) => (r.findings || []).map((f) => ({ ...f, ward: r.ward })))
const l1 = flat.filter((f) => f.severity === 'L1')
const l2 = flat.filter((f) => f.severity === 'L2')
const l3 = flat.filter((f) => f.severity === 'L3')
const unreached = all.filter((r) => !r.spellFetched).map((r) => r.ward)

return {
  perWard: all.map((r) => ({ ward: r.ward, fetched: r.spellFetched, convergence: r.convergence, n: (r.findings || []).length })),
  unreached,
  L1: l1,
  L2: l2,
  L3: l3,
  verdict: unreached.length
    ? 'INVALID — ' + unreached.length + ' ward(s) could not fetch their spell: ' + unreached.join(', ')
    : (l1.length + l2.length === 0 ? 'CONVERGES — ready' : 'DIVERGES: ' + l1.length + ' L1 + ' + l2.length + ' L2'),
}
