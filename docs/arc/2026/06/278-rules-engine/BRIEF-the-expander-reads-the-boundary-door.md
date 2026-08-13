# BRIEF — the macro expander consults `quote_boundary` instead of its own list

> **Design + rulings:** `DESIGN-STONE-the-expander-reads-the-boundary-door.md`. Read its
> § "The mechanism, grounded", § "THE ONE CONTRACT DECISION" and § "STEP 0". **Do not re-derive
> them.**

## The work, one paragraph

`src/macros/expand.rs:441` carries a hand-rolled three-head set of data forms —
`quasiquote`/`quote`/`holon::literal`. `src/resolve/boundary.rs` is the one place the boundary-head
set is encoded, and its `AllData` arm also names `:wat::core::forms`. Because the expander's copy
omits it, a `forms` block's arguments are macro-expanded in the parent's world when they are data
for another world. Delete the retired `:wat::core::define` from the door's `AllData` arm, then
replace the expander's literal set with a `quote_boundary` consult.

## The ONE contract decision, pinned

**The expander consults the door — it does not get its own list, corrected or widened.**

```rust
if matches!(quote_boundary(head), Boundary::AllData | Boundary::Quasiquote) {
    return Ok(WatAST::List(items, list_span));
}
```

Both variants are named on purpose: `quasiquote` classifies as `Boundary::Quasiquote`, not
`AllData`, and its current behaviour must not change.

## Read in order

1. **`src/macros/expand.rs:435-444`** — the hand-rolled set. This is the subject; it is what you
   replace.
2. **`src/macros/expand.rs:446-468`** — the `MatchesSubject` check. **Copy this shape.** It consults
   the door and its comment states the discipline you are applying (*"so this doesn't drift into a
   second, hand-rolled copy of the same language fact"*). The door is already imported here, so
   step 2 needs no new `use`.
3. **`src/resolve/boundary.rs:80-90`** — `quote_boundary`. The `AllData` arm at `:83` is where
   `":wat::core::define"` comes out in step 1.
4. **`src/rete/validate.rs:449-466`** — `walk_for_make_rule`. **Context only, do NOT edit.** This is
   why the symptom surfaced as a rete error rather than a macro error: it descends every form
   consulting no boundary, and matches the *expanded* `make-rule`. Reading it is how you understand
   the gate; touching it is STOP-2.
5. **`wat-scripts/scratch-pad/probe-arc278-rules-ship-as-declared-payload.wat`** — its header
   carries the verbatim RED and the measured verdict this change must preserve.

## Implementation sketch

```
STEP 1  boundary.rs:83 — drop `":wat::core::define" |` from the AllData arm. Build. The compiler
        names any consumer; the floor names any behavioural one.

STEP 2  expand.rs:441 — replace the three-head literal with
        `matches!(quote_boundary(head), Boundary::AllData | Boundary::Quasiquote)`.
        Keep the early `return Ok(WatAST::List(items, list_span));` exactly as it is.

STEP 3  New gate probe: wat-scripts/scratch-pad/probe-arc278-forms-block-is-inert.wat —
        a `(:wat::core::forms …)` block holding two `defrecord`s and a `defrule` that
        references them, plus a `:user::main` that returns nil. It must LOAD CLEAN.
        Confirm it is RED before step 2 and green after; put both outcomes in its header.

STEP 4  Weigh (below).
```

## Blast radius

`src/macros/expand.rs` + `src/resolve/boundary.rs` + one new probe `.wat`. **No existing `.wat`
should need an edit** — STOP-1 makes that falsifiable.

## ⛔ STOP triggers

1. **STOP-1 — if any existing `.wat` needs an edit, STOP and report which file, which form, and
   what broke.** That file is a live consumer of parent-side expansion inside a `forms` block. It is
   a real finding that changes the design, and it is not yours to migrate.
2. **STOP-2 — do NOT touch `src/rete/validate.rs`.** Its boundary blindness is real, tracked, and a
   separate strike. Conflating them makes a failure in either read as a failure in both.
3. **STOP-3 — do NOT touch `src/resolve/walk.rs`'s `skip(4)`** (task #90). Same root, separate
   strike.
4. **STOP-4 — the expander consults the DOOR.** If you find yourself writing
   `|| head == ":wat::core::forms"`, stop — that is the defect, not the fix.
5. **STOP-5 — if deleting `define` from the `AllData` arm breaks anything, STOP and report.** Do not
   restore it, and do not work around it. A live consumer of a hard-cut form is a finding.
6. **STOP-6 — if the floor moves for any reason other than your new probe, STOP.** Report the
   failing test's entire stdout+stderr block **verbatim** — never a summary, never a `head`/`tail`
   window — and name the exact assertion or match arm that fired. **Do NOT re-run first**: a re-run
   that goes green destroys the only evidence.

## The acceptance gate

1. **★ `probe-arc278-forms-block-is-inert.wat` loads clean.** RED before step 2 with
   `#wat.rete/UnknownFactType … is not a registered fact type` pointing INSIDE the forms block;
   green after. This is the load-bearing row — capture both outputs verbatim in your report.
2. **★ `probe-arc278-rules-ship-as-declared-payload.wat` still prints
   `SUBJECT (helper IN payload) => EVALUATED derived=1` and
   `CONTROL (helper OMITTED)    => CHECK-FAILED`.** The declared-payload proof must survive the
   change that legalises its ergonomic form.
3. **`every_wat_scripts_file_loads` passes** — the whole scratch corpus still parses and
   type-checks on the new runtime.
4. **Quasiquote is untouched** — name a macro-expansion test that exercises a quasiquote template
   and confirm it passes; the two-variant match exists so this cannot regress silently.

## Weigh

Run these yourself, in the FOREGROUND, and report the numbers you read:

```
cargo build --release
./target/release/wat --check wat-scripts/scratch-pad/probe-arc278-forms-block-is-inert.wat
./target/release/wat wat-scripts/scratch-pad/probe-arc278-rules-ship-as-declared-payload.wat
cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'
cargo clippy --release --all-targets
```

The orchestrator runs the full floor centrally after your report — do not run it yourself. Baseline
is **4391 passed / 0 failed**; expect that plus nothing, since your probe is a `.wat` under the
existing loader gate rather than a new test.

## Prior comparable

`228b68fa` — the `closure_extract` `MatchesSubject` arm: the same shape of fix (an arm that should
have consulted the door and didn't), with the same kind of comment recording why the neighbouring
arm's reasoning did not transfer. Copy its register for the comments you leave behind.
