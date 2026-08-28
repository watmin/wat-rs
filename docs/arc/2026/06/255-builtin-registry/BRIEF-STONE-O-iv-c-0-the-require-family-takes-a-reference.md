# STONE O-iv-c-0 — the `require_*` family takes a reference

> The stepping stone under the holon sweep. Read the design's **"H-1a SHIPPED"** and
> **"H-1b SHIPPED"** sections first (`DESIGN-STONE-H-holon-adopts-the-kernels-interface.md`) —
> H left holon in the exact shape this stone completes.

## Why this exists

After Stone Q, a holon verb can become ALGEBRA — `fn f(s: &Value, span: &Span)` — and the macro
generates both doors. Post-H, the shape is already this close:

```rust
pub(crate) fn eval_subspace_dim(s: &WatAST, env: &Environment, sym: &SymbolTable, list_span: &Span)
    -> Result<Value, EvalBreak> {
    let s = require_subspace(":wat::holon::OnlineSubspace/dim",
                             eval_inner(s, env, sym)?.value_owned(),   // ← the only env/sym use
                             list_span)?;
    let n = s.with_ref(":wat::holon::OnlineSubspace/dim", |s| s.dim())?;
    Ok(Value::i64(n as i64))
}
```

**But `require_subspace` takes `v: Value` — by value.** An ALGEBRA fn holds `&Value`, so every one of
**109 call sites** across holon would have to write `s.clone()`. The sweep would *add* 109 clones
instead of *deleting* 73 eval lines.

**Change the family to take `&Value` first**, and the sweep becomes pure deletion.

## The work

Nine functions in `src/holon/require.rs` change `v: Value` → `v: &Value`:

```
require_hologram · require_fn · require_vector · require_subspace · require_reckoner
require_engram · require_engram_library · require_string · require_numeric
```

Each body matches on `v`; with a reference it matches on the reference and clones what it extracts.
Read them before assuming — measured, two shapes:

- **Arc extractors** (`require_subspace`, `require_reckoner`, `require_engram`, …) currently MOVE an
  `Arc` out: `Value::OnlineSubspace(s) => Ok(s)`. With `&Value` that becomes `Ok(s.clone())` — **an
  `Arc` refcount bump, not a deep copy.**
- **Primitive extractors** (`require_numeric`, `require_string`) copy or clone already.

Then update the **109 call sites** (`hologram` 14 · `subspace` 17 · `reckoner` 19 · `engram` 19 ·
`atom` 40) to pass `&`. Mechanical; the compiler finds every one.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A `require_*` cannot take a reference without a deep clone.** An `Arc` bump is fine; cloning a
   `String`, a `Vec`, or a collection body is not — that would trade 109 clones for a worse 109.
   STOP and name it; that one keeps its by-value signature and the sweep clones at its sites.
2. **Any behaviour changes.** Value AND error text, everywhere. This stone changes ownership, not
   semantics. If a `TypeMismatch`'s rendered `got` differs — `ValueSnapshot::of(&other)` now sees a
   reference where it saw an owned value — STOP and report; that is exactly the kind of subtle
   move this row exists to catch.
3. **You migrate a verb to ALGEBRA.** Not this stone. O-iv-c-1 and O-iv-c-2 do that.
4. **A call site outside `src/intrinsic/holon/` or `src/holon/` needs changing.** Measured: the
   family is holon-internal. If the compiler names another caller, the blast radius is wrong —
   STOP and report where.

## Acceptance — run each, report the actual output

```
 0. ★ NOTHING MOVED. A scratch .wat under wat-scripts/scratch-pad/ (`--check` clean) that triggers
    one error from EACH of the nine require_* fns — a wrong-typed argument for each — before and
    after, diffed. Build the "before" with `git show HEAD:<path>`, never `git stash`.
    ⚠ Include the rendered `got` value in each, not just the message: STOP-2's whole point is that
    `ValueSnapshot::of` now sees a `&Value`.

 1. ★ EVERY CALL SITE FOUND BY THE COMPILER, NOT BY GREP. Report the count the compiler forced you
    to touch, per file, and confirm it matches 14/17/19/19/40. A discrepancy is a finding —
    my count is a grep of `require_` occurrences and may include non-call-site mentions.

 2. ★ NO DEEP CLONE WAS INTRODUCED. For each of the nine, say what the reference version does with
    what it extracts: `Arc::clone` (a refcount bump), a primitive copy, or something worse. If any
    is worse, that is STOP-1.

 3. cargo build --release --all-targets — clean. Report any warning verbatim.

 4. cargo nextest run --release -E 'test(holon) + test(intrinsic)' — Summary verbatim.
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

Row-by-row: the command, its actual output, PASS/FAIL. The per-fn answer from row 2. Then the honest
deltas. Every rider on this chain has caught a real defect in an orchestrator brief; the last one
found that a row-4 expectation in it did not survive contact with the disk and said so instead of
forcing it.
