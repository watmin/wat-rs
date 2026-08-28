# STONE H-1a — holon declares its real arity (subspace · engram · reckoner · hologram)

> Read `DESIGN-STONE-H-holon-adopts-the-kernels-interface.md` first. It carries the measurement,
> the root cause, and what this does NOT touch.

## The work

Every one of holon's 95 handlers declares `args: &[WatAST]` — the **variadic** shape — and then
hand-rolls its own arity check. The collections, before O-iv-b migrated them, declared `m: &WatAST`
— the **fixed** shape — and hand-rolled **zero**, because `#[wat_intrinsic]` generates the check for
fixed-arity handlers and generates nothing for variadic ones.

```
hand-rolled `if args.len() != …`   holon 89   ·   the migrated collections 0
```

**The signature currently lies and the body corrects it.** `(metadata-of :wat::holon::to-holon)`
reports `:arity -1` for a verb whose own doc reads `(:wat::holon::to-holon v)` — verified live. That
is the same defect Stone P2 fixed for `:wat::core::if` this morning, in a second place.

You convert the four smaller files. **`atom.rs` is H-1b and is not yours.**

```
src/intrinsic/holon/subspace.rs   10
src/intrinsic/holon/engram.rs     10
src/intrinsic/holon/reckoner.rs    8
src/intrinsic/holon/hologram.rs    7
                                  35
```

## The shape

```rust
-pub(crate) fn eval_holon_to_holon(args: &[WatAST], env: &Environment, sym: &SymbolTable, list_span: &Span)
-    -> Result<Value, EvalBreak> {
-    if args.len() != 1 { return Err(RuntimeError::new(list_span.clone(), ArityMismatch{…})); }
-    let v = eval_inner(&args[0], env, sym)?.value_owned();
-    to_holon_inner(v, args[0].span())
-}
+pub(crate) fn eval_holon_to_holon(v: &WatAST, env: &Environment, sym: &SymbolTable, _span: &Span)
+    -> Result<Value, EvalBreak> {
+    let val = eval_inner(v, env, sym)?.value_owned();
+    to_holon_inner(val, v.span())
+}
```

Per handler: declare the real parameters, delete the hand-rolled check, rewrite `args[i]` as the
named parameter. **`@arg` names must match the new parameter idents** — `wat_doc::check_args`
enforces it and the build will tell you immediately.

★ **THE COMPILER IS THE INSTRUMENT.** Where `list_span` becomes genuinely unused after the check
goes, the compiler says so — and that verb has just identified *itself* as span-free. Do not guess,
do not grep: **report what the compiler tells you, per verb.** This design has already retracted
three text classifiers for exactly this question.

- `list_span` unused → rename to `_span` and give it the `// rune:lint(unused-span)` rune the
  collections use, with a one-clause reason.
- `list_span` still used → leave it, and say in your report WHAT it is still used for. That list is
  the input to Stone Q's sizing and is one of this stone's two real deliverables.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A verb is genuinely variadic or range-arity** (like `atom.rs`'s `from-holon`, which takes 1 *or*
   3). Leave it `args: &[WatAST]` with its hand-rolled check — that check is honest. Name every one
   you leave and why. Do NOT force a fixed arity onto a verb that does not have one.
2. **Any behaviour changes.** Value AND error text, for every one of the 35, before and after. The
   generated `ArityMismatch` must match the hand-rolled one it replaces — same kind, same `op`, same
   `expected`/`got`. If one differs, STOP and report the difference; do not adjust the expectation.
3. **You touch `atom.rs`.** H-1b.
4. **You touch the `-> :T` runtime annotation** in `from-holon`. It is in `atom.rs` and it is a
   separate, unsettled question — arc 258.4 retired that ascription language-wide and holon still
   implements it. Not here.
5. **A verb's real arity is unclear from its doc and body.** STOP and name it rather than picking one.

## Acceptance — run each, report the actual output

```
 0. ★ METADATA-OF STOPS SAYING -1. For all 35, `(:wat::runtime::metadata-of <verb>)`'s `:arity`
    before and after. Before: -1 for every one. After: the real N (or -1 for any STOP-1 verb you
    correctly left variadic — list those separately). One scratch .wat under
    wat-scripts/scratch-pad/, `--check` clean.

 1. ★ BEHAVIOUR IS BYTE-IDENTICAL. For all 35: a success call and a WRONG-ARITY call, before and
    after, diffed. Build the "before" with `git show HEAD:<path>` — never `git stash`.
    The wrong-arity row is the load-bearing one: it proves the generated check replaced the
    hand-rolled one exactly.

 2. ★ WHAT THE COMPILER SAID ABOUT SPANS. Per verb: did `list_span` become unused? Report the list
    both ways. For every verb where it is STILL used, one line on what it is used for. ⚠ This is a
    deliverable, not a footnote — it is the input to Stone Q.

 3. ★ THE HAND-ROLLED CHECKS ARE GONE.
      grep -c 'args.len() !=' src/intrinsic/holon/{subspace,engram,reckoner,hologram}.rs
    Report before and after. Anything non-zero after must be a STOP-1 verb you named.

 4. cargo build --release --all-targets — clean, and report any warning verbatim even if it does
    not fail the build.

 5. cargo nextest run --release -E 'test(holon) + binary_id(wat::reflection)' — Summary verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone. ⚠ `cargo build` is NOT enough on doc
  prose: a recent stone shipped ten `doc list item overindented` clippy errors that both `build` and
  the floor were blind to. Keep `///` lists plainly formatted.
- You may not spawn sub-agents.
- **No `git stash`, in any form.** `git show HEAD:<path>` for a pre-image.
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. The per-verb span table from row 2. Every
STOP-1 verb you left variadic, with its reason. Then the honest deltas.

This orchestrator has had **three** span classifiers retracted on this exact question in one
afternoon — one of them failed a control I wrote myself. If a claim in this brief does not survive
contact with the disk, that is the most useful thing you can hand back.
