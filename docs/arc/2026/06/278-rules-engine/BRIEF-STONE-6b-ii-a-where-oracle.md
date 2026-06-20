# BRIEF — Stone 6b-ii-a: `where`/TestNode in the ORACLE + the compile fence

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `./target/release/wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

Teach the **wat oracle** (`wat/rete.wat`) the `(:wat::rete::where <expr>)` condition. A `where` is a
left-only **filter node** (TestNode): it keeps a token iff `(:wat::rete::eval-test <expr>
<token-bindings>)` is true (the 6b-i primitive, already built). Add: the `TestNode` record + `Node`
defenum variant; a branch at the TOP of `compile-condition` that, on a `where` cond, **fences** it
(`pure? ∧ deterministic?`, raise on fail — 6a primitives, already built) then mints a TestNode wired
parent→test; and a **test-pass** in `fire-once` that runs AFTER `hash-join-pass` and BEFORE
`production-pass`, filtering `beta-memory[parent]` into `beta-memory[test-id]`. This is the ORACLE only —
no Rust, no native kernel (that's 6b-ii-b). Contract: `DESIGN-STONE-6b-where-test.md` (the 6b-ii-a entry).

## Read in order (the rooms — all in `wat/rete.wat`)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-6b-where-test.md` — the 6b-ii-a contract.
2. The node records + `Node` defenum (~`:69-112`) — add `TestNode` beside `ProductionNode`:
   `(:wat::Record::def :wat::rete::TestNode [id <- :wat::core::i64  expr <- :wat::WatAST  children <- :wat::core::PersistentVector<wat::core::i64>])`
   and a `:TestNode [node <- :wat::rete::TestNode]` variant in the `Node` defenum.
3. `compile-condition` (`:462-500`) + `find-or-mint-root-join` (`:392-417`) + `network-add-child`
   (`:351-355`) + `CompileState`/`MintResult`/`CondFoldAcc` — the mint+wire+thread pattern. Add a branch at
   the TOP of `compile-condition`: **if `cond` is a `(:wat::rete::where <expr>)` form** → (a) extract
   `<expr>` (the 2nd child — ground the WatAST child accessor, e.g. `ast->children` + `nth`; the make-rule
   path uses it); (b) **fence**: `(:wat::core::and (:wat::rete::pure? expr) (:wat::rete::deterministic? expr))`
   — if false, raise a compile error (use the same error idiom the file already uses, e.g. the
   `MalformedForm`/raise the other compile fns use); (c) mint a `TestNode` at `next-id` into the network,
   wire `parent-id → test-id` (if parent ≥ 0), return `CondFoldAcc state' test-id`. The ELSE branch is the
   existing alpha+join logic, unchanged.
4. `root-join-pass` (`:698-720`) + `hash-join-pass` (`:824-860`) — the **pass-fold model**: a pass is a
   fold step over one node-id that reads/writes `beta-memory`. Model the **test-pass** on these: for a
   `TestNode` id, read `beta-memory[parent-of(test)]` (the parent reverse-lookup `node-parent`, used by
   `fire-production` — reuse it), filter each Token by `(:wat::rete::eval-test (TestNode/expr node)
   (Token/bindings token))`, write the kept tokens to `beta-memory[test-id]`.
5. `fire-once` (`:977-1025`) — insert the test-pass fold between the hash-join-pass fold and the
   production-pass fold (a new `foldl` over node-ids, same shape as the others, threading the bmem).
6. How a Token's bindings are read (`Token/bindings`) + `eval-test` (`src/rete/matcher.rs`, 6b-i) — note
   `eval-test` takes its expr arg as a value that EVALUATES to a WatAST; inside the engine the expr flows
   as a WatAST *value* (the `TestNode/expr` accessor), so `(:wat::rete::eval-test expr bindings)` with
   `expr` a bound WatAST var works directly — NO quote needed (quote is only in the probe).
7. `tests/probe_arc278_6b_ii_a_where_oracle.rs` — the 5 assertions to green (do NOT edit it).

## Ordering constraint (v1 scope — name it, don't fight it)

The fire passes are type-segregated folds (all root-joins, then all hash-joins, then tests, then
productions). So a `where` is applied **after the joins** — it must come after the conditions that bind its
`?vars` (the natural order; you can't filter on `?c` before `?c` exists). Chained `where`s (test→test) work
because tests are id-ordered within the single test-pass fold. A `where` placed BETWEEN two type conditions
(followed by another join) is OUT OF v1 SCOPE — banked `6b-perf`. The probe's rules all put `where` last.

## Blast radius (bounded)

- `wat/rete.wat` ONLY. NO Rust (`eval-test`/`pure?`/`deterministic?` already exist). NO `kernel.rs` (6b-ii-b).
- ⛔ Do NOT touch the `render-dag` compound-concat FIXTURE (it is a deliberate proof-by-diff; leave it).

## STOP triggers (halt + surface; do not improvise)

1. If the WatAST child accessor (extract `<expr>` from `(where <expr>)`, read a keyword head) cannot be
   grounded in the file's existing helpers — STOP, report what IS available.
2. If `(:wat::rete::eval-test expr bindings)` / `(:wat::rete::pure? expr)` cannot be called with a bound
   WatAST *value* (vs a quoted form) — STOP, report the actual contract.
3. If the test-pass cannot slot into `fire-once` between hash-join and production without restructuring the
   other passes — STOP, describe the obstacle.
4. If greening needs editing Rust (`matcher.rs`/`kernel.rs`/`purity.rs`/`runtime.rs`) — STOP (out of scope).

## Done = green

`cargo test --release -p wat --test probe_arc278_6b_ii_a_where_oracle` → 5/5. Then the floors (EXPECTATIONS).
