# BRIEF — STONE: the tail door

Give `role = tail` a callable pointer and a guard to call it from. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-tail-door.md` — read its § "TWO placement
facts" before touching `eval_tail`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5119, HEAD `bd291e5c2`.

⚠ **This stone changes a proc macro you cannot compile**, and its guard has a placement that fails
*silently* if wrong. Mirror existing codegen; report anything you are unsure of.

## Read in order

1. The DESIGN's § "TWO placement facts the guard-hoist contract could not supply" and
   § "THE ONE CONTRACT DECISION".
2. **The eval door's shipped code** — `crates/wat-macros/src/wat_special_form_impl.rs`'s `emit`, and
   `src/intrinsic/mod.rs`'s `eval_handler` fold. This stone is its sibling; copy the shape.
3. **`src/runtime.rs`'s `eval_tail`** (lines ~932–1083). Read the whole function before editing it.
   The rete `Form` re-mapping a few lines above `match head {` is the placement constraint.
4. `crates/wat-macros/src/wat_intrinsic.rs`'s `sniff_return` — you reuse the DETECTION, not the
   wrapping. See § 1's note.

## The work

### 1 — `role = tail` carries a pointer

`SpecialFormImplSubmission` gains `tail_handler: Option<TailHandler>`, alongside the `eval_handler`
the eval door added. The macro emits it **only for `role = tail`**.

```rust
pub(crate) type TailHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;
```

⚠ **The wrapping is the INVERSE of the eval door's, so it cannot share `wrap_call_for_return`.**
That helper wraps *to* `TrackedValue`; a tail shim must produce `Value`:

```
fn returns Result<Value, EvalBreak>        → pass through
fn returns Result<TrackedValue, EvalBreak> → .map(|tv| tv.value_owned())
```

★ **Reuse `sniff_return` — the detection — and write the inverse wrap beside it.** Last stone's
STOP-4 demanded sharing because the decision was identical; here the decision is the same and the
*action* is opposite, so a sibling helper is correct. Say in your report which you shared and which
you wrote fresh, and why.

### 2 — a separate slot on the entry

`IntrinsicEntry` gains `tail_handler: Option<TailHandler>`. ⛔ **Not folded into `handler`.**
`handler` is what `dispatch_keyword_head_value` calls; a tail impl invoked there would run in
non-tail position where its contract does not hold.

### 3 — the guard, in the one place it works

In `eval_tail`, **after** the rete `Form` re-mapping block and **immediately before** `match head {`:

```rust
if let Some(entry) = crate::intrinsic::registry().lookup_entry(head) {
    if let Some(tail) = entry.tail_handler {
        return tail(args, &list_span, env, sym);
    }
}
match head { … }
```

⛔ Above the re-mapping it would miss every `:wat::rete::core::*` `Form`-class spelling. **And it
would not fail** — the fallthrough still evaluates correctly, just without TCO, with every test
green. That is why the placement is the stone.

### 4 — delete exactly three arms

`:wat::core::if`, `:wat::core::let`, `:wat::core::match` in `eval_tail`. One-line retirement note
each, in the shape the eval door used.

⛔ **`:wat::core::do`, `and`, `or`, `ann-form` and `:wat::rete::insert` KEEP THEIR ARMS.** They carry
no registry row, so their arm is their only tail dispatch. STOP-2.

### 5 — the placement probe, as a durable artifact

Write `wat-scripts/scratch-pad/255-tail-door-rete-form-spelling.wat`: deep tail recursion whose
recursive call sits in tail position under a **rete `Form`-class spelling** of a form whose arm this
stone deletes. It must terminate rather than grow the stack. The orchestrator runs it; you are
writing the artifact, not the verdict.

★ Consult `src/rete/vocabulary.rs`'s `RETE_OPS` for a real `OpClass::Form` row and use its actual
`rete_name` — do not invent a spelling.

## Blast radius

`crates/wat-macros/src/wat_special_form_impl.rs` · `src/intrinsic/mod.rs` (type, submission field,
entry field, fold) · `src/runtime.rs` (`eval_tail`: one guard, three arms out) · the three
`#[wat_special_form_impl(role = tail)]` sites · one new scratch-pad `.wat` · whatever the compiler
names. **No verb changes behaviour — the same three fns run, reached from the registry.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — THE GUARD GOES AFTER THE RETE RE-MAPPING.** Not at the top of `eval_tail`, not before
the `WatAST::Keyword` match. If you place it anywhere the local `head` has not yet been rewritten
from `:wat::rete::core::X` to `:wat::core::X`, the door silently loses TCO for every rete `Form`
spelling and nothing goes red.

**⛔ STOP-2 — FOUR ARMS AND `rete::insert` STAY.** `do`, `and`, `or`, `ann-form`, `:wat::rete::insert`
have no registry row. Deleting their arms strips TCO from live forms. Only `if`, `let`, `match` go.

**⛔ STOP-3 — DO NOT FOLD `tail_handler` INTO `handler`.** Separate slot. The eval door folded on
purpose and this one must not; the DESIGN's contract decision says why.

**⛔ STOP-4 — DO NOT REUSE `wrap_call_for_return`.** Its direction is opposite here. Reuse
`sniff_return` for the detection and write the inverse wrap. ⚠ If you find yourself making
`wrap_call_for_return` take a direction flag, stop and report — that is one function answering two
questions.

**⛔ STOP-5 — `step_list` IS NOT PART OF THIS.** It was measured and refuted as a door: a closed
19-name competence table whose `NoStepRule` fallthrough is correct. Its arms stay. Do not add a
guard, a role, or a `StepHandler`.

**STOP-6 — you cannot compile a proc macro or run the probe.** Mirror the eval door's codegen
exactly. Report every construct copied vs invented, and report your placement reasoning as
**unverified**, as the last four riders correctly did.

## Report

Per-file diff summary; the macro change verbatim; **which helper you shared and which you wrote
fresh, per STOP-4**; the guard verbatim **with the lines immediately above it**, so the orchestrator
can see it sits below the re-mapping; the three arm deletions; confirmation the four unregistered
arms and `rete::insert` are untouched; the probe `.wat` verbatim and the `RETE_OPS` row you took its
spelling from. Then: **what surprised you** — a tail fn whose signature did not match the DESIGN's
table, a rete `Form` row that does not round-trip to a form this stone touches, or a fold in
`registry()` that could not take a second pointer cleanly.
