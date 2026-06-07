# Arc 252 — The coverage-rune + the warded-home coverage gate

**Status:** OPEN 2026-06-07. Builder-directed, **highest priority — rig it, prove it, ward it
before we ship, and before the migration resumes** (so every subsequent home wards WITH coverage
from the start; establishing the pattern now, retroactively on the existing homes, irons out every
edge case before the migration multiplies them).

**Name PROVISIONAL** (`252-coverage-rune`) — intueri-cast the real names (the rune category word,
the gate tool) at settle.

**Origin (builder, 2026-06-07):** "we should rig up cargo-llvm-cov?... we can rig up a rune
convention too, not tied to any given spell — the language-being-worked-on coverage tool (ruby's
simplecov, our pending wat-cov). This is our coverage exception, not a percentage target — we
strive for 100% per file and we rune off areas that are justifiably skippable; we argue every rune
on its own merit." Surfaced by circumspicere's value/-ward negative-space L3 (no co-located unit
tests; a gap with no number on it).

---

## The doctrine: 100%-minus-argued-runes (NOT a threshold)

A percentage threshold says "15% may rot silently." We strive for **100% coverage per warded file**;
every uncovered region is either **tested** or carries a **`rune:coverage`** that argues, on its own
merit, why it is justifiably skippable. The gate passes iff **(covered regions) ∪ (runed regions) =
all regions**. Uncovered-AND-not-runed is a finding (test it, or rune it with a reason). Same ethos
as the rest of the substrate: no silent gap; every exemption carries its reason.

## The triad — no new spell needed

The coverage-rune completes a triad the grimoire already runs:

```
clippy           raises  →  #[allow]          exempts  →  excusare weighs
coverage measurer raises →  rune:coverage     exempts  →  excusare weighs   ← this arc
```

excusare's discipline is already tool-agnostic ("any inline checker-suppression with a reason slot").
A `rune:coverage(...)` is exactly such a suppression. So the grimoire update is the **convention** +
**excusare explicitly recognizing it** — NOT a new castable spell. The measurer + gate are the raiser
(like clippy); the rune is the exemption (like `#[allow]`); excusare is the weigher.

## Three parts, cleanly separated

1. **Measurer** (per-language, commodity): `cargo-llvm-cov` (Rust src/ — INSTALLED 2026-06-07);
   a future wat-cov (wat corpus); simplecov (Ruby). Emits per-region coverage (JSON/lcov).
2. **The rune-aware gate** (universal — the real deliverable, the "wat-cov" layer): cargo-llvm-cov
   reports uncovered regions but **knows nothing of our runes**. The gate (a) runs the measurer
   leak-safe, (b) maps each uncovered region → source line, (c) checks that line carries a
   `rune:coverage` — **pass iff every uncovered region is runed.** Reports uncovered-not-runed as
   findings; lists the runed exemptions (for excusare to weigh).
3. **The rune convention** (grimoire-level, cross-language — *our* coverage exception, tied to no spell):
   `// rune:coverage(<category>) — <reason>`. Categories, each argued on its own merit:
   - `unreachable` — invariant/compiler-guaranteed-dead (e.g. value.rs's 13 `unreachable!` arms,
     `is_atomizable`-guaranteed; if ever hit, that IS the bug). The first exemplars.
   - `defensive` — an error path the production flow cannot trigger but must exist.
   - `platform` / `cfg` — env/cfg-gated paths not run in this measurement environment.
   - `proves-elsewhere` — covered by an integration test the per-file measurer can't attribute
     (cite the test).

## Leak-safe runner (load-bearing constraint)

cargo-llvm-cov WRAPS the test run → it must scope to the non-leaky tiers
(`cargo llvm-cov --lib -p wat` + named `--test`s + integration-run.sh), **NEVER** `--workspace`
(the proc-leak the recovery doc bans). The gate's runner honors this.

## The warded-home stamp gains a third axis

vigilatum currently asserts **L1+L2=0 + clippy-0**. This arc adds **coverage: 100%-or-runed**.
A warded home's stamp means audited AND clean AND **exercised**. Connects to arc 250
(vigilatum-integrity — self-enforcing stamps); the coverage gate is another self-enforcement axis.

## The plan (priority order — builder-set)

1. **Spellbook** (grimoire): the `rune:coverage` convention doc (datamancy.dev) + excusare/SKILL.md
   recognizing it. intueri-name the category word / any new grimoire entry. (Publishing to the live
   datamancy MCP is human-gated; write + commit the convention now, ship on approval.)
2. **cargo coverage**: verify cargo-llvm-cov + a leak-safe baseline across the 12 warded homes
   (function/check/types/collection/macros/scope/comms/remedy/argspec/rust_deps + value/) — real
   numbers before the gate. Build the rune-aware gate (the wat-cov layer; a script or xtask).
3. **100%-or-runed on ALL warded files**: drive each warded home to covered-or-runed; value/'s 13
   `unreachable!` arms get the first `rune:coverage(unreachable)` exemplars. Ward the gate tooling.
4. **Resume migration** — every subsequent home (scalar/, algebra/, eval/, …) wards WITH coverage
   from the start; the gate is part of the ward ritual.

## Why precedence now (builder)

"Making it precedence now irons out all problems we could find." Establishing the coverage discipline
on the 12 EXISTING homes surfaces every edge case (unreachable arms, defensive paths, integration-
attribution gaps, the leak-safe-runner wrinkles, the gate's rune-parsing) BEFORE the migration
multiplies them across a dozen more homes. Iron it on what we have; carry the proven pattern forward.

## Cross-references

- arc 250 (`docs/arc/2026/06/250-vigilatum-integrity/STUB.md`) — self-enforcing stamps; coverage is a sibling axis.
- arc 251 (`docs/arc/2026/06/251-types-as-forms/`) — the migration this gates going forward; SCORE-STONE-251.2-ward.md (circumspicere's negative-space finding that birthed this).
- `docs/VIGILATUM.md` — the stamp doctrine (gains the coverage axis).
- datamancy.dev grimoire — excusare (weighs the rune); the rune convention joins the rune family.
- Tasks: #190 (this), #188 (Duration u64), #189 (conformare out-of-home).
