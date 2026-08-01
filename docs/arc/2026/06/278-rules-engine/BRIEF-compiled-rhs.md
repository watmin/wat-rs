# BRIEF — the RHS compiles once (`CompiledRhs`)

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — it does not suspend you, and
nothing will wake you. There is no notification coming.

You are the ONLY rider in the field, so the build lock is yours: **run cargo in the FOREGROUND and
block on it.** Never background a build or a test and then end your turn — you will simply die with
the numbers unread. Your turn ends when the results are in your hands.

Do not commit, do not push, do not stash, do not revert anything. The orchestrator weighs and
commits.

## The work in one paragraph

`build_insert_fact` (`src/rete/matcher.rs:554`) is called once per derived fact and re-derives a
**static** program every time: it re-validates the `(:wat::rete::insert (:Type …))` form shape,
re-detects kwargs-vs-positional, re-allocates the class `String` from the type keyword, and — via
`resolve_operand` (`:503`) — rebuilds each `?var` lookup key with `Value::String(Arc::new(
name.to_string()))`, a `String` allocation plus an `Arc` allocation, for a key fixed at rule-compile
time. Measured on the fanout cell: **120,000 key allocations, exactly 3.00 per derived fact**, i.e.
240,000 heap allocations rebuilding three constants in one fire. Compile that program **once per
rule at setup** and execute it per fact.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-compiled-rhs.md`** — the design, the contract
   decision, and the affirmative out-of-scope cuts.
2. **`src/rete/compiled_cond.rs`** — **the exemplar.** This exact stone, one layer up, for the LHS:
   a compile-at-setup form plus an executor with the same signature as the interpreter it replaces.
   Mirror its shape, its naming, and its module layout. You are writing its sibling.
3. **`src/rete/matcher.rs:503-540`** (`resolve_operand`) and **`:554-673`** (`build_insert_fact`) —
   what you are compiling away. Note the four `prod:*` phase marks inside; see "the marks" below.
4. **`src/rete/kernel.rs:2067-2083`** — where `compiled_conds` is built at setup. Your
   `CompiledRhs` map is built in the same region, the same way.
5. **`src/rete/kernel.rs:2144-2156`** (`rule_rhs_cache` built) and **`:2748-2752`** (consulted in the
   production pass, where `build_insert_fact` is called). That call site is the one you re-point.

## Implementation sketch — fill it in; do not invent a different shape

In `src/rete/compiled_rhs.rs` (new module, sibling of `compiled_cond.rs`):

```rust
pub(crate) enum RhsOp {
    Bind(Value),   // a PRE-BUILT Value::String key — the whole point; never rebuilt per fact
    Lit(Value),    // a literal, pre-built
}

pub(crate) struct CompiledRhs {
    class: String,        // stripped of the leading ':' ONCE, at compile time
    ops:   Vec<RhsOp>,    // one per field, in written order (kwargs already unwrapped)
}

/// Compile one `(:wat::rete::insert (:Type arg…))` form. All the validation
/// `build_insert_fact` does per fact happens HERE, once.
pub(crate) fn compile_rhs(insert_form: &WatAST) -> Option<CompiledRhs> { … }

/// Execute. Same return shape as `build_insert_fact`.
pub(crate) fn exec_compiled_rhs(
    c: &CompiledRhs,
    bindings: &rpds::HashTrieMapSync<Value, Value>,
) -> Result<Value, EvalBreak> { … }
```

In `kernel.rs`, beside `rule_rhs_cache`, build `compiled_rhs_cache: HashMap<String, Vec<CompiledRhs>>`
at setup; in the production pass call `exec_compiled_rhs` where `build_insert_fact` is called today.

**Keep the defensive fallback pattern** `compiled_conds` uses: if `compile_rhs` returns `None` for a
form, fall back to `build_insert_fact` for that form rather than failing the fire. Copy how
`compiled_conds.get(aid)` handles its `None` arm at `kernel.rs:2217-2226`.

**`build_insert_fact` is NOT deleted.** It stays as the reference implementation and the other half
of the differential — exactly as `alpha_match_inner` did for the LHS.

## The marks

The four `prod:validate` / `prod:shape` / `prod:resolve` / `prod:construct` marks live inside
`build_insert_fact`. On the compiled path most of what they bracket no longer exists per fact. Put a
**single** `phase_end("  ├ prod:compiled-rhs", …)` pair around the compiled execution so the phase
census still has a row for the production pass's fact-building, and leave the four originals where
they are (they still fire on the fallback path). Do not try to preserve all four on the compiled
path — bracketing work that no longer happens would report an instrument, not a measurement.

## ⚠ THE TRAP — `src/rete/kernel.rs:5768`

`fanout_rhs_key_alloc_census` currently ends with:

```rust
assert!(
    get("match:key-alloc") > 0,
    "no key allocations counted — the counter never fired, so any conclusion drawn from a
     zero here would be an artifact.{table}"
);
```

**Your change makes zero the CORRECT answer, so this assertion inverts.** Do NOT delete it and do
NOT weaken it. Re-point it, the way the A8 sharing gate was re-pointed earlier today
(`a8_node_share_fire_census`, HOLD → PRODUCE, in this same file — read it for the shape):

- assert `match:key-alloc` is **exactly 0**, with a message saying what that now proves (the
  compiled RHS rebuilds no keys);
- **keep a non-vacuity guard**, because a fire that never ran also yields zero. Assert
  `prod:derivations == 40_000` (the counter already exists) so the zero cannot be an artifact of a
  dead fire;
- update the doc comment above the test — it currently says every remaining count is the production
  pass, which stops being true when the count is zero. A gate whose prose outlives its property is
  the defect, even when the assertion still fires.

## Blast radius

`src/rete/compiled_rhs.rs` (new), `src/rete/matcher.rs`, `src/rete/kernel.rs`, and the one test
above. No new dependencies. Nothing under `wat/` — the oracle stays naive by ruling. No changes to
`Token`, `Element`, or any binding representation.

## STOP triggers — each rejects; none lets you ship less

1. **STOP-1** — a `:then` form shape exists that `compile_rhs` cannot represent with `Bind`/`Lit`.
   Do not invent a third op and do not silently fall through in a way that hides it: make it return
   `None` (so the fallback runs), and **report the shape** with a `file:line` example.
2. **STOP-2** — the differential disagrees: `exec_compiled_rhs` and `build_insert_fact` produce
   different class, field values, or field order for the same input. STOP and report both values.
   Do not adjust the test to accommodate a difference.
3. **STOP-3** — you find yourself needing to change `Token`, `Element`, `resolve_operand`'s
   signature, or anything under `wat/`. That is outside this stone; STOP and report.

## What to verify, in the foreground, before you report

- `cargo nextest run --release -E 'test(fanout_rhs_key_alloc_census)' --no-capture` — read the count.
- `cargo nextest run --release` — the whole floor. Read the **Summary line**, never a piped exit code.
- `cargo clippy --release --workspace --all-targets -- -D warnings` — read its own exit code.

## Your report

- `match:key-alloc` before and after, and `prod:derivations`.
- The floor's Summary line verbatim, and clippy's exit code.
- The re-pointed assertion, quoted as you left it.
- Anything a STOP fired on.
