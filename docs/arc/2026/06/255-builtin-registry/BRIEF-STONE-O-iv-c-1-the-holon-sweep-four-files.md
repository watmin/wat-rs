# STONE O-iv-c-1 — the holon sweep: subspace · engram · reckoner · hologram

> The machine is O-iii's, proven on `vector` and again on the collections (O-iv-b, 32 verbs).
> Three stones cleared the way here: **H** gave holon real arities, **Q** gave the value door a
> call span, **O-iv-c-0** made the `require_*` family take `&Value`. Read O-iv-b's brief for the
> shape; this brief covers only what is different.

## The work

Migrate **32 SHELL verbs** to ALGEBRA:

```
src/intrinsic/holon/subspace.rs   10
src/intrinsic/holon/engram.rs     10
src/intrinsic/holon/reckoner.rs    8
src/intrinsic/holon/hologram.rs    4      (3 more in that file are BINDING — see below)
                                  32
```

Every one is pure deletion, because c-0 already did the hard part:

```rust
-pub(crate) fn eval_subspace_dim(s: &WatAST, env: &Environment, sym: &SymbolTable, list_span: &Span)
-    -> Result<Value, EvalBreak> {
-    let s = require_subspace(":wat::holon::OnlineSubspace/dim",
-                             &eval_inner(s, env, sym)?.value_owned(),
-                             list_span)?;
-    let n = s.with_ref(":wat::holon::OnlineSubspace/dim", |s| s.dim())?;
-    Ok(Value::i64(n as i64))
-}
+pub(crate) fn subspace_dim(s: &Value, span: &Span) -> Result<Value, EvalBreak> {
+    let s = require_subspace(":wat::holon::OnlineSubspace/dim", s, span)?;
+    let n = s.with_ref(":wat::holon::OnlineSubspace/dim", |s| s.dim())?;
+    Ok(Value::i64(n as i64))
+}
```

**`&eval_inner(s, env, sym)?.value_owned()` becomes `s`.** Nothing is added.

## The one thing that is NEW here — the trailing `&Span`

O-iv-b's collections were span-free; these are not. **Stone Q made an ALGEBRA fn able to take a
trailing `&Span`, and this is the first sweep to use it.** H already sorted the verbs for you by
naming the parameter:

```
list_span : &Span   the span is USED   → migrate to `fn f(…: &Value, span: &Span)`
_span     : &Span   the span is UNUSED → migrate to `fn f(…: &Value)`, no span at all
```

Per file: subspace 9 used / 1 unused · engram 9 / 1 · reckoner 7 / 1 · hologram 5 / 2 (its counts
span both SHELL and BINDING handlers).

⚠ **Do not carry a span a verb does not use** — the `unused_span_justified` lint will catch it, and
Stone Q-2 is the record of what happens when a stone tries to hold a span it does not read.

## What stays BINDING — 3 in `hologram.rs`

`hologram.rs` has 7 handlers, only 4 of which are SHELL. The other three use `env`/`sym` for more
than evaluating their own arguments — several call `require_encoding_ctx(op, sym, span)`, which
takes the `SymbolTable` itself. **`require_encoding_ctx` is a BINDING marker: a verb that calls it
needs `sym` and can never become ALGEBRA.** Leave those three, name them in your report.

The shell census is the starting point, not the verdict: `wat-scripts/hunt/stone-o-shell-census.awk`.
**The compiler is the verdict** — if a verb you migrate still needs `env` or `sym`, it will say so.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **You need to change the macro.** It is proven on 38 verbs across two sweeps. A needed change
   means this population is not what was measured — STOP and name the verb.
2. **A verb still needs `env` or `sym` after the arg-eval is removed.** It is BINDING; leave it and
   name it. Do not thread `sym` into an ALGEBRA fn.
3. **Any behaviour changes** — value or error text, direct call or through `apply`. All 32 must be
   byte-identical on the direct path. If one differs, STOP; do not adjust the expectation.
4. **A migrated verb carries a `&Span` it does not read**, or drops one it does. Both are lint
   failures and both are wrong.
5. **`atom.rs`.** That is O-iv-c-2.

## Acceptance — run each, report the actual output

```
 0. ★ ALL 32 REACH apply. One scratch .wat under wat-scripts/scratch-pad/ (`--check` clean) calling
    every one of the 32 through `(:wat::core::apply …)`. BEFORE: each reports the O-iv-a diagnostic
    (`registered, but no handler taking EVALUATED arguments…`). AFTER: all 32 answer. Paste both.
    ⚠ Several take an OnlineSubspace / Engram / Reckoner / Hologram — construct them, or if a type
    is genuinely unconstructible from wat, say so and exercise its wrong-TYPE path instead.
    (H-1a found 4 Engram readers unreachable from wat today — a pre-existing gap, not yours.)

 1. ★ DIRECT CALLS BYTE-IDENTICAL. All 32, success and error paths, before and after, diffed.
    Build the "before" with `git show HEAD:<path>` — never `git stash`.

 2. ★ THE SPAN STILL POINTS AT THE CALLER. For one migrated verb that KEEPS its span, trigger its
    `require_*` TypeMismatch from two different lines in one file and show the reported location
    differs. Stone Q bought that; this row proves the sweep did not lose it.

 3. ★ PROVE ONE BY SABOTAGE, ON THE THING ITSELF. Pick a migrated verb, make it return a wrong
    constant, show BOTH doors return it — direct AND apply — restore. Confirm the edit LANDED
    before reading its output.

 4. ★ THE 3 BINDING HANDLERS ARE NAMED and untouched, with the reason each needs `sym`.

 5. ★ REGISTRY POPULATION UNCHANGED at 382. Use the ANCHORED form:
      grep -rhoP '^\s*#\[wat_intrinsic\(\s*"\K[^"]+' src/ --include=*.rs | sort -u | wc -l
    plus the 2 special forms. A verb moves between kinds; none is added or removed.

 6. cargo build --release --all-targets — clean. Report any warning verbatim.

 7. cargo nextest run --release -E 'test(holon) + test(intrinsic) + test(apply)' — Summary verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- **No `git stash`, in any form.**
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. The 3 BINDING handlers with reasons, and the
per-verb span disposition (kept / dropped). Then the honest deltas. Every rider on this chain has
caught a real defect in an orchestrator brief — the last one found my blast radius was drawn around
a file instead of a role, and a whole tenth family member outside it.
