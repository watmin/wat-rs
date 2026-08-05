# BRIEF — migrate the callers the armed fence named. **The fire IS the worklist.**

> **State, measured 2026-08-05, none inherited.** HEAD `ab6720dc` is green and pushed. The fence is
> ARMED in the working tree (`wat/rete.wat`, uncommitted; parked copy at
> `docs/arc/2026/06/278-rules-engine/law-a-fence-flip.patch`). The release binary at
> `./target/release/wat` is **already built with the fence armed** — use it, do not rebuild.
>
> With it armed: floor **4321 passed / 40 failed**; `check-where-shapes.sh` 9/9 families refused.
> Refusals name two axes: **50 `is not total`**, **36 `is not a rete primitive`**.

## The chain every `where` expression must now satisfy

```
is-pure  ∧  is-deterministic  ∧  is-total  ∧  is-rete
```

Each measured strictly and separately — *"verbosity is our shield."* A refusal names the FIRST
conjunct that failed, so fixing one head can reveal the next on the same expression. **That is the
waterfall, and it is the method** (`docs/SUBSTRATE-AS-TEACHER.md`): the fail-count is the progress
meter, not a crisis.

## ▶ HOW TO WORK — read the screams, do not hunt

Do **not** grep for a worklist; a line-based grep over these files is demonstrably noisy (it catches
`quasiquote`/`unquote` that sit on a `where` line without being inside the form). Instead:

```bash
./wat-scripts/perf/grid/check-where-shapes.sh 2>&1 | grep -oE "is not [a-z ]*— '[^']*'"
```

Fix the head it names. Re-run. Repeat until the family passes. The fence is exact and located.

## ★ THE MAPPING — grounded off the 70 `RETE_OPS` rows, never inferred

Every target you emit must appear as a `rete_name:` in `src/rete/vocabulary.rs`. Verify, don't guess.

| kind | rule |
|---|---|
| **mechanical** | `:wat::core::X` → `:wat::rete::core::X` (one twin) |
| **holon** | `:wat::holon::X` → `:wat::rete::holon::X` (**not** `core::`) |
| **per-type** | `> < >= <= + = not=` have **NO generic twin and never will** → pick `i64`/`f64`/`string`/`bool`/`keyword`/`enum` from the operand's DECLARED type in the same rule's LHS |
| **multi-twin** | `first` → `{PersistentVector,Vector,List}/first`; `get` likewise |
| **fallback** | 17 ops take a MANDATORY 4th arg `:undefined <value>` — see below |

### ★★ THE FALLBACK OPS — a call-SHAPE change, not a rename

`i64::{+ - * / mod quot rem}` · `f64::{+ - * /}` · `string::subs` · `{PV,Vector,List}/get` ·
`holon::{cosine,dot}` take **four** arguments. A spelling-only rename compiles past the fence and
then dies at dispatch with `ArityMismatch: expected 4, got 2`.

> *"Clara will raise an exception when you divide by zero; wat-rete forces the user to make a
> conscious choice. Clara allows runtime exceptions. We must not."* — the builder

**THE RULE FOR CHOOSING THE VALUE — choose it so the ENCLOSING PREDICATE answers its own question
honestly, NO, when the operand is undefined.** A rule whose arithmetic is undefined does not fire.

**PROVEN EXEMPLAR, already on disk and green — copy its shape:** `where-numeric.wat`, commit
`f2ff67ce`.

```clojure
(i64::< (i64::quot ?a 7)   0)     ->  :undefined 0     ; 0 is not < 0
(i64::> (i64::/    ?a ?z)  1)     ->  :undefined 0     ; 0 is not > 1   ← ?z is a VARIABLE: reachable
(f64::> (f64::*    ?x ?x) 100.0)  ->  :undefined 0.0   ; 0.0 is not > 100.0
```

⛔ Do **not** skip the param because a divisor is a literal and the hole looks unreachable. An
unreached hole makes the choice CHEAP, never OPTIONAL.

### ★★ SHARPENED 2026-08-05 — "the enclosing predicate" means the WHOLE guarded expression

Found by a rider on `where-control.wat:233`, and the naive reading would have shipped a bug. The
`where` is:

```clojure
(let [s (i64::+ ?a ?b :undefined ?)]
  (and ?n (if (i64::> s 6) true (i64::< s 3))))
```

Pick `:undefined 0` — the obvious "smallest honest value" — and: `(> 0 6)` is false, so control
falls to the else-arm, `(< 0 3)` is **TRUE**, and **the rule FIRES on undefined input.** Exactly the
outcome the fallback exists to prevent, produced by applying the rule to the nearest comparison
instead of the whole expression.

`:undefined 3` makes both branches read false.

**So: the value must make the ENTIRE fenced expression answer NO — every branch of every `if`/`cond`
on the path, not just the comparison the op is nested in.** Where the expression is a bare
comparison those are the same thing; where it is a conditional they are not, and the difference is a
rule that fires when it must not.

### ⛔ THE ONE MIXED-TYPE SITE

`(:wat::core::< ?a 0.5)` — `?a` is i64, `0.5` is f64 → `(f64::< (i64::to-f64 ?a) 0.5)`.
**Never widen a rete op to accept mixed operands.** The explicit cast is the feature.

## ⛔ DO NOT TOUCH

- `:wat::core::record?` ×2 and `:wat::core::Uuid/v4` ×1 — negative controls; their refusal is pinned
  by `assert_eq!` in `tests/rete/probe_fence_names_the_head.rs`.
- `probe_arc278_6b_ii_a_where_oracle_impure.wat` · `probe_fence_names_the_head_nondet.wat` — they
  exist to prove the fence REJECTS.
- `:wat::core::unquote` / `quasiquote` — **NOT targets, proven by run**: they are template-escape
  syntax and are gone before the fence ever sees the form
  (`wat-scripts/scratch-pad/probe-unquote-is-gone-before-the-fence.wat`).
- Anything under `src/`. Anything outside a `where` form.

## ⚠ HELPER FUNCTION BODIES ARE IN SCOPE — this surprises people

A user `defn` called FROM a `where` is walked by the fence (the composition door recurses). So
`:wsb::edge?`, `:wst::feline?`, `:wnst::c2`/`c3` and their siblings must ALSO be migrated even
though their bodies are not lexically inside a `where`. The refusal will name the head in the
helper's body; follow it there.

## STOP triggers — rejection criteria, never permission to defer

- **STOP-1.** An operand whose type you cannot determine from the DECLARED field type in the same
  rule's LHS. Do not guess, do not widen. Report `file:line` and move on.
- **STOP-2.** A target name absent from `RETE_OPS`. The row must be minted first — orchestrator's
  call, not yours.
- **STOP-3.** Any edit to a DO-NOT-TOUCH file, or to `src/`.
- **STOP-4.** `:derived` output changing. A spelling migration must not move a derived fact. (A
  genuine TOTALITY divergence from Clara is expected and is not this — report it, do not hide it.)

## Rules of engagement

- Work only in `/home/watmin/work/holon/wat-rs/`. Any path containing `.claude/worktrees/` is harness
  state and illegal to operate on.
- **Do NOT commit, push, stash, or revert.** The orchestrator integrates and weighs centrally.
- **Do NOT run `cargo build` / `nextest` / `clippy`.** `./target/release/wat` is already built with
  the fence armed. A second build against the same `target/` lock stalls everyone.
- You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you and no
  notification is coming. Run every verification in the FOREGROUND; your turn ends when the numbers
  are in your hands.

**REPORT:** every `file:line` you changed with the head before→after; every STOP you hit with its
`file:line`; and the final `check-where-shapes.sh` line for each family you own, verbatim.
