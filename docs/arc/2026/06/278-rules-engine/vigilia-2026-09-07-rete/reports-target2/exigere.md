## EXIGERE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

**Population re-derivation** (25 files, 23,886 lines — confirmed via `wc -l`):

```
grep -nE  '\b(TODO|FIXME|XXX|HACK)\b' <25 files>   → 0
grep -niE '\b(TODO|FIXME|XXX|HACK)\b' <25 files>   → 0 (case-insensitive re-check)
```

Zero TODO-family in target 2, independently re-derived twice — target 1's zero did not carry over; this target's zero was earned separately.

Deferral-phrase sweeps run (all 25 files):

```
grep -niE 'deferred to (a )?future arc|future arc when|future cleanup|future polish|future-self|
  future caller|will be (added|implemented|extracted|supported)|will land|can land later|
  left for (future|later|a )|to be added|not yet implemented|not yet supported|scratch arc|
  punted|pending arc|next arc|small follow-up'                          → 3 hits (all false positives, verified by reading)

grep -niE "outside scope|outside .{0,30}'?s scope|would require .* refactor|
  would be a substrate-level refactor|could RAII-ify|could extract|could refactor"  → 0

grep -niE "intentionally discarded|should be (added|extended|extracted|fixed|refactored|
  supported|handled)|should land|should consider|if pressure|if demand|when needed|
  when .* surfaces"                                                     → 0

grep -niE "for now|eventually|someday|\bstub\b|unimplemented|placeholder|
  \bdefer(red|ral)?\b|\bpunt\b"                                         → 32 hits (all read; all present-tense/historical/domain-terminology)

grep -niE "\btemporary\b|\bworkaround\b|\bTBD\b|quick fix|band-?aid|for the moment"  → 1 (false positive: "a temporary" = a local variable)

grep -niE "\boutside\b"    → 18 hits, none scope-defense framing
grep -niE "\bshould\b"     → 20 hits, all invariant-assertions ("should be unreachable")
grep -niE "left for"       → 1 hit (a negation: "no value left for a future hand to forget")

grep -rn "rune:exigere" <25 files>   → 2 hits (matches the prior-ward enumeration exactly)
```

## Findings read and dismissed (verified false positives)

- `compiled_cond.rs:88` — *"two places a new comparison had **to be added**"* — past-tense history of a duplication that was removed.
- `expr_ir/eval.rs:184` — *"will land on the form, not the sub-expression"* — present-tense description of where an error surfaces.
- `purity.rs:1758` — *"no unordered value **left for** a future hand to forget to sort"* — a negation; states nothing is left to defer.
- `compiled_cond.rs:1378` — *"AFFIRMATIVELY CUT, not deferred (T7's close, 2026-08-25)"* — explicit renunciation of deferral with inline architectural reasoning; accepted affirmative scope-bounding, makes no tracked-elsewhere claim, so no arc-verification is owed.
- `compiled_rhs.rs:407` — *"would make every case here **defer** to the interpreter"* — hypothetical about a bad test setup.
- `matcher.rs:230-237, 497` — `cond-has-deferred-constraint?` / *"a **deferred** join constraint would be lost"* — domain terminology naming an algorithmic concept, present-tense.
- `expr_ir/eval.rs:1370-1390` — *"THIS DOC USED TO SAY… AND THAT WAS THE WHOLE DEFECT"* — the falsified-belief-recorded-in-place pattern the cast exempts.
- `wat/rete/compile.wat:598-599` — *"that was a **deferral** written AGAINST the stone… **This IS that strike**"* — past-tense correction, present-tense fix delivered.
- `vocabulary.rs:1707` — *"nothing stops a **future** hand from writing `Ret::NoScheme`…"* — a BEWARE-style risk warning, not a promise of work.
- `vocabulary.rs`/`purity.rs` PLACEHOLDER / BRIEF-*.md citations — present-tense type facts or dated historical audit notes.
- `alpha_tree.rs:35` — *"outside this repo and no gate here could ever check it"* — present-tense architectural boundary.
- `wat/rete/syntax.wat:355` — *"Any future sweep over this file must exclude it, or expect to undo this one line again."* — a maintenance-hazard warning (BEWARE-shape) about a tool's future re-run.
- `wat/rete/compile.wat:998` — *"Clara **defers** accumulators…"* — describes a reference system, not this codebase.

## FINDING (1)

**`wat/rete/factbag.wat:7`**
> `;; Doors, all under `:wat::rete::factbag::` — the whitelist a future rung-3 seal will name:`

- **Level:** L1 (hard-cut default — no named arc reference in the comment itself)
- **Category:** future-arc-deferral phrase family (*"will name"* ≈ *"will be"* / *"will land"*)
- **Direction:** either fix now (formalise the whitelist in the type system today) or bound it — cite the tracking arc by number inline with a `rune:exigere(attested-arc)`. As written it names no tracker; *"rung-3"* is a repo-wide phase-label, not an arc number, and grepping the wider tree shows a companion doc stating a sibling "rung 3" item is *"neither scoped nor scheduled"* (`docs/arc/2026/04/109-kill-std/NOTE-kwargs-or-positional-is-decided-in-five-places.md:86`) — i.e. a real, currently-untracked deferral, not an idle phrase.

**Considered and NOT flagged, same file, adjacent line** — `wat/rete/factbag.wat:19`: *"`tests/lint/no_raw_factbag_access.rs` is the seal today. When rung 3 arrives, that gate is deleted — the deletion is the proof."* This is a self-obsoleting design contract stated in present tense (this file substitutes for type-level enforcement that doesn't exist; its own future deletion is the objective, self-verifying trigger) — structurally close to the ward's exempted *"Arc N intentionally does NOT cover Y… if/when X, a NEW ARC opens"* pattern. **I read it as exempt, but flag the judgment call explicitly** since it sits one line from a genuine finding on the same "rung-3" concept.

## Runes verified (2, matches re-derived count exactly)

- **`purity.rs:22`** — `rune:exigere(attested-arc) — registry is arc 255.` Full context (`:15-22`) names the DESIGN artifact inline. **Verdict: clear** — `docs/arc/2026/06/255-builtin-registry/` exists on disk, and the specific `NOTE-purity-is-definition-time-queryable-metadata.md` exists inside it (confirmed by `ls`).
- **`wat/rete.wat:524`** — `rune:exigere(scope-affirmative) — arc 278 proof-by-diff fixture: nested string::concat is left intentionally. The arc-277 auto-fix is bare-symbol-only and cannot reach this compound case. Do NOT hand-fix.` **Verdict: clear** — `docs/arc/2026/06/278-rules-engine/` exists; the reason gives a concrete, checkable substrate-architectural cause (the auto-fix tool's own scope limit).

## FINDINGS

One item to report: `wat/rete/factbag.wat:7` (L1, bare future-deferral phrase, no tracker).
