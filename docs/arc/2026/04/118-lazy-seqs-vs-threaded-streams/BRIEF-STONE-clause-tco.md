# BRIEF — clause heads get TCO

You are a rider, not the orchestrator. **Ending your turn ENDS you** — it does not suspend you,
nothing wakes you, no notification is coming. Run every verification in the **FOREGROUND** and block
on it: your turn ends when the numbers are in your hands, not when a command is launched. A floor run
takes ~4 minutes; wait for it.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

⚠ **Every program you run goes under a memory cap:**
`systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 timeout 180 ./target/release/wat <file>`
`MemorySwapMax=0` is the load-bearing half — without it a runaway swaps instead of dying and takes
the machine with it. Read exit codes directly, never through a pipe (`cmd | tail` reports `tail`'s
status, not the command's).

## Read first

1. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-clause-tco.md` — **read the
   CORRECTION section**; the design was re-planned once and the first plan is preserved as wrong.
2. `…/EXPECTATIONS-STONE-clause-tco.md` — the scorecard, fixed before the strike.

## The work in one paragraph

`eval_tail` emits a tail call only when the call head is in `sym.functions`. A `defclause` head
resolves to a `ClauseSet`, so it falls to `_ => eval_inner(...)` and recurses on the real stack —
every `defclause` in wat is non-tail-recursive today. Give `Clause` a pre-built `Arc<Function>` at
registration, extract clause *selection* from the existing dispatcher, and add one `eval_tail` arm
that selects and emits a tail call — **except** for clauses carrying `:ensure`, which must keep the
ordinary path because a post-condition needs a frame to return into.

## Rooms — every site verified on disk

| what | where |
|---|---|
| `pub struct Clause` — gains `pub func: Option<Arc<Function>>` | `src/value/value.rs:393` |
| ★ **a Function built FROM a Clause, verbatim exemplar to copy** | `src/runtime.rs:1205` |
| `Function`'s fields | `src/value/environment.rs:46` — `name, params, type_params, param_types, ret_type, rest_param, rest_param_type, body, closed_env, rete, synthesized_for` |
| `ArgSpec` | `fixed_params: Vec<(Identifier, TypeExpr)>`, `rest_param: Option<(Identifier, TypeExpr)>` |
| `Clause` construction (parse) — add `func: None` | `src/runtime.rs:7831` |
| `Clause` construction (extend-type impls) — `func: None` is correct there | `src/runtime.rs:8240` |
| `Clause` construction in a `#[test]` fixture — add `func: None` | `src/check/env.rs:486` |
| ★ **ClauseSet assembly — the set NAME is in scope here; fill each clause's `func`** | `src/runtime.rs` ~7978–7990 |
| ★★ **the dispatcher to extract selection from** | `src/runtime.rs:8364` `eval_call_to_defclause_with_vals` |
| the seam inside it: everything before this line is selection+binding | its `eval_inner(&clause.body, &scope, sym)` call |
| ★★ **`eval_tail`'s keyword-head match — add the arm before the `_`** | `src/runtime.rs` ~4334 |
| how a clause-valued head is looked up (`sym.def_value(name)` → `Value::wat__core__clauses(cs)`) | `src/runtime.rs:7286`–7302 |
| the signal to construct | `src/runtime.rs:4393` `EvalSignal::TailCall { func, args, call_span }` |
| tail-aware siblings to copy for shape | `eval_if_tail` `:4402` · `eval_match_tail` `:4560` |

`FunctionBody::Wat(Arc<WatAST>)` and `Clause.body` is already `Arc<WatAST>` — direct.

## The shape

```
1. Clause gains `func: Option<Arc<Function>>`.
2. At ClauseSet assembly (name in scope), fill each clause's func — copy runtime.rs:1205 verbatim;
   it already builds a Function from a Clause. `closed_env: None` (clauses are top-level).
3. Extract from eval_call_to_defclause_with_vals:
       select_clause(cs, &vals, sym) -> Result<(usize, Environment), EvalBreak>
   covering arity + type + :guard + binding. The existing fn then calls it and does body + :ensure.
   EXTRACT — do not copy the loop.
4. eval_tail gains an arm: head resolves via sym.def_value to Value::wat__core__clauses
       → evaluate args
       → select_clause
       → clause.ensure_fn.is_none() AND clause.func.is_some()
             → Err(EvalBreak::Signal(EvalSignal::TailCall { func, args: vals, call_span }))
         otherwise
             → today's ordinary call, unchanged
```

The args are already evaluated on that path — **build the signal directly; do not route back through
`emit_tail_call`, which re-evaluates raw args.**

## The gate

| # | assertion |
|---|---|
| 0 | run `scripts/floor.sh` FIRST and record the baseline: **4714 passed, 0 failed** |
| 1 | ★ `probe-clause-tco-deep-defclause.wat` was `rc=139` SIGSEGV → now **prints 200000** |
| 2 | `probe-clause-tco-deep-defn.wat` still prints **200000** |
| 3 | ★★★ `probe-clause-tco-ensure-still-fires.wat` still raises **`PostconditionFailed`**, rc≠0 |
| 4 | `probe-clause-tco-guard-selects.wat` still prints **120** |
| 5 | floor GREEN — read the **Summary line**, never a piped exit code |
| 6 | `cargo clippy --release --all-targets` → **0** |
| 7 | `#[ignore]` count → **13** |
| 8 | the new `Clause` field is an `Option<Arc<Function>>` — report the diff line |

All four probes are in `wat-scripts/scratch-pad/`, already committed, with their baselines in
their headers.

## STOP triggers — ship nothing on that axis; report and stop

- **STOP-1 — `:ensure` becomes reachable via the tail path.** A tail call abandons the frame, so a
  post-condition cannot run. If you cannot cleanly exclude ensure-bearing clauses, STOP. A green
  row 1 with a broken row 3 is a worse substrate than we have now.
- **STOP-2 — selection cannot be extracted without duplicating it.** Two copies of clause dispatch
  is the defect this project keeps deleting. If the seam will not cut cleanly, STOP and report where
  it binds.
- **STOP-3 — args would evaluate twice** on the tail path. Report it rather than accepting it; an
  effectful argument firing twice is a correctness bug, not a perf note.
- **STOP-4 — the floor goes red for any reason other than a golden's line-number shift.** Do NOT
  re-run first: `scripts/floor.sh` keeps the untruncated log at `.floor/latest/`. Copy the failing
  test's **entire** stdout and stderr **verbatim** and name the exact assertion or match arm that
  fired. **There is no such thing as a known flake.**
- **STOP-5 — the `#[ignore]` count moves off 13.**

⚠ **Goldens:** an `.edn` golden under `tests/diagnostics/` failing because a **line number shifted**
is yours to update — that IS the work. Say which moved and by how much. Anything else red is STOP-4.

## Out of scope — affirmative cuts

- **`reduce-walk` in `wat/seq.wat`** — a workaround for this very defect. Whether it stays once
  clauses TCO is a separate ruling; leave it alone.
- **The six remaining three-call Stream walks** (`remove`, `take-while`, `drop-while`, `take-nth`,
  `reductions`×2) — B2's completion, unrelated to tail position.
- **Task #58** — a clause recursing inside a `cons` or an argument still consumes stack and still
  dies silently. This stone does not touch that and must not claim to.

## Report

The scorecard row by row with real results, the diff line for row 8, the honest deltas (anything
that surprised you or that you changed beyond the plan), and line counts. If a STOP fired: the
verbatim evidence and which trigger — shipping nothing on that axis is the correct outcome.
