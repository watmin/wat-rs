# CURRENT STATE — annihilate interpretation in wat-rete

> **Locked 2026-08-17 so a compaction cannot drop it.** This is the live
> breadcrumb for the rete *compiler* endeavor. Read this whole file before
> touching `src/rete/`. History of the *draw* lives in the stones this
> file cites; if a stone below disagrees with a dated ruling here, **this
> file wins** and the stone is stale.

## The endeavor, in one sentence

**Annihilate all interpretation in wat-rete.** Every rete expression
becomes a compiled circuit. Fire supplies only concrete typed `Value`s.
Compilation of exprs is the last perf laggard after weeks of attacking
the rest.

Clara **pure** mouths are locked. What Clara has and we cut
(`insert!` / `retract!` / salience / untyped maps) stays cut — that
impurity is what the fence exists to refuse.

## Why now (the deps are satisfied)

| Dep | Status |
|---|---|
| Closed vocabulary (law A, `RETE_OPS`) | Armed at `:where`, `:then`, user accum folds |
| Pure ∧ deterministic ∧ total | Armed. Every `RETE_OPS` row is `total: true` (build-red otherwise). Partial core ops enter only as `Fallback` + `:undefined` |
| `:wat::rete::core::defn` membrane | Body proved once at freeze (`#88`) |
| Named recursion | **Refused at load** (`#87`, 2026-08-17). `#wat.runtime/ReteDefnRecursive` |
| Expressivity (Clara-pure) | All five remaining mouths DONE (`REMAINING-CLARA-MOUTHS.md`) |
| Step 0 measurement | Walk is 77% of a `where` eval; 540 ns/eval vs 21 ns floor; dispatch 75% of the walk |

`pure?` still admits a cycle (a cycle is not impure). The wall is the
declaration, not a fifth axis. Totality means *never raises*, not
*terminates*. eBPF-shaped: static refusal at load, never a runtime budget.

## The destination machine

The compiled program is a **closed circuit**. `Expr` nodes. `OpIdx`
resolved once. `?var` is a slot index. Fire does not walk `WatAST`,
does not hash a name, does not build an `Environment`.

Dispatch is: **this typed concrete value, this opcode.**

`defn` and `fn` are the same kind of thing — a compiled `Program`
waiting for slots:

| Form | Who fills the slots | When |
|---|---|---|
| `:wat::rete::core::defn` | caller arguments | at the call |
| literal `fn` with no frees | `foldl`'s `(acc, x)` | each iteration |
| `fn` that mentions an outer `?var` | that binding, then `(acc, x)` | capture at creation, params at call |

Capture is not interpretation. It is writing known slots a moment
earlier — Minamide's `(code, env)`, both residual data. The live
corpus `foldl`s have **no frees**; those lambdas *are* anonymous defns.

This is Futamura's first projection (never named on disk until now;
`DESIGN-STONE-compiled-conditions.md` already said *"proving partial
evaluation on a small understood surface"*). `compiled_cond` and
`compiled_rhs` are that, half-done. `RhsOp::Expr(WatAST)` is the
residual they never finished specializing. Arc 170's
`ClosurePackage` (`prologue` + `entry_form`) is the same pair, built
for process-spawn, not rete.

## The build — one core, three adjacent flips

Drawn: `DESIGN-STONE-the-one-expression-core.md`.
Wired for **`where` only** (2026-08-17). `src/rete/expr_ir.rs` exists.
`compile-condition` refuses via `(:wat::rete::lower expr)`. Native
`fire-rules` stashes `HashMap<id, Program>` once at `fire_fixpoint_delta`
setup (same table shape as `compiled_conds`) and the TestNode filter
calls `exec_where` only — no re-`lower`, no `eval_inner`. `:expr` stays
on the wat record for compile / spec / census. `eval_test_core` remains
the oracle (slow, trivially reviewable). Leading `:exists` `where` still
re-lowers via `binding_extensions`. `spec_equals` is green.

1. **One `Expr` DAG** over the closed rete vocabulary. Nested children
   (builder, 2026-08-06: *"matches the precedent"*). Not bytecode
   offsets. The enum discriminant *is* the jump table.
2. **Wire only `where`.** **Done.** Differential against `eval_test_core`
   — same `bool`, same `Err`. `eval_test_core` is not deleted.
3. **Flip `cond`, then `rhs`, one at a time.** Each already has a
   green interpreter differential. Not started.

There is **no `Interp` arm.** `BRIEF-compiled-where.md` still describes
`Op::Interp` and a third sibling `compiled_where.rs`. **That brief is
stale.** The builder cut the hatch on sight. A falling-back compiler
makes the perf claim unfalsifiable and is the mask class.

Four surfaces, one core; they differ only in prologue / epilogue:

| Surface | Prologue | Epilogue |
|---|---|---|
| `where` | token bindings → slots | must be `bool` |
| `compiled_cond` | fact fields → slots | bool + the slots ARE the binds |
| `compiled_rhs` | token bindings → slots | `Value` becomes a field |
| accum fold | gathered values → slots | the reduced `Value` |

68 of 75 `RETE_OPS` rows are strict (`Call`). Twenty are
`CallFallback`. Seven are lazy: `and` · `or` · `if` · `let` · `match`
· `cond` · `fn`. `not` is a strict boolean.

`compiled_cond::Op::Or` / `Op::Not` are **clause** combinators (they
bind). Expression `or`/`not` combine values and bind nothing. Same
spelling, different ops.

## What shipped this session (2026-08-17) — on `origin/main`

- **278 query:** answers are binding maps; fact-bind
  `(?p <- :ns::Type …)` is how you get the record. One public
  `query` mouth. `query-ask` annihilated.
  Commit `d2d73dc3`. Clippy `--all-targets` fix `b46b5f1f`
  (CI is `clippy --release --workspace --all-targets -- -D warnings`;
  local `--workspace` without `--all-targets` is a narrower surface).
- **Mouths 1–5** locked with Clara twins. `check-query-compat.sh`:
  3 families, 24 rows, Clara == oracle == native.
- **#87 rete-defn may not recurse.** Gray-node DFS over named Wat
  callees at `apply_rete_defn_contracts`. Self and mutual refused;
  acyclic DAG (`wrap` → `leaf`, `where-nesting` c1…c10) still loads.
  Probes: `tests/rete/probe_arc278_rete_defn_recurse*`.
- Rebased onto Claude's 19 commits (`2072bce4`). Rete-cohort nextest
  override (60s/120s, `priority = 98`) already covered
  `spec_equals_native`. Do not add a named override above it.

Floor after rebase: `.floor/2026-08-17T10-25-55Z/` —
`4703 passed, 19 skipped`. `spec_equals` 38.338s.

## In `lower()` this strike (2026-08-17, uncommitted)

- **`src/rete/expr_ir.rs`:** `Expr` / `Pat` / `Program` / `lower` /
  `exec_where` / `exec_test` / `eval_lower`. No `Interp` arm.
- **HOF callee** is the first arg of `foldl`/`foldr`/`map`/`filter`/
  `reduce` only. Literal `fn` or a named rete-defn keyword. The flag
  is consumed at that node — a binder inside the `fn` body (`acc` in
  `(and acc …)`) is not a callee.
- **`Program.params`** are the declaration-order slots. A literal `fn`
  compiled inside a `where` shares the parent slot numbering; foldl
  writes `[acc, x]` there and copies the parent frame for captures.
- **`CallFallback`** faces the same four holes `dispatch_rete_op` does:
  i64 raise, non-finite f64, `Option::None` (`*/get`), `MalformedForm`
  whose `head` is `core_name` (`first`, `string::subs`).
- **`match`** unit enum tags are `Pat::Variant` (composed
  `type_path::variant_name`), not keyword literals. `Some`/`None`/
  `Ok`/`Err` stay the dedicated value shapes.
- **`(:Type/field recv)`** is `Expr::Field { idx }` from `TypeEnv`.
- **Inlining `CallUser` is CUT.** A call *is* the circuit.
- Grid: `grid_axes_run_and_derive_nonvacuously` green (was 5/39 dead).
  `spec_equals_native_on_every_where_family` green.

## Ruled, still true

- **STOP-2 — the frame.** Copied captures. A lambda is a `Program`.
  A parent pointer into a live interpreter frame is off the table.
- **eBPF, not a fifth axis.** Recursion / bounds are load-time
  refusals. `pure?` does not lie about cycles.
- **HOF fn-arg vs capture are different questions.**
  *Which body?* vs *where do its frees live?* The corpus `foldl`s
  (`where-collection`, `user-reduce`) are all literal `fn` with no
  frees — they do not force capture. The parent-frame copy is there
  so a future free is a slot write, not a new mechanism.

## Still open — they block different things

| Open / settled | Status |
|---|---|
| HOF fn-arg | **Settled (4Q).** Callee visible in the AST. Unknown `Function` at `foldl` does not load. |
| Fn in a fact field | **Settled.** Facts are records; records are pure data. A function is not a fact field. Same class as HOF-lexical: it cannot arrive from WM. |
| Depth / nodes / derived-fact explosion | **Later.** Near-term DoS is closed by no recursion. Cardinality explosion (MySQL/Athena-shaped client guard) is a different stone. `Program` may *record* measured depth/nodes; do not enforce a number we have not derived. |
| `(:Type/field ?var)` | **Settled — compile the index.** The class and field are **in the accessor head** (`:wfb::Temp/c` → type `Temp`, field `c`). `TypeEnv` gives the `usize` at rule-compile. The 2026-08-06 “we don’t know `?route`’s class” claim assumed a TestNode compiled from the expr *alone*. At rule-compile we have the form *and* `collect_rule_bind_types`. Carry-the-name is the worse residual, not the required one. |
| `match` map-destructure field index | Only that arm. Possible; not specified. Not a v1 blocker. |

`(foldl ?f 0 xs)` is a `LowerError` (HOF settled). No numeric
ceiling until one is derived. Cardinality DoS is a later stone.

**Next flip:** `compiled_cond`, then `compiled_rhs`. Same `Expr`
core. Do not start `(b)` until cond/rhs are on the circuit or the
builder names `(b)` next.

`(b)` — index the compiled predicates (discrimination tree; lab
`ShadowNode`) — **tracked, after `(a)`**. Ruling 2026-08-01 in
`DESIGN-STONE-compiled-where.md` (*compile, THEN index*). Alpha
already has its tree (`DESIGN-STONE-alpha-discrimination-tree.md`,
`src/rete/alpha_tree.rs`). `(b)` is the same idea on `where`
circuits. Semantic (a token that never reaches a rule never runs
that `where`). Wants `(a)`'s `Expr` so dimensions are analyzed
once. Builder: wanted, in time — not this strike. No standalone
`DESIGN-STONE` for the where-tree yet; the draw lives in
compiled-where until `(a)` ships.

## What a new self must not do

- Do not write `compiled_where.rs` as a third sibling compiler.
  Write `src/rete/expr_ir.rs`.
- Do not add `Op::Interp`.
- Do not treat `BRIEF-compiled-where.md` as the brief. It predates
  the hatch refusal.
- Do not globally raise nextest timeouts. Named overrides only.
  The rete cohort already owns `spec_equals`.
- Do not run Java inside Rust tests. `check-query-compat.sh` is a
  shell script; JDK lives at `$HOME/opt/jdk-*`.
- Do not re-run a red floor. ARM first.
- Push `origin/main`, never `origin/grok`.
- Do not police termination as a fifth *axis*. The load refusal is
  the wall.

## Read order

1. **This file.**
2. `DESIGN-STONE-the-one-expression-core.md` — the `Expr` set.
3. `DESIGN-STONE-compiled-where.md` — Step 0 numbers; three flips.
4. `src/rete/compiled_cond.rs` then `compiled_rhs.rs` — the two
   half-compilers and the `RhsOp::Expr` hole.
5. `src/rete/purity.rs` `apply_rete_defn_contracts` — four axes +
   the cycle walk.
6. `tests/rete/probe_arc278_rete_defn_recurse.rs` — the recursion
   gate.
7. `wat-scripts/perf/grid/REMAINING-CLARA-MOUTHS.md` — expressivity
   is closed; do not reopen it as “the next mouth.”
