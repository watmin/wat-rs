# STONE P6-a — a special form names its implementations

> **Builder's ruling, 2026-08-28: _"we finish it -- this check, eval, tail struct is agreeable."_**
> Design and derivation: `NOTE-a-special-form-declaration-names-none-of-its-three-implementations.md`.
> Read it first — especially *"one record, not one call"* and the `serve-dispatch-op` precedent.

## The work

A special form's declaration is a doc-only unit struct that names none of the code that runs:

```rust
#[wat_special_form(":wat::core::if")]
pub(crate) struct If;                    // ← names nothing
```

while the form actually IS three named Rust fns across two phases:

```
:wat::core::if    check  infer_if       src/check.rs:7444
                  eval   eval_if        src/runtime.rs:9221
                  tail   eval_if_tail   src/runtime.rs:4554
:wat::core::let   check  infer_let      src/check.rs:7718
                  eval   eval_let       src/runtime.rs:8682
                  tail   eval_let_tail  src/runtime.rs:4611
```

You give each implementation an annotation that names the form and its role, mirroring exactly what
`#[wat_intrinsic]` already does for 380 handlers, and `show-source` starts printing all three.

**This stone is the MECHANISM, proven on the two registered forms. It does NOT collapse the eval or
tail match — that is P6-c and it is not yours.**

## Row 0 — the census, before any code

The NOTE deliberately refuses to repeat the 294 seam's figure of "36 special forms" because it was
never validated. **Measure it, with an instrument you state.** Two questions:

1. How many special-form heads exist across `src/runtime.rs`'s eval match, its tail match, and
   `src/check.rs`'s `infer_*` match — as a set of FQDNs, deduplicated?
2. For each, which of `check` / `eval` / `tail` does it have?

**STOP-0: if the role axis does not fit** — a head that needs a fourth role, or a form whose check
and eval FQDNs disagree — say so and stop before touching the macro. The record shape the builder
agreed to is `{check, eval, tail}`; a population that does not fit it is a finding that must reach
him before it is built around.

⚠ Report the instrument, not just the number. Four counts of one population were wrong in this arc
because a grep matched attribute text inside doc comments.

## The ONE CONTRACT DECISION — a third inventory stream, keyed by (FQDN, role)

A proc-macro sees only the tokens of the item it annotates, so the struct's attribute **cannot**
capture a fn in another file. **Put the annotation on each implementation:**

```rust
#[wat_special_form_impl(":wat::core::if", role = check)]  fn infer_if(…)     { … }
#[wat_special_form_impl(":wat::core::if", role = eval)]   fn eval_if(…)      { … }
#[wat_special_form_impl(":wat::core::if", role = tail)]   fn eval_if_tail(…) { … }
```

Each emits `inventory::submit!` of a **new** type:

```rust
pub(crate) enum SpecialFormRole { Check, Eval, Tail }

pub(crate) struct SpecialFormImplSubmission {
    pub name: &'static str,            // the form's FQDN
    pub role: SpecialFormRole,
    pub source: &'static str,          // quote!(#item).to_string() — as wat_intrinsic.rs:565 does
}
inventory::collect!(SpecialFormImplSubmission);
```

`registry()` gathers this third stream and folds it into the entry.

⚠ **Bucket the stream ONCE into a `HashMap<&str, Vec<…>>` before the special-form loop**, then drain
per form. Iterating the whole stream inside the loop is O(n·m) and, more importantly, reads as if the
association were incidental rather than keyed.

⚠ **`IntrinsicEntry`'s fields are all `&'static` today.** The gathered impls are built at fold time,
so this field is owned (a `Vec`), and that is fine — `registry()` is a `OnceLock` that owns its
entries. Say so in a comment; the next reader will notice the asymmetry.

⚠ **The annotated fns must not otherwise change.** `#[wat_intrinsic]` passes its item through
unchanged (`#item` in the emitted tokens); do the same. `infer_if` and `eval_if` keep working exactly
as they do now — this stone adds a submission, it does not reroute a call.

## ★ THE WALL — and it is what makes "finished" checkable

Add a `#[test]` that asserts **every `Kind::SpecialForm` entry carries at least a `check` and an
`eval` impl**. Without it, `impls.is_empty()` silently means both *"this form has no source"* and
*"nobody annotated it yet"* — the exact absence-read-as-an-answer this NOTE family exists to kill,
re-created inside the stone that answers it.

`tail` is **not** required: 8 of the eval match's heads have a tail rule and the rest fall through to
`eval_inner`, which is correct and merely not tail-optimized. `tail: None` is an honest absence — see
the NOTE's discriminator table.

With only `if` and `let` registered, this wall is green the moment the six annotations land, and it
is what will fail loudly when P6-c registers the rest.

## `show-source`'s new output

Print all impls, labelled by role, in `check → eval → tail` order. When `impls` is empty, keep
**exactly** P2's existing text — do not invent a second wording for "no source available".

```
;; :wat::core::if — special form (check · eval · tail)

;; role: check
fn infer_if(…) { … }

;; role: eval
fn eval_if(…) { … }

;; role: tail
fn eval_if_tail(…) { … }
```

⚠ The macro cannot know a file path (`proc_macro::Span::source_file` is unstable) — **do not fake
one.** The fn name is in the captured source; that is the locator.

## Rooms — verified against `28407c99d`

```
crates/wat-macros/src/wat_special_form.rs      161 lines — the sibling macro; mirror its shape
crates/wat-macros/src/wat_intrinsic.rs:565     `quote!(#item).to_string()` — the capture to copy
crates/wat-macros/src/lib.rs                   where a new proc-macro is exported
src/intrinsic/mod.rs   SpecialFormSubmission   — has NO `source` field; that is why "" is hardcoded
src/intrinsic/mod.rs   the special-form fold    — `source: ""`, and where the third stream is gathered
src/intrinsic/mod.rs   struct IntrinsicEntry    — gains the impls field
src/intrinsic/mod.rs   Kind, Arity              — where SpecialFormRole belongs, beside them
src/intrinsic/reflect.rs  eval_show_source      — P2's Kind::SpecialForm gate; it gives way to this
src/check.rs:7444 infer_if · :7718 infer_let
src/runtime.rs:9221 eval_if · :4554 eval_if_tail · :8682 eval_let · :4611 eval_let_tail
```

## Blast radius

`crates/wat-macros/` (a new macro + its export), `src/intrinsic/mod.rs`, `src/intrinsic/reflect.rs`,
and **six annotation lines** in `src/check.rs` and `src/runtime.rs`. No eval-match change, no
tail-match change, no behaviour change to any form.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

0. **Row 0's role axis does not fit the population.** See above.
1. **Any form's behaviour changes.** This stone adds submissions and a reflection surface. If `if` or
   `let` evaluates differently, or a type-check result moves, STOP.
2. **You reroute a call.** The eval and tail matches keep calling the same fns directly. Collapsing
   them into registry lookups is P6-c. If you find yourself editing either match, STOP.
3. **The wall cannot be made green** — a registered special form without a check or eval impl. That
   is a finding about the population, not a reason to weaken the test.
4. **`impls.is_empty()` grows a second meaning.** One wording for "no source available", P2's.
5. **You need `proc_macro::Span::source_file()` or another unstable API.** STOP; report what you
   wanted it for.

## Acceptance — run each, report the actual output

```
 0. ★ THE CENSUS. The number, the instrument, and the per-form role table. STOP-0 if it does not fit.

 1. ★ show-source PRINTS THREE IMPLEMENTATIONS. Scratch .wat under wat-scripts/scratch-pad/
    (loader-gated, must `--check` clean), before and after, for `:wat::core::if` AND
    `:wat::core::let`. Before: P2's "no source available" line. After: three labelled blocks whose
    bodies are the real `infer_*` / `eval_*` / `eval_*_tail` fns.

 2. ★ THE WALL GOES RED WHEN IT SHOULD. Remove ONE annotation (say `eval_let`'s), show the new
    test FAILS and NAMES `:wat::core::let` and the missing role; restore; show green.
    `NISI FRANGAS, NIHIL PROBAS.` Confirm each edit LANDED before reading its output.

 3. ★ NOTHING ELSE MOVED. `metadata-of` and `render-doc` for `if` and `let` byte-identical before
    and after — P2 set `:arity 3` / `-1` and the "Syntax:" line; both must survive.
    An INTRINSIC's `show-source` (`:wat::i64::+`) byte-identical too.

 4. ★ THE FORMS STILL RUN. `(:wat::core::if true 1 2)` → 1, `(:wat::core::if false 1 2)` → 2,
    and a `let` with sequential binders. Plus a TAIL-position case for each, deep enough to prove
    the TCO path is untouched — the tail match still calls `eval_if_tail` directly.

 5. cargo build --release --all-targets — clean.

 6. cargo nextest run --release -E 'binary_id(wat::reflection) + test(special_form) + test(show_source) + test(metadata) + test(tco)'
    Summary lines verbatim. A green test going red may be a golden pinning P2's output — report the
    assertion, do not edit the golden.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers before your
  turn ends — a rider on this chain was lost mid-strike and left an implementation with no evidence.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- **No `git stash`, in any form.** Use `git show HEAD:<path>` for a pre-image.
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Then the honest deltas. Every rider on this
chain has caught a real defect in an orchestrator brief — one refuted its opening premise outright.
That is the most useful thing you can hand back.
