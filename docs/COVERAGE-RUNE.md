# The coverage-rune convention

**Status:** canonical spec (arc 252). Proven in wat-rs first; **ships to the datamancy grimoire +
excusare's recognized-override list on approval** (the gated publish — "rig/prove/ward before we
ship"). Names provisional pending intueri at the ward.

> Our coverage exception. Not a threshold — **100% per file, minus regions runed (and argued) as
> justifiably skippable.** Every uncovered line is either tested or a named, weighed decision.

## The doctrine: 100%-minus-argued-runes

A percentage threshold ("≥85%") licenses silent rot in the uncovered remainder. We strive for **100%
coverage per warded file**; every region the measurer reports uncovered must be EITHER exercised by a
test OR carry a **`rune:coverage`** that argues — on its own merit — why it is justifiably skippable.
The gate passes a file iff **(covered regions) ∪ (runed regions) = all regions**. Uncovered-AND-not-
runed is a finding: test it, or rune it with a reason. Same ethos as the rest of the substrate —
no silent gap; every exemption carries its reason.

## The rune

```
// rune:coverage(<category>) — <reason>
```

Tool-agnostic and **tied to no spell** — it is the coverage exception of the language being worked
on, whatever measures it (cargo-llvm-cov for Rust, a future wat-cov for the wat corpus, simplecov
for Ruby). Placement: on (or immediately above) the uncovered region it exempts — a fixed-content
match the gate keys on. The reason is required; a bare or vague reason fails.

**Categories (each argued individually — no blanket grants):**

- `unreachable` — invariant- or compiler-guaranteed-dead. The region cannot execute by a structural
  guarantee; if it ever did, THAT is the bug (the panic IS the proof). *Exemplar: value.rs's 13
  `unreachable!` arms, `is_atomizable`-guaranteed.* The reason must name the guaranteeing invariant.
- `defensive` — an error/edge path the production flow cannot trigger but which must exist for
  safety. The reason must name why production can't reach it.
- `platform` / `cfg` — a path gated to a platform/cfg not exercised in this measurement environment.
  The reason must name the gate.
- `proves-elsewhere` — the region IS exercised, by a test the per-file measurer can't attribute
  (e.g. an integration test that drives it through a process boundary). The reason MUST cite the test.

## The triad — no new spell

The coverage-rune completes a pattern the grimoire already runs:

```
clippy            raises  →  #[allow]          exempts  →  excusare weighs
coverage measurer raises  →  rune:coverage     exempts  →  excusare weighs
```

- **Raiser:** the measurer + the rune-aware gate — they report uncovered-not-runed regions (the finding).
- **Exemption:** `rune:coverage(...)` — like `#[allow]`, it silences the gate for a region, with a reason.
- **Weigher:** **excusare** — its discipline is already tool-agnostic ("any inline checker-suppression
  with a reason slot"). It audits each `rune:coverage` at birth (is the skip justified?) and over time
  (still?). A `coverage(unreachable)` whose guaranteeing invariant was removed becomes a STALE-GUARD;
  a `proves-elsewhere` whose cited test was deleted becomes ORPHANED. No new spell is needed —
  excusare's recognized-override list simply names `rune:coverage` explicitly.

## The gate (the "wat-cov" layer — the real deliverable)

The measurer (cargo-llvm-cov) reports uncovered regions but **knows nothing of our runes.** The gate:
1. Runs the measurer **leak-safe** — scoped to the non-leaky tiers (`cargo llvm-cov --lib -p wat`
   + named `--test`s + integration-run.sh); **never** `--workspace` (proc-leak).
2. Maps each uncovered region → source line.
3. Checks the line carries a `rune:coverage` (or is otherwise covered). **Pass iff every uncovered
   region is runed.** Reports uncovered-not-runed as findings; lists the runed exemptions (excusare's input).

## The warded-home stamp gains a third axis

`vigilatum` asserts **L1+L2=0 + clippy-0**. With this convention it also asserts **coverage:
100%-or-runed**. A stamp means audited AND clean AND **exercised**. (Connects arc 250 vigilatum-
integrity — the coverage gate is another self-enforcement axis; a stamp that can silently lose
coverage is a stamp that lies.)

## Ship checklist (the gated publish)

- [ ] convention proven in wat-rs (gate built; applied 100%-or-runed across the 12 warded homes)
- [ ] gate tooling warded (its own vigilatum)
- [ ] datamancy: excusare/SKILL.md adds `rune:coverage` to the recognized-override surface + verdict examples
- [ ] datamancy: a grimoire convention doc (this file's content) + manifest regen + sign + publish (human-gated)
- [ ] intueri-cast the category words + the gate-tool name before the grimoire ship

See `docs/arc/2026/06/252-coverage-rune/DESIGN.md` for the full arc plan.
