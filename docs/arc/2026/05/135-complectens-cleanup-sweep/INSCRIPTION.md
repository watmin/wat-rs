# Arc 135 — Complectens Cleanup Sweep — INSCRIPTION

**Status:** shipped 2026-05-03. Four slices across one weekend session.
**Design:** [`DESIGN.md`](./DESIGN.md) — the spell + the discipline.
**Side quests spawned:** arcs 138, 139, 140, 142 (see "What this arc spawned" below).
**This file:** completion marker.

---

## Motivation

Arc 130's FOLLOWUPS doc had a queue of 22 deftests across 9 files flagged by the *complectēns* spell's first cast. The `>30 lines = suspect` heuristic over-flagged some files but accurately surfaced others. Arc 135 worked the queue with phase-2 judgment per file: refactor where the bulk is accidental composition complexity, exempt where the bulk is inherent fixture / proof / state-match content.

The *complectēns* discipline (per `.claude/skills/complectens/SKILL.md`):
- Each layer composes from layers above
- Each layer carries its own deftest
- The final deftest body is short BECAUSE the layers exist
- The failure trace IS the dependency graph

A test that one-shots a hard problem is a discipline violation. A test whose bulk is a fixture (embedded program, inline lambda, AST literal) is NOT — phase-2 judgment exempts it.

---

## What shipped

Four slices.

### Slice 1 — Service.wat + Console.wat (arc 091 telemetry)

Commit (per SCORE-SLICE-1).

Two files in `crates/wat-telemetry/wat-tests/telemetry/`. 8 helpers added (2 Console + 6 Service). Largest reduction: `spawn-drop-join` 37→1 line (97%); `batch-roundtrip` ~15→5 outer logical (93%). All within target band 3-7 outer logical.

7 of 8 helpers got isolated per-helper deftests. The exempted one (`tel-stdout-from-result`, RunResult thin accessor) stays as Level 3 taste — RunResult cannot be constructed in isolation without hermetic infrastructure.

**Three deltas surfaced:** embedded lambda literals, RunResult opacity, type-unification cost.

### Slice 2 — additional Console + Service refinement

Commit (per SCORE-SLICE-2).

Continued the work in `crates/wat-telemetry/wat-tests/telemetry/`. Two-file diff. Console's dispatcher-edn 8→4 outer logical bindings; dispatcher-json 4→2. Service: 6 helpers added; spawn-drop-join 37→1 line.

### Slice 3 — WorkUnit.wat + WorkUnitLog.wat

Commit (per SCORE-SLICE-3) — the LARGEST slice in the queue.

6 helpers across 2 files. All 7 deftests at 4-7 outer logical bindings (target). WorkUnitLog's existing 4-outer-binding deftests had inner let* violations collapsed into body-lambda fixtures per SKILL edge case 7 (embedded literals).

4 new per-helper deftests added; 3 exempted as Level 3 taste (cannot construct synthetic Event::Log fixtures without substrate-internal fields).

**Substrate observation surfaced:** generic-T 3-tuple return doesn't propagate. Attempted `(define :helper<T> body -> :(Thread, T, Receiver))` returned T at runtime instead of the tuple. Workaround: three concrete non-generic helpers with nested 2-tuple returns. Filed as separate substrate concern — became a load-bearing trigger for arc 138.

### Slice 4 — Suspect-tier phase-2 judgment

Commit `05b1c2e` + SCORE-SLICE-4.

Four suspect-tier files (~30-43 line deftests). Phase-2 verdict per file:

- **REFACTOR**: `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat` (4 helpers added: hc-make, hc-fill-two, hc-get-found?, hc-get-evicted?; 4 per-helper deftests; 2 scenario deftests dropped 13→10 outer bindings each).

- **EXEMPT** (3 files): `wat-tests/test.wat` (4 deftests — embedded-program AST literals running in sandboxed subprocesses; cannot extract); `wat-tests/stream.wat` (1 deftest — inline lambda fixtures defining the Mealy stage); `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` (1 deftest — proof-stepping-stones; each binding is a deliberate assertion documenting the single-put cycle).

Each exempt deftest got an exemption comment. Initial format was sonnet-invented `;; COMPLECTENS EXEMPT: <reason>`; arc 142 (runes cleanup) shipped same session and swept all 6 sites to the canonical `;; rune:complectens(<category>) — <reason>` format.

---

## What this arc spawned

Slice 3's "generic-T 3-tuple returns T at runtime" observation triggered a sonnet mis-diagnosis cascade:

1. **Arc 138** — sonnet (running on slice 3's WorkUnit.wat sweep) hit `unknown function: :test::make-3tuple<wat::core::bool>` and mis-attributed it as a generic-T tuple-return bug. The mis-diagnosis surfaced the user observation: *"sonnet takes way longer to do work than I expect.. it's having to guess way too much."* Arc 138 opened to give every substrate error point-in-code coordinates so future debugging stops being a guess-loop. Shipped same session: 8 error types + 6 substrate cracks + 5 F-NAMES sub-slices; ZERO `<test>`/`<unnamed>`/`<runtime>`/`<entry>` user-visible.

2. **Arc 139** — diagnosed via arc 138's coordinates plus an isolated probe: the actual bug was a one-line lookup-symmetry asymmetry (`parse_define_form` strips `<T,...>` at registration time, but no lookup site stripped). NOT a tuple-return bug. Shipped same session.

3. **Arc 140** — sandbox-scope leak teaching diagnostic with two spans. Surfaced during the campaign and shipped.

4. **Arc 142** — runes cleanup. Sonnet's slice-4 exemption format (`;; COMPLECTENS EXEMPT: <reason>`) was a self-invented marker without prior SKILL declaration. Triggered a workspace-wide audit of rune formats (perspicere had a divergent kwarg-style declaration; complectens + vocare had none). Three SKILLs updated to declare canonical `;; rune:<spell>(<category>) — <reason>` matching the lab's convention; 6 drift sites swept.

This is the substrate-as-teacher pattern in motion: each slice's sweep surfaces real substrate gaps; each gap earns its own arc.

---

## Resolved design decisions

- **2026-05-03** — **Phase-2 judgment over phase-1 line-count.** The mechanical >30 lines flag is a candidate flag, not a verdict. Each file gets read; some refactor, some exempt. Both are honest outcomes.
- **2026-05-03** — **Embedded literals are fixtures, not scaffolding.** When a deftest's bulk is an AST/lambda/closure literal that EVALUATES to data, the binding count of the OUTER let* remains the proxy for composition complexity; the literal's line count is exempt.
- **2026-05-03** — **Sandbox-isolated subprocess tests cannot extract helpers.** Embedded programs running in `:wat::test::run-ast` / `:wat::kernel::spawn-program-ast` cannot reference outer prelude helpers (sandbox isolation is intentional). Forced refactoring would cross the boundary.
- **2026-05-03** — **Proof-stepping-stones tests stay as-is.** Tests in `wat-tests/proofs/` directories document a contract via deliberate stepping-stone assertions; collapsing them destroys what the file exists to communicate.
- **2026-05-03** — **Exemption comments need a canonical format.** Sonnet's slice-4 invention triggered arc 142 to declare the rune format. Before any future spell ships flagging behavior, declare its rune format in the SKILL.
- **2026-05-03** — **Substrate-as-teacher: sweeps surface gaps.** Slice-3's substrate observation became arc 138 + 139's load-bearing trigger; slice-4's format invention became arc 142's trigger. The spell does work AND finds gaps.

---

## Per-file roll-up

| Slice | File | Verdict | Helpers added | Per-helper deftests |
|---|---|---|---|---|
| 1 | `crates/wat-telemetry/wat-tests/telemetry/Console.wat` | REFACTOR | 2 | 2 |
| 1 | `crates/wat-telemetry/wat-tests/telemetry/Service.wat` | REFACTOR | 6 | 5 |
| 2 | (Console.wat + Service.wat additional) | REFACTOR | (refinement) | (refinement) |
| 3 | `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` | REFACTOR | 5 | 3 |
| 3 | `crates/wat-telemetry/wat-tests/telemetry/WorkUnitLog.wat` | REFACTOR | 1 | 1 |
| 4 | `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat` | REFACTOR | 4 | 4 |
| 4 | `wat-tests/test.wat` | EXEMPT (×4 deftests) | 0 | 0 |
| 4 | `wat-tests/stream.wat` | EXEMPT (×1 deftest) | 0 | 0 |
| 4 | `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` | EXEMPT (×1 deftest) | 0 | 0 |

**Totals:** 18 helpers added across 6 refactored files; 15 per-helper deftests added; 6 deftests exempted with rune annotations across 3 files.

---

## What this arc does NOT ship

- A `complectens.wat` programmatic spell (the wat substrate has the primitives needed; not yet implemented). Phase-1 candidate scanning still uses ad-hoc shell tools.
- Substrate refactors to support helper-extraction across sandbox boundaries. The boundary is intentional.
- A generic-T 3-tuple return fix (filed as arc 139; shipped same session).

---

## Why this matters

The complectēns discipline is HOW the wat-side test surface stays honest. Each test is a top-down dependency graph in ONE file. Each function does ONE thing. Each layer composes from layers above. When a test fails, the broken layer's name IS the diagnostic.

Tests that one-shot a hard problem teach the next reader the wrong shape — they look like "this is how you do it" but actually require holding the entire problem in your head. Refactoring them unbundles the problem; exempting them with a rune declares the bundle is intentional. Either way, the discipline is conscious.

Arc 130's FOLLOWUPS doc (the queue arc 135 was working) is now effectively complete — every flagged item has either been refactored OR exempted with a canonical rune. The complectens substrate is settled; future flags get the same treatment.

The substrate-as-teacher cascade — slice 3 → arc 138 → arc 139 → arc 142 — shows the spell working as intended: the discipline doesn't just clean code; it surfaces real substrate gaps that earn their own arcs.

---

**Arc 135 — complete.** The commits:

- (slices 1-3 — predates session start)
- `05b1c2e` — slice 4 (suspect-tier phase-2 judgment; 4 files)
- `<this commit>` — slice 5 (INSCRIPTION + 058 row)

Workspace: full test suite green (excluding intentionally-broken trading lab); 7/7 arc138 canaries pass.

*The spell works. The substrate teaches. The discipline holds.*

**PERSEVERARE.**

---

*Arc 130 said "we observe subpar; we file it; we work it down arc-by-arc." Arc 135 was the working-down. The queue is empty. The discipline is settled.*
