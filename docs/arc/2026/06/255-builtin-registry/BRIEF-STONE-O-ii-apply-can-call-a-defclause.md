# STONE O-ii — `apply` can call a defclause

> Read `DESIGN-STONE-O-one-declaration-feeds-both-doors.md` first — its ⛔ CORRECTION banner carries
> this strike's whole motivation, and the builder's pushback that surfaced it.

## The work

`:wat::core::apply` **cannot apply a defclause.** Not a subset — none of them:

```
(:wat::core::apply :wat::core::+ [1 2 3])
  →  err ":wat::core::apply: expected wat::core::keyword, got wat::core::clauses <clauses::wat::core::+/25>"
```

**29 defclauses exist, 22 of them production** — `+ - * / reduce sort sort-by into filterv mod quot
rem run! reductions nth-spec` and more. `(apply reduce …)`, `(apply sort …)`, `(apply + …)` — the
reason `apply` exists — are all refused. This is the entire user-facing arithmetic and
higher-order surface.

★ **The door you need is already built.** `dispatch_keyword_head` handles a clause-set head
(`src/runtime.rs:6758`) by calling `eval_call_to_defclause`, which evaluates the ASTs and hands off
to **`eval_call_to_defclause_with_vals(cs, vals, list_span, sym)`** — a value-level entry that takes
`Vec<Value>` and needs **no `env` at all**. `eval_apply` already holds exactly that: its `combined`
is a `Vec<Value>`. **You are wiring an existing value-level entry to a caller that already has the
values.** No new dispatch logic, no clause-selection code, no duplicated loop.

## Rooms — verified against `fe602d707`

```
src/runtime.rs:10605   fn eval_apply                     — the function you edit
src/runtime.rs:10664   Step 5 — the fn-valued head arm   — ★ COPY THIS SHAPE EXACTLY; yours sits beside it
src/runtime.rs:10669   Step 6 — the keyword gate         — where a clauses head dies today
src/runtime.rs:8064    fn eval_call_to_defclause         — the AST-level entry; NOT what you call
src/runtime.rs:8307    fn eval_call_to_defclause_with_vals(cs: Arc<ClauseSet>, vals: Vec<Value>,
                                                           list_span: &Span, sym: &SymbolTable)
                                                         — ★ THIS is what you call
src/runtime.rs:6758    dispatch_keyword_head's clauses arm — the precedent: the direct path has had
                                                             this arm since Stone 237.2
```

## Implementation sketch

Step 5 already handles a fn-valued head, one line above the keyword gate that rejects everything
else. Your arm is its sibling:

```rust
    // Step 5 — fast path: fn-valued head (Arc 009 lift OR let-bound fn).
    if let Value::wat__core__fn(func) = &head_val {
        return apply_function(func.clone(), combined, sym, list_span).map_err(Into::into);
    }

    // Stone O-ii — clause-set head. `dispatch_keyword_head` has had this arm since Stone 237.2
    // (runtime.rs:6758); `apply` never grew it, so every defclause — `+`, `reduce`, `sort` — was
    // refused by the keyword gate below. `combined` is already the evaluated args, which is
    // precisely what the value-level entry wants.
    if let Value::wat__core__clauses(cs) = &head_val {
        return eval_call_to_defclause_with_vals(cs.clone(), combined, &list_span, sym);
    }

    // Step 6 — keyword-valued head: extract name + dispatch chain.
```

⚠ **Mind `list_span`'s type at your line.** Step 5's `apply_function(…, list_span)` takes it by
value; `eval_call_to_defclause_with_vals` takes `&Span`. Read the surrounding code and pass what
each wants — do not clone defensively, and do not change either signature.

⚠ **Do NOT route through `eval_call_to_defclause`** (the AST entry, `:8064`). It expects
`&[WatAST]` and would need `env` to re-evaluate arguments `apply` has already evaluated — the exact
impedance mismatch this whole stone is about. Calling it would mean re-deriving ASTs that do not
exist: `apply`'s arguments have no syntax.

## What this strike does NOT do

- **It does not touch the arity story.** `select_defclause_clause` already reports arity failures per
  clause (`NoMatchingClause` with a `ClauseAttempt` per arm), so a wrong-arity `apply` on a defclause
  errors today rather than panicking. Stone O-i covers the intrinsic value door; the two are
  independent and neither waits on the other.
- **It does not touch the special-form rejection** at Step 7. A defclause is not a special form.
- **It does not change `dispatch_keyword_head`.** The direct path is already correct; this strike
  makes `apply` agree with it.

## Blast radius

`src/runtime.rs`, inside `eval_apply`, between Step 5 and Step 6. Nothing else.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A defclause called through `apply` disagrees with the same call made directly.** For every verb
   you test, `(apply f [a b c])` must equal `(f a b c)` — value AND error text. If any differs, STOP
   and report; do not adjust the expectation.
2. **You need `env` to make it work.** `eval_call_to_defclause_with_vals` does not take one; if you
   find yourself needing an `Environment`, you are on the AST entry by mistake, or a clause body
   needs binding state `apply` cannot supply. Either way STOP and name which.
3. **You need to change a signature** — of `eval_call_to_defclause_with_vals`, `eval_apply`, or
   anything they call. STOP and report; the value-level entry was built for exactly this.
4. **A previously-working `apply` call changes.** The two committed probes record today's outputs in
   their headers. Any row that moves other than the DOOR 1 rows is a regression — STOP.
5. **Special forms become applicable.** Step 7's rejection list must keep rejecting. If your arm
   fires before Step 7 for anything on that list, STOP.

## Acceptance — run each, report the actual output

```
 0. ★ THE THREE DOOR-1 ROWS FLIP, AND NOTHING ELSE MOVES.
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-apply-has-three-broken-doors.wat
    DOOR1 (apply + [1 2 3])  ERR -> ok:6
    DOOR1 (apply * [2 3])    ERR -> ok:6
    DOOR1 (apply sort [v])   ERR -> ok:<the sorted vector>
    Every other row IDENTICAL — including DOOR2's two, which are Stone O-iii/iv's, not this
    strike's. Paste the whole run. Do not edit the probe.

 1. ★ APPLY AND THE DIRECT CALL AGREE — the property, not one example. For at least SIX defclauses
    spanning more than arithmetic (use `+`, `*`, `-`, `sort`, `into`, `reduce` or `filterv`), show
    `(apply f [args])` and `(f args)` side by side, identical. Write it as a scratch .wat under
    wat-scripts/scratch-pad/ (loader-gated: it must `--check` clean). Include a variadic case with
    THREE args and the zero-arg identity `(apply + [])` → 0.

 2. ★ PROVE IT BY BREAKING THE DOOR. Comment out your new arm, re-run row 0, show the three DOOR1
    rows go back to ERR, restore. Confirm the edit LANDED before reading its output.

 3. ★ THE OTHER PROBE IS UNTOUCHED.
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat
    Identical to today's output, every row.

 4. ★ SPECIAL FORMS STILL REFUSED. Pick two from Step 7's SPECIAL_FORMS list and show
    `(apply <special-form> …)` still returns its MalformedForm diagnostic.

 5. cargo build --release --all-targets — clean.

 6. cargo nextest run --release -E 'binary_id(wat::wat_lang)' plus any test naming apply, defclause,
    or clause. Report the Summary line verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything in the FOREGROUND. Your turn ends when the numbers are in your hands, not when a
  command is launched.
- You may run `cargo build`, `./target/release/wat --check`, `./target/release/wat <file>`, and a
  scoped `cargo nextest run --release -E '<filter>'`. The orchestrator runs the full floor and
  clippy centrally — leave those alone.
- You may not spawn sub-agents.
- Do not commit, push, stash, revert, or create a worktree. Leave the tree dirty; the orchestrator
  weighs and commits.
- If a number surprises you, report the surprise. This whole stone exists because the builder
  refused a framing of mine that measured correctly and described the wrong subject. A brief that
  turns out to be wrong is the most useful thing you can hand back.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Then the honest deltas — what surprised you,
what this brief got wrong, what you had to decide that it did not settle.
