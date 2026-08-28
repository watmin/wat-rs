# STONE O-iv-c-2 — the holon sweep: `atom.rs`

> The last holon wave. Read `BRIEF-STONE-O-iv-c-1-the-holon-sweep-four-files.md` for the shape and
> the design's **"THE ARG-SPAN CLASS IS `apply`'s PERMANENT FLOOR"** section for why this file's
> migratable population is so much smaller than its handler count.

## The work — 16 verbs, not 41

`atom.rs` has 60 handlers. **Only 16 can become ALGEBRA.** The shell census alone would have said
41; O-iv-c-1 proved that census is a candidate list, and applying its lesson here:

```
MIGRATABLE                  16
ARG-SPAN — cannot migrate   25    reads `<arg>.span()`; `Value` carries no span
BINDING (env/sym or ctx)    19
```

⚠ **That table is a CANDIDATE LIST produced by a pattern**, controlled only against O-iv-c-1's five
known refusals. **Verify each verb yourself before migrating it.** If your count differs from
16/25/19, that is a finding — report it; three span classifiers were retracted in one afternoon on
this exact question.

The migration itself is the same pure deletion the last three stones set up:

```rust
-pub(crate) fn eval_holon_atom(v: &WatAST, env: &Environment, sym: &SymbolTable, _span: &Span) -> … {
-    let v = eval_inner(v, env, sym)?.value_owned();
-    …
+pub(crate) fn holon_atom(v: &Value) -> … {
+    …
```

`list_span: &Span` → keep it as a trailing `&Span`. `_span: &Span` → drop it entirely.

## The three disqualifiers — check every verb against all three

1. **`<arg>.span()`** — the handler reads an argument's own source location. `Value` has none, and
   `apply`'s arguments *have no syntax to have a span of*. **This one is permanent** — do not
   "solve" it with the call span; that is a different location and a deliberate downgrade the
   builder has not ruled on.
2. **`require_encoding_ctx`** — takes `&SymbolTable`. A verb calling it needs `sym`. BINDING.
3. **`env` / `sym` used for anything but the arg-eval.** BINDING.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. ⛔ **`eval_holon_from_holon` is out of scope entirely**, as it was for H-1b. Range arity (1 or 3),
   returns `TrackedValue` with `Provenance::RuntimeBuilt`, and parses a runtime `-> :T` annotation
   that **arc 258.4 retired language-wide**. Three unsettled questions in one handler. Say you left it.
2. **A verb reads an argument's span.** Leave it BINDING and name it. Converting it trades
   per-element precision for `apply` reachability — the builder's call, never a rider's.
3. **Any behaviour changes.** Value or error text, direct or through `apply`, for every migrated
   verb. If one differs, STOP.
4. **You need to change the macro.** It is proven on 65 verbs across three sweeps.
5. **A migrated verb carries a `&Span` it does not read**, or drops one it does.

## Acceptance — run each, report the actual output

```
 0. ★ YOUR OWN DISPOSITION TABLE FIRST, before migrating anything. Per verb: MIGRATABLE /
    ARG-SPAN / BINDING, with the reason. Compare against 16/25/19 and report every disagreement.

 1. ★ EVERY MIGRATED VERB REACHES apply. One scratch .wat under wat-scripts/scratch-pad/
    (`--check` clean), before and after. BEFORE: each reports O-iv-a's "registered, but no handler
    taking EVALUATED arguments…". AFTER: each answers.

 2. ★ THE REFUSED ONES STILL REPORT THE DIAGNOSTIC, unchanged. Include several ARG-SPAN and
    BINDING verbs in the same probe as controls — the row proves you refused rather than missed.

 3. ★ DIRECT CALLS BYTE-IDENTICAL for every migrated verb, success and error, before and after,
    diffed. `git show HEAD:<path>` for the pre-image — never `git stash`.

 4. ★ AN ARG-SPAN VERB STILL POINTS AT ITS ARGUMENT. Pick one you refused; trigger its error from
    two call sites and show the location tracks the ARGUMENT, not the call. That is what refusing
    protected. ⚠ `EvalError` carries only `{:kind :message}` — use an UNCAUGHT crash, which prints
    the full `:location`, as O-iv-c-1's rider did.

 5. ★ REGISTRY POPULATION UNCHANGED at 380 + 2 special forms, anchored:
      grep -rhoP '^\s*#\[wat_intrinsic\(\s*"\K[^"]+' src/ --include=*.rs | sort -u | wc -l

 6. cargo build --release --all-targets — clean. Report any warning verbatim.

 7. cargo nextest run --release -E 'test(holon) + test(intrinsic) + test(apply)' — Summary verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally.
- You may not spawn sub-agents.
- **No `git stash`, in any form.**
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Your disposition table with every
disagreement against mine. Then the honest deltas. O-iv-c-1's rider refused five verbs my brief told
it to migrate, and was right to — that is the standard here.
