# Realizations — slice 1 review (2026-05-09)

## What surfaced

Slice 1 of arc 170 (commit `787c977` + SCORE `bb155ed`) shipped a
working closure-extraction algorithm — 14/14 scorecard rows pass,
Mode A clean, 2108/0 verified locally. The substrate primitive is
sound: free-symbol walker, dep-closure builder, portability
check, topological sort all correct.

But the **public shape of `ClosurePackage` carries the
entry-keyword ceremony DESIGN explicitly killed.**

```rust
// Slice 1 shipped:
pub struct ClosurePackage {
    pub forms: Vec<WatAST>,
    pub entry: String,  // ← the ceremony DESIGN settled to retire
}
```

For inline-lambda input, slice 1 synthesizes
`:__closure::__pkg_<n>`, wraps the fn AST in
`(:wat::core::define :__closure::__pkg_<n> (fn ...))`, appends to
`forms`, exposes the synthetic name as `entry: String`. The
consumer (spawn-process Rust) then looks up that synthetic name
in the frozen world and applies.

**This contradicts the DESIGN-conversation settlement** (DESIGN.md
lines 102-108, 484-509):

> 5. The "name discovery" path (substrate looks up a canonical
>    entry symbol) creates ceremony. The user's preference: **the
>    fn IS the program**; pass it directly; substrate handles
>    closure extraction internally.

> 16. User questioned why entry-keyword is needed: *"why do we
>     even need a name if the forms /are/ the thing that
>     matters?"*

> 17. User refined further: the fn IS the program; spawn-process
>     takes fn directly; no Program wrapper type; closure
>     extraction is internal

The DESIGN killed the entry-keyword at the wat surface. Slice 1
re-introduced it one layer down at the Rust public-API surface.
Same ceremony, different layer.

## Why the deficiency wasn't caught in scoring

The scorecard rows in EXPECTATIONS-SLICE-1.md verified:
- Module + types minted (A)
- Subsystems implemented correctly (B-F)
- Tests pass (G)
- Workspace clean (H)
- No surface added at wat level (I)
- Branch isolation (L)
- Zero Mutex (M)
- Diagnostic UX (N)

What was MISSING from the scorecard: a **DESIGN-intent alignment
row.** A row that asks: *"Does the public shape of this
substrate primitive honor the DESIGN's settled architectural
intent?"* — not just the BRIEF's spec, but the DESIGN's spirit.

The agent shipped exactly what the BRIEF specified (synthetic
name + entry field, per CLOSURE-EXTRACTION.md). The BRIEF was
correct relative to its own spec. **The spec itself was wrong
relative to DESIGN.** The orchestrator (me) drafted the BRIEF
without recognizing that the synthetic-name approach
contradicted the conversation log captured in DESIGN.md lines
484-509.

The agent did its job. The orchestrator's BRIEF was the upstream
defect.

## The honest shape

The fn-form `(fn [stdin :IOReader stdout :IOWriter stderr :IOWriter] :nil ...)`
already evaluates to a fn Value. The substrate's evaluator turns
fn-forms into fn Values directly. We don't need to wrap in a
define + look up by name; we can keep the entry as a fn-form
expression.

```rust
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,  // type defs + dep defs (the captured environment)
    pub entry_form: WatAST,     // an expression evaluating to a fn Value:
                                //   - inline-lambda input: the fn-form AST itself
                                //   - keyword-path input:  a Symbol AST that
                                //     resolves into prologue's defines
}
```

Consumer (spawn-process Rust):

```rust
let pkg = extract_closure(&fn_value, sym, &types)?;
let frozen = startup_from_forms(pkg.prologue, ...)?;
let fn_value = eval(&pkg.entry_form, env, frozen.symbols())?;
let result = apply_function(fn_value, args, frozen.symbols(), span)?;
```

No synthetic name. No `entry: String`. The fn IS the program at
the structural level too.

## What needs to ship

Slice 1b — structural reshape:

1. **`closure_extract.rs`**:
   - `ClosurePackage` becomes `{ prologue, entry_form }`
   - Synthetic-name machinery (`__closure::__pkg_<n>` counter,
     wrap-in-define logic) removed
   - For inline-lambda input: emit the fn-form AST as
     `entry_form`; do not wrap or name it
   - For keyword-path input: emit the symbol AST as `entry_form`;
     prologue includes the user's existing define for that symbol
   - Topological sort: types → captures → user deps (NO
     trailing entry define — the entry is `entry_form`, not in
     `prologue`)

2. **Tests `tests/wat_arc170_closure_extraction.rs`**:
   - T1-T15 assertions update to the new shape
   - Regression: re-freezing prologue + evaluating entry_form
     produces a fn Value with behavior equivalent to the
     original input fn

3. **CLOSURE-EXTRACTION.md** amendment:
   - Steps 1, 6 reshape (entry resolution + assembly)
   - Synthetic-name section retired
   - Invariants update (I2 retires; new invariant for
     entry_form evaluating to a fn Value)
   - Test cases update to assert prologue + entry_form roundtrip

4. **DESIGN.md slice plan** amendment:
   - Insert slice 1b between slices 1 and 2
   - Slice 2 explicitly depends on slice 1b's reshape

## What does NOT change

- The closure-extraction algorithm (free-symbol walker, dep
  closure, portability check, Value→AST encoder for captures)
- Honest deltas A through F from SCORE-SLICE-1 still apply:
  - Q-impl-2 captured-fn-value gap (closures-of-closures)
  - Value-kind encoding gaps
  - Diagnostic type-name spelling
  - Topological sort edge tracking
  - Auto-accessor short-circuit
- SCORE-SLICE-1.md (immutable historical record per
  `feedback_inscription_immutable.md`)
- The arc 170 client/server framing
- The spawn primitive surface (`(:wat::kernel::spawn-process fn)`)

## Discipline lesson — for future BRIEFs

Add to EXPECTATIONS scorecards a row of the form:

> **DESIGN-intent alignment** — does the shipped public shape
> honor the DESIGN's settled architectural intent (not just the
> BRIEF's literal spec)? If the BRIEF's spec drifted from
> DESIGN, surface as honest delta and STOP.

This catches BRIEFs that drift from the DESIGN they're
supposedly implementing. The orchestrator drafts the BRIEF; the
scorecard is the verification mechanism that the BRIEF didn't
silently quietly diverge.

Candidate addition to `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6
as a new failure mode (FM 17): **BRIEF spec drifts from DESIGN
intent without scorecard catching it.** Worked example: arc 170
slice 1 (this realization).

## Cross-references

- DESIGN.md lines 102-108 (the settled "fn IS the program"
  framing)
- DESIGN.md lines 484-509 (DESIGN-time conversation log)
- SCORE-SLICE-1.md (immutable; documents 14/14 pass against the
  insufficient scorecard)
- CLOSURE-EXTRACTION.md (the spec doc that carried the
  synthetic-name approach; gets amended for slice 1b)
- `feedback_attack_foundation_cracks.md` — fix the crack now,
  before slice 2 leans on the wrong shape
- `feedback_inscription_immutable.md` — SCORE stays as-is; fix
  ships forward
- `feedback_no_known_defect_left_unfixed.md` — bias is "ship
  everything we know how to do," not "ship the smaller win"
