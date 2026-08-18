# CURRENT STATE — annihilate interpretation in wat-rete

> **Locked 2026-08-17 so a compaction cannot drop it.** This is the live
> breadcrumb. Read this whole file before touching `src/rete/` or
> `wat/rete.wat`. If a stone below disagrees with a dated ruling here,
> **this file wins** and the stone is stale.

**Right now:** leftover rematch is on exists/not, HashJoin, and
accumulate `:from`. Clara mouths 1–7 locked. Compiler unification
is **unparked**: flip `compiled_cond`, then `compiled_rhs`. That is
the three-step list (`DESIGN-STONE-compiled-where.md`). User acc
folds are a **fourth surface** of the same `Expr` core, not a
required flip. `(b)` ShadowNode after cond/rhs sit on `Expr`.

**Tree — do not invent a cleaner one:**

| What | Where | Status |
|---|---|---|
| Compiled `where` | `30725034` | local, not pushed |
| Oracle bag + leftover rematch + `where-join-left` + `where-accum-from-left` | this commit | local, not pushed |
| Keyed `?g` bucket | `DESIGN-STONE-keyed-gather.md` | **not started** (speed, not alg) |

## The endeavor, in one sentence

**Annihilate all interpretation in wat-rete.** Every rete expression
becomes a compiled circuit. Fire supplies only concrete typed `Value`s.

**That is the endeavor, not the live wall.** Compiled `where` landed
(`30725034`). Node-share fire is 5 ms. The live wall is exists/not
over the **wrong bag**, then over an **unkeyed** alpha. Do not
reopen the compiler because this sentence names it.

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
the oracle (slow, trivially reviewable). Leading fact-shaped `:exists`
seeds from alpha. Combinator / `:where` inners rematch via leaf
alphas / `exists-cond-under`. `spec_equals` green.

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

## Earlier this session — already on `origin/main`

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

## In `lower()` — landed in `30725034` (local)

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

**Compiler unification is UNPARKED.** The three steps are: one
core (drawn), wire `where` (done), flip `cond` then `rhs`. User
acc folds are not a step on that list. Keyed gather is speed.

`(b)` — index the compiled predicates (discrimination tree; lab
`ShadowNode`, *"only go down paths that are actually possible"*) —
**after** the oracle true-up and after the remaining mouths sit on
`Expr`. Alpha already has this tree (`alpha_tree.rs`). `(b)` is the
same idea on `where` circuits.

## NOW — true up the oracle (exists/not is a data problem)

This is **not** a creative compiler strike. `DESIGN-STONE-7-exists.md`
already specified the gather: `token-element-compatible?` over the
inner **alpha**. Accumulate `:from` and HashJoin already do that.
What shipped instead was a **session-fact scan** on both mouths.

**Why the scan existed (do not re-derive):** leftover `?v < ?m` after
accum. Empty-seed alpha never sees the left-bound var, so someone
copied `any-fact-matches-under` / `wm_fact_slice` over the whole
fact bag and that workaround became the universal algorithm. The
leftover is often an **inline constraint** on the fact pattern
(`where-not-bound`), not a `:where` sibling. Structural populate +
seeded rematch is the rete answer. Do not put the WM scan back.

### Cut 1 — LANDED (this commit)

**Compatible-only over alpha was not enough.** `where-not-bound`
(`?v < ?m` after accum, Clara `test-accum-result-in-negation`) is
fact-shaped. Empty-seed alpha-match / `compiled_cond` compile that
constraint as a permanent miss (`Op::Fail`). Compatible-only then
sees an empty bag and `:not` always passes. The leftover is an
**inline constraint**, not always a `:where` sibling.

What the dirty tree does now:

| Mouth | Bag | Check |
|---|---|---|
| Fact-shaped `:exists` / `:not` | that node's **alpha** | `alpha-match-under` with the token seed (not `token-element-compatible?` alone) |
| Populate of an alpha whose cond has a deferred `?var` | same alpha | `alpha-match-local` / `compile_condition_local` — skip the unbound constraint so the facts enter |
| Combinator `:and` / `:or` / nested `:not` inner | **leaf** alphas (`mint-leaf-alphas` at compile) | `binding-extensions` rematches each leaf; no session-bag scan when the leaf alpha exists |
| `:where` inner | no bag | `eval-test` / `exec_test` |
| No alpha minted for a leaf | session facts (legacy fallback) | `alpha-match-under` |

Helpers: oracle `token-exists-under` / `any-seeded-element?` /
`mint-leaf-alphas` / `alpha-els-for-cond`. Native twins
`token_exists_under` / `any_seeded_in_alpha` / `alpha_els_for_cond`.
New rust primitives: `:wat::rete::alpha-match-local`,
`:wat::rete::cond-has-deferred-constraint?`.

`spec_equals_native_on_every_where_family` green (includes
`where-not-bound`, `where-not-and`, `where-not-and-bound`).
7exists / 7a / 7b / 8b green.

**Honesty holes:**

- Oracle `exists-uses-alpha-probe?` is five `ast-name` string
  equals. Native uses `classify_rete_clause`. Same five heads.
- `DESIGN-STONE-7-exists.md` said leading `:exists` raises.
  Clara made it legal. Do not restore the raise.
- A leftover on accumulate `:from` is CLOSED.
  Family `where-accum-from-left`. Oracle gather rematch
  (`alpha-match-under` over from-els). Native gather rematch
  (`fact_bindings_under` on the keyed bucket). 7/7 == Clara.
  spec == native. Empty `:from` still count 0.
- A **join** cond with a leftover `?w > ?c` is CLOSED.
  Family `where-join-left`. Oracle rematch first (`cross-join-node`
  via `alpha-match-under`), then native (`join_extend` on P6 +
  `keyed_join`). `check-where-shapes.sh where-join-left` 9/9 ==
  Clara. `check-spec-native.sh` 9/9. Do not drop the rematch.
- Clippy `--all-targets -D warnings` re-run after the matcher
  collapsible-match fix.

### Cut 2 — NOT STARTED (this is the next strike)

Linear scan of `|alpha|` is still the native wall
(`accum_fire_phase_census` 200×200: fire **215 ms**, filter
**47%**). `DESIGN-STONE-keyed-gather.md` is the algorithm:
once per round, `HashMap` over the node's alpha by the shared
`?g` tuple, then each token probes its bucket. Same bag as
hash-join.

That stone (2026-07-31) said **no `.wat` changes** — that
applies to the **key**, not the bag. Cut 1 already moved both
mouths onto alpha. Cut 2 keys **native** over that alpha;
the oracle stays a linear fold over the same elements
(`OCVLI NOVI, ORACVLVM IMMOTVM`). Order, empty-bucket, and
empty `join_keys` → cartesian are load-bearing in that stone.

Accumulate `:from` is the same index. After cut 1 it is **not**
the 47% slice (`accum:fold` is 9 ms). Do not “fix” the
accumulate gather first thinking that is the wall.

### Measured 2026-08-17 (do not rediscover)

Two clocks. Do not mix them.

**Clock A — native fire only**
(`accum_fire_phase_census` / `node_share_fire_phase_census`):

| What | Before cut 1 (WM scan) | After cut 1 (linear alpha) |
|---|---|---|
| accum 100×200 | 451 ms, filter 88% | **70 ms**, filter 22% |
| accum 200×200 | 1.83 s, filter 94% (1.73 s) | **215 ms**, filter 47% |
| accum:fold 200×200 | 9 ms (1%) | 9 ms — built-in folds were never the wall |
| node-share 50×200 | — | 5.0 ms, filter 76% (compiled `where`; done) |

**Clock B — compiled `where` vs oracle walk** (same 10k evals):
241 ns vs 936 ns (**3.9×**). Floor still 9.5 ns. Unrelated to
exists/not.

**Clock C — `run-axis.sh` with `fire-rules-spec` still in the
wat process** (measured **before** cut 1; oracle wall **not
re-timed** after the alpha probe):

- accum `[50 200]` wall 67 s vs Clara 4.6 s (`:wall-winner :clara`).
  Native fire 106 ms. The 67 s **is** `fire-rules-spec`.
- min-finding `[2000 3]`: native 11 ms, oracle **288 s**. The
  20-minute cells are this mouth. JVM tax is moot.

**Clock D — Clara vs native-only** (spec fire stripped from a
**temp copy** of the axis `.wat`; **before** cut 1):

- accum `[50 200]` `:ratio 0.45 :clara` (104 ms vs 47 ms)
- `[100 200]` **0.20 :clara** (415 ms vs 85 ms)
- At `[200 200]` native fire was ~1.8 s — that **was** the WM
  scan. After cut 1 it is 215 ms. **Clara vs native-only has
  not been re-run on the dirty tree.**

`run-axis.sh` is the timer. Do not wrap grid cells in Python.
`GRID_RUNS=1` is a look; near-parity needs 3. Do not kill a
17-minute cell — that *is* the cell.

## What shipped — local, not pushed

`30725034` (compiled `where`) plus **this commit** (oracle bag +
leftover rematch). Do not push until asked. `origin/main` never
`origin/grok`.

- **`where` compiled** (`30725034`).
- **Exists/not / join / `:from` rematch** the left token. Families
  `where-join-left`, `where-accum-from-left`. Clara 1–7 locked.
- Query maps / fact-bind / clippy `--all-targets`: on `origin/main`.

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
- Push `origin/main`, never `origin/grok`. Do not push until asked.
- Do not police termination as a fifth *axis*. The load refusal is
  the wall.
- Compiler flips are `cond` then `rhs`. Do not invent a third flip
  named “user folds.” Do not start `(b)` until cond/rhs are on
  `Expr`. Keyed gather is not a compiler dep.
- Do not revert cut 1 back to `wm_fact_slice` for fact-shaped
  inners. Do not fold leftover `?v < ?m` into the alpha probe.
  Do not refuse leading `:exists`.
- Do not wrap `run-axis.sh` in Python. Do not treat a long grid
  fire as hung. Do not put `fire-rules-spec` back into a “native
  vs Clara” wall clock.
- Do not cite Clock C / Clock D numbers as post-probe. They are
  pre-cut-1. Do not cite 1.83 s as the current native fire.

## Read order

1. **This file** — especially **NOW**.
2. Dirty code: `wat/rete.wat` `token-exists-under` (just after
   `token-element-compatible?`) and `src/rete/kernel.rs`
   `token_exists_under`. Filter-pass Negation/Exists arms.
   Leading-exists seed.
3. `DESIGN-STONE-keyed-gather.md` — **cut 2**. Line numbers in
   that stone are stale; the algorithm and the three contract
   clauses are not.
4. `DESIGN-STONE-7-exists.md` — original gather (we returned to
   it). Leading-exists raise is stale; Clara made it legal.
5. `DESIGN-STONE-the-one-expression-core.md` — parked until
   oracle is sane. `where` is already wired.
6. `src/rete/expr_ir.rs` — compiled `where` (landed in `30725034`).
7. `wat-scripts/perf/grid/run-axis.sh` — the timer.
8. `wat-scripts/perf/grid/REMAINING-CLARA-MOUTHS.md` — expressivity
   is closed; do not reopen it as “the next mouth.”
