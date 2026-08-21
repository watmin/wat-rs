# EXPECTATIONS — arc 109 γ-i: `fn` takes the `:- [T …]` binder

**Written BEFORE the strike**, so the result cannot move the goalposts. Every row is re-run by the
orchestrator independently; the rider's report is a hypothesis until then.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | `defn` takes the binder | `--check` a file with `(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)` | exit 0 |
| 2 | ★ anonymous binder fn is GENERIC | `--check` `(let [f (fn :- [T] [x <- :T] -> :T x) _ (f 1) __ (f "s")] nil)` | exit 0 |
| 2b | ★ negative control for row 2 | the same WITHOUT the binder: `(fn [x <- :T] …)` applied at two types | still **FAILS** — proves row 2 passed because of the binder, not because rigidity was silently relaxed |
| 3 | both spellings on one decl | `--check` `(:wat::core::defn :user::f<T> :- [T] …)` | a located error naming the contradiction; NOT a silent pick |
| 4 | 251.7 does not regress | probe rung 1 — a no-param-list `defn` applied at two types | exit 0 |
| 5 | the HOF control is undisturbed | probe rung 4 | exit 0 |
| 6 | parametric kwargs `defn` | binder-spelled kwargs `defn`; inspect the expansion via `macroexpand` | bundle is `Kwargs<T,U>`, **not** monomorphic |
| 7 | variadic `defn` | binder-spelled `defn` with `& xs <- :T` | registers and checks |
| 8 | `def` untouched | `def` of a non-fn value; and `git diff --stat src/check.rs` | registers; **check.rs has ZERO changes** |
| 9 | the floor | `scripts/floor.sh` (orchestrator, centrally) | **4855/4855**, 0 FAIL, 19 skipped |
| 10 | clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

Rows 1–8 are the rider's acceptance criteria and are cheap (`--check`, ~0.2s each). Rows 9–10 are
the orchestrator's, run centrally on a quiescent tree.

## Independent prediction

**Runtime: 35–60 min.** Not a peel — row 2 is a genuine capability. Three of the five files are
mechanical (the peel, its two call sites, the union in three `def`-fn recognizers, the macro splice);
`infer_fn`'s generalization is the whole risk and could take the majority of the time alone.

**2× time-box: 120 min.**

## Trap doors, named before the strike

1. ★ **`infer_fn` builds no scheme.** The rigidity is not an oversight to patch but a missing step to
   add. If the rider treats row 2 as a peel, it will report success on row 1 and quietly fail row 2 —
   which is why **2b exists**: without a negative control, a generalization that never happened and a
   rigidity that was silently relaxed produce the same green.
2. **One instantiation is not a test.** A rigid `:T` passes a single application. Row 2 must apply at
   two DIFFERENT types or it measures nothing. `[[feedback_a_green_test_can_prove_nothing]]`
3. **The `wat/core.wat` half cannot be verified by the rider** — `include_str!` bakes the stdlib at
   RUST-compile time. Expect its macro edit to be described rather than demonstrated, and weigh it by
   reading the diff and rebuilding centrally. **A rider claiming it verified the macro half is
   reporting something it could not have measured.**
4. **`:-` lexes as a KEYWORD, not a Symbol.** Measured this session. The proc-macro crate carried a
   dead `Symbol(":-")` arm for months on exactly this mistake. If the peel matches a Symbol it will
   silently never fire, and rows 1 and 2 will fail with the ORIGINAL error — which reads like "not
   implemented yet" rather than "matched the wrong node kind."
5. **Row 6 has zero corpus instances.** It is a rule about `defn`'s `{b}::Kwargs{p}` derivation from
   `name-tp`, which the binder spelling empties. A rider scoping from the corpus will not find it and
   will not think to check. `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`
6. **A cascade is expected and is NOT a crisis.** If the peel changes the shape `try_parse_fn_shape_def`
   sees, many stdlib defs may fail to register at once. That is the substrate naming its next site,
   not a reason to revert. `docs/SUBSTRATE-AS-TEACHER.md`.

## Scoring method

Written after the orchestrator's OWN re-run of every row — never from the rider's report. Row 2 is
re-run first and row 2b immediately after it; a green row 2 with a green 2b means the change did not
do what it claims, and the stone does not ship.
