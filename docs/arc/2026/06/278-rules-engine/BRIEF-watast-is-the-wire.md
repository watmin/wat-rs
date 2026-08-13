# BRIEF — a declared `:wat::WatAST` accepts any well-formed EDN

> **Design + rulings:** `DESIGN-STONE-watast-is-the-wire.md`. Read its § "The law", § "THE ONE
> CONTRACT DECISION", § "Where it lands" (including the adjacent-implementation trap), and § "What
> this does NOT weaken". **Do not re-derive them.**

## The work, one paragraph

Only a `WatAST` can cross the wire — an `i64` crosses because an i64 *is* a WatAST. A declared field
type is a **refinement applied after decode**, not a gate on whether the value may cross. For
`:wat::WatAST` that refinement is the **identity** and can never fail, but the decode walker does not
implement it: handed a form (which crosses as a bare untagged EDN list), it reports
`expected=:wat::WatAST got=List` and the request is refused. Add the identity arm.

## The ONE contract decision, pinned

**In the decode walker, a declared `:wat::WatAST` accepts any well-formed EDN value.** Not a tag on
write; not a special case in `defservice`; one arm in the walker every op already routes through.

The precedent to mirror is in our own source — R7's universal top, `src/types.rs:5212`:

```rust
// :wat::core::Value is the universal subtype-top: every type <: Value.
if sup == ":wat::core::Value" { return true; }
```

Same move, one domain over. Leave a comment in that register.

## Read in order

1. **`wat/service.wat:1437`** — `shape-guarded`, where `defservice` emits
   `(:wat::edn::validate ~req-binder ~req-ty-kw)` into every op arm unconditionally. Context: this is
   why one arm fixes every service. **Do not edit it.**
2. **`src/runtime.rs:15213`** — `eval_edn_validate` and its doc. It states outright that it is a thin
   wrapper over `edn_shim::edn_to_typed_value`, "the deep walker … per-field / per-element … yields
   the offending path". Dispatched at `runtime.rs:4726`.
3. **`edn_shim::edn_to_typed_value`** — **the subject.** Find its `TypeExpr::Path` arm. It is the
   walker that produced `path ["defs" "[0]"]`.
4. **`src/runtime.rs:15307`** — `conforms_check`'s `TypeExpr::Path` arm. **CONTEXT ONLY — this is the
   TRAP, not the subject.** Near-identical shape, returns a bare `bool`, yields no path; `validate`'s
   own doc says `conforms?` "cannot serve here". Confirm your target by the PATH it emits, never by
   resemblance.
5. **`src/types.rs:5212`** — R7's one-branch universal top, the comment register to copy.
6. **`wat-scripts/scratch-pad/probe-arc278-watast-on-the-wire-decomposed.wat`** — its header carries
   the full measured chain including the wrong turn taken and superseded.

## Implementation sketch

```
STEP 1  In `edn_shim::edn_to_typed_value`'s TypeExpr::Path arm: when the declared name is
        `:wat::WatAST` (strip the leading ':' the way the surrounding code does), accept the
        value as-is. The identity refinement — it cannot fail.

STEP 2  Verify the BARE case as well as the parametric one. `Vector<WatAST>` reaches the arm
        per-element; a bare `[form <- :wat::WatAST]` field reaches it directly. Only the
        parametric path has been measured — if the bare case takes a different route, say so.

STEP 3  Run the two existing probes (below). Rewrite the red-by-measurement verdict in
        `probe-arc278-rules-cross-the-wire.wat`'s header to what it then proves — its header
        instructs you to, and a verdict kept after it turns is a defect.

STEP 4  Add the negative row: a genuinely wrong field must still be refused (gate row 4).
```

## Blast radius

The one walker arm, plus probe-header updates. **No `defservice` change, no `wat/` change, no corpus
migration.** If you find yourself editing `wat/service.wat` or adding a tag to the EDN writer, stop —
both are ruled out in the stone.

## ⛔ STOP triggers

1. **STOP-1 — do NOT tag `WatAST` on write.** The stone rules this out with prior art
   (Chapter 59): a tag re-mints at the boundary the escape hatch that chapter deleted. If the
   identity arm seems impossible without a tag, STOP and report why.
2. **STOP-2 — do NOT edit `conforms_check`.** It is the adjacent walker, not the subject. If your
   change makes a `conforms?` test move, you are in the wrong function.
3. **STOP-3 — the identity applies to `:wat::WatAST` ALONE.** If a wrong field type (`[n <- i64]`
   handed a String) stops being refused, you have widened the hole instead of closing the edge.
   That is gate row 4 and it is falsifiable — run it.
4. **STOP-4 — do NOT chase the locus asymmetry.** Thread faces `RequestMalformed`; process reports
   a bare `LOST disconnected`. Real defect, separately tracked, and your fix will make this path stop
   firing — which HIDES it. Leave it. Report it if you touch it accidentally.
5. **STOP-5 — if the floor moves for any reason other than your probes, STOP.** Report the failing
   test's entire stdout+stderr block **verbatim** — never a summary, never a `head`/`tail` window —
   and name the exact assertion or arm. **Do NOT re-run first**: a re-run that goes green destroys
   the only evidence.

## The acceptance gate

1. **★ `wat-scripts/scratch-pad/probe-arc278-rules-cross-the-wire.wat`**:
   `SUBJECT (helper IN payload) => DERIVED n=1` and `CONTROL (helper OMITTED) => REJECTED
   check-failed`. Red today. This is the load-bearing row — a rule and the fn its `where` calls,
   crossing a process boundary and firing.
2. **★ `wat-scripts/scratch-pad/probe-arc278-watast-on-the-wire-decomposed.wat`**: all three arms
   `Ok` — `echo(i64) Ok n=7`, `count(Vec<WatAST>) Ok n=3`, `count THREAD Ok n=3`.
3. **A bare `:wat::WatAST` field crosses** (STEP 2). Add the smallest probe that shows it.
4. **★ THE NEGATIVE ROW — nothing else loosened.** A field declared `i64` handed a String must still
   come back `RequestMalformed`. Show it, don't assert it.
5. `every_wat_scripts_file_loads` passes; clippy clean.

## Weigh

Run these yourself, in the FOREGROUND, and report the numbers you read:

```
cargo build --release
./target/release/wat wat-scripts/scratch-pad/probe-arc278-watast-on-the-wire-decomposed.wat
./target/release/wat wat-scripts/scratch-pad/probe-arc278-rules-cross-the-wire.wat
cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'
cargo clippy --release --all-targets
```

The orchestrator runs the full floor centrally after your report — do not run it yourself. Baseline
is **4391 passed / 0 failed**.

## Prior comparable

`f1a811cb` (this arc, today) — the macro expander made to consult `resolve::boundary` instead of its
own hand-rolled set: same shape of fix (one place re-deriving a language fact the substrate already
encodes once), same discipline in the comment left behind.
