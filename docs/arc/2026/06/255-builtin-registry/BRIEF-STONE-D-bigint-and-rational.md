# STONE D — bigint + rational get their homes: the numeric tower's last two verb families

DRAWN + BRIEFED 2026-08-26 against `11b85591e`.
PRIOR ART, and it is the whole method: **A-i** `b2d10158f` · **A-ii** `1333e90d0` · **B-i**
`ae2330bc1` · **B-ii** `870d59898` · **C** `11b85591e`. Read C's commit message before you start —
its disarmed-negative-tests finding is why this brief has a pre-check the numerics' briefs lacked.

## Why these two, and why now

Builder, 2026-08-26: *"we will add full support for all rust primitive numerics once we have the
pattern set."* The pattern is set — the numerics went home and their old spellings are a check-time
error with a remedy. These are the tower's remaining verb families.

**And the destination already exists.** `wat/core.wat:88+` (arc 300 stone C1) holds `wat.core/+` as a
**defclause with a clause per numeric type** — i64, f64, bigint, rational — each dispatching to the
per-type intrinsic. Line 106 already reads `(:wat::i64::to-bigint x)`: B-i migrated it. **Two of the
four clause targets are still in `core::`'s junk drawer. This stone finishes the set**, and after it
every arm of that defclause points at a `wat.<type>/` home.

```
:wat::core::bigint::    + - * /  to-f64  to-rational        ->  :wat::bigint::*
:wat::core::rational::  + - * /  to-f64                     ->  :wat::rational::*
:wat::core::rational/   numerator  denominator              ->  :wat::rational::{numerator,denominator}
```

The two slash-form accessors follow the recorded `:wat::core::Uuid/v4 -> :wat::uuid::v4` precedent.
`:wat::core::bigint` and `:wat::core::rational` — the bare TYPES — **do not move**; they are arc
251's `wat.type/`, and the trailing `::` (or `/`) is the whole discrimination.

## The measured ground — 140 occurrences, 5 files

```
wat/core.wat            69     ← the defclause arms. THE BOOTSTRAP. See the hazard below.
src/runtime.rs          35     impls + dispatch arms
src/check.rs            13     type schemes
src/rete/purity.rs      13     the pure/total axes
wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat   2
```

Impl shape is **identical** to i64/f64 before A-i: `eval_bigint_arith<F>` and `eval_rational_arith<F>`
are generic over a closure, so they cannot carry `#[wat_intrinsic]` themselves — write one fixed-arg
handler per name that DELEGATES. **Do not copy the arithmetic; share it.** Pass the op name as a
PARAMETER so an error names the spelling the caller used (A-i proved this both ways).

## ⛔ THE ORDER IS NOT NEGOTIABLE, AND `wat/core.wat` IS WHY

`wat/core.wat` is the FIRST file loaded. If the old spelling becomes an error while `core.wat` still
uses it, **the whole substrate fails to load and every downstream file cascades** — B-i hit exactly
this when `:wat::i64::/` was missing from `is_pure_total` and `kwargs-lower` took the entire corpus
down with it.

```
PHASE 1   register the new names.  BOTH SPELLINGS LIVE.  Nothing in the corpus moves.
PHASE 2   move the corpus by codemod.  Both spellings still work.
PHASE 3   retire the old.  36 -> now 12 retirement rows; delete the old machinery.
```

Verify the tree builds and `core.wat` loads at the end of each phase. If you retire before the
corpus moves, you will not get a helpful error — you will get everything.

## ★ THE PRE-CHECK STONE C PAID FOR — run it before Phase 3

Stone C retired the numerics and **silently disarmed eleven negative tests**. Their `.wat.bad`
fixtures used the retired spelling in executable position while their tests asserted only
`assert!(result.is_err())` — so each began passing on the RETIREMENT error instead of the defect it
was written to prove, and **nothing went red**.

I have already run that check for this stone and it comes back CLEAN — no `.wat.bad` fixture uses
`bigint`/`rational` in executable position. **Re-run it yourself before Phase 3 anyway**, because
Phase 2 may move a fixture into range:

```bash
for f in $(git grep -lE ':wat::core::(bigint|rational)[:/]' -- '*.bad'); do
  n=$(grep -E ':wat::core::(bigint|rational)[:/]' "$f" | grep -vE '^\s*;;' | wc -l)
  [ "$n" -gt 0 ] && echo "$f  $n executable"
done
```

Any hit: read that fixture's test. If it asserts a bare `is_err()`, migrating the fixture is
mandatory — and say so in your report.

## ★★ CENSUS BY EXTENSION, NOT BY GUESS

Stone C's acceptance bar read `-- '*.rs' '*.wat'` and six `.jsonl` MCP fixtures were invisible to it;
they broke the floor. B-ii found `.wat.bad` is invisible to `git ls-files '*.wat'` **by extension**.
The all-extension census for this stone is `rs` and `wat` only — I ran it — **but re-run it at the
end**, because a count taken before the work is not a count taken after:

```bash
git grep -lE ':wat::core::(bigint|rational)[:/]' -- ':!docs' | sed 's/.*\.//' | sort | uniq -c
```

## Your role

cwd `/home/john/work/holon/wat-rs`; run `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` must never be touched.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check|--grep <f>`, `./target/release/wat <f>`, and single named tests.
**Not** the floor, **not** clippy — the orchestrator measures those centrally.

The corpus move is a **wat-fix RULES codemod** — copy
`wat-scripts/fixes/rename-core-numerics-to-their-homes.wat`. Its two traps still apply:
`rename-keyword-prefix` is a **silent no-op** on `::`-terminated prefixes, and the **KEYWORD-ONLY**
guard is mandatory. ⚠ The slash forms (`rational/numerator`) are a DIFFERENT prefix shape — handle
them or report that they need a second rule.

## STOP triggers — each rejects

1. **STOP-1 — a retirement row does not fire.** Prove a retired spelling produces a CHECK-time error
   naming its replacement, and say which door. A prior stone shipped 14 inert rows.
2. **STOP-2 — a `.wat.bad` fixture is in executable range and its test asserts a bare `is_err()`.**
   Report it before Phase 3 lands, with the before/after error text.
3. **STOP-3 — `wat/core.wat` will not load at the end of any phase.** Stop there; do not push
   forward hoping a later phase fixes it.
4. **STOP-4 — a room's line number does not hold.** Written against `11b85591e`.

## Acceptance — every row derives its bar, and each measures a MECHANISM

```bash
# 1. the new names exist and RUN (not merely register).
#    a probe under wat-scripts/scratch-pad/ asserting a result for each of the 12.
./target/release/wat wat-scripts/scratch-pad/<probe>.wat; echo "EXIT=$?"      # 0

# 2. the old spelling is a CHECK error naming its replacement.
printf '(:wat::core::defn :user::main [] -> :wat::core::nil\n  (:wat::core::let [_a (:wat::kernel::println (:wat::core::bigint::+ (:wat::i64::to-bigint 2) (:wat::i64::to-bigint 3)))] nil))\n' > /tmp/d.wat
./target/release/wat --check /tmp/d.wat; echo "EXIT=$?"    # non-zero, remedy names :wat::bigint::+

# 3. the bootstrap still loads — the one that cascades if it does not.
./target/release/wat --check wat/core.wat; echo "EXIT=$?"  # 0
cargo test --release --test lint every_wat_scripts_file_loads_on_the_current_runtime

# 4. all-extension census, AFTER the work. Classify every survivor.
git grep -lE ':wat::core::(bigint|rational)[:/]' -- ':!docs' | sed 's/.*\.//' | sort | uniq -c

# 5. the TYPE did not move — arc 251's, and the discrimination is the trailing :: or /.
git grep -oE ':wat::core::(bigint|rational)' -- ':!docs' | wc -l      # before and after; derive it

cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **Which door produced the retirement error**, and its full text.
- **The negative-fixture pre-check result**, run by you, before Phase 3.
- **How you shared each implementation** — show the code, not a description.
- The four defclause arms in `wat/core.wat` after the move: paste them, so I can see all four
  numeric clauses pointing at `wat.<type>/` homes.
- The cascade's waterfall, per phase.
- Anything the brief got wrong. What you did NOT do, and why.
