# `workflows/` — verification orchestration

Reusable Claude-Code workflows that orchestrate the project's verification discipline. Invoked via the
`Workflow` tool with `{scriptPath: "workflows/<name>.js", args: {...}}`.

## `vigilia.js` — the generic vigilia runner

Casts a **chosen set of datamancy wards** against a target home, then `circumspicere` last (optional).
This is *vigilia* (the grimoire meta-spell) operationalized as a fan-out: one sub-agent per ward, all in
parallel, then the surround. It **retires** the old bespoke `vigilatum-<home>-*` scripts — one generic
tool, parameterized per cast.

### Invoke

```
Workflow({
  scriptPath: "workflows/vigilia.js",
  args: {
    target: ["src/capability/mod.rs", "src/capability/registry.rs", "src/capability/policy.rs"],
    wards:  ["intueri", "solvere", "conformare", "purgare", "struere", "sequi",
             "temperare", "exigere", "excusare", "perspicere", "complectens", "vocare"],
    circumspicere: true
  }
})
```

Returns `{ perWard, unreached, L1, L2, L3, verdict }`. `verdict` is `CONVERGES — ready`
(L1+L2 == 0), `DIVERGES: n L1 + m L2`, or `INVALID` (a ward couldn't fetch its spell).

### How it works (design notes)

- **Live fetch, never embed.** Each worker fetches its spell live from the signed `datamancy` MCP
  (`https://datamancy.dev/<ward>/SKILL.md`), SHA-256-verified at the server per read. The spells are
  updated regularly — embedding a snapshot would cast a *stale* discipline; live-fetch always casts the
  current one. (This intentionally diverges from the grimoire's "embed, never fetch" rule, which assumes
  workers can't reach the MCP — here they can, and the MCP verification makes live-fetch both current
  *and* signed.)
- **The orchestrator picks the roster, per vigilia's selection rule:** universal-code wards always;
  conditional wards by trigger (e.g. `conformare` if the home has error types, `sequi` if it threads
  state, `excusare` if it carries `rune:` exemptions, `complectens`/`vocare` if it has tests, `secare`
  if parallel, `mora` if it waits); `cernere` only for wat-language code (skip for pure Rust homes).
- **`circumspicere` discovers its own surround** — it's the "look around" ward; the runner gives it only
  the target + the inward coverage and lets it find the complement (callers, trust boundaries,
  claims-vs-code), rather than spoon-feeding per-home hints.
- **`args` arrive JSON-stringified.** The harness may deliver `args` as a string rather than a parsed
  object; the script normalizes with a `typeof args === 'string' ? JSON.parse(args) : args` guard at the
  top. (This is what "args don't wire" actually was — they wire fine once parsed.)

### Severity + the stamp

`L1` = correctness lie · `L2` = structural mumble · `L3` = taste (noted, never gating). A home earns its
`//! vigilatum: <date> — vigilia N-spell …, L1+L2=0` stamp only over a **converged** cast (L1+L2 == 0),
weighed against the disk. Cast, don't narrate; ground every finding in a `file:line`.

## Future workflows

- a coverage-gate workflow (cargo-llvm-cov over warded homes)
- a grimoire `--check` (fail loud if a served spell is missing from the catalog)
