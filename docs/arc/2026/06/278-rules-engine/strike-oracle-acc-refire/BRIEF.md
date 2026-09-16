# BRIEF — the oracle accretes superseded accumulate results

Make the oracle stop keeping facts derived from an accumulate result that has since changed, so it
returns what Clara returns. Read `DESIGN.md` first — its ★ ONE CONTRACT DECISION rules out the
shortcut, and its ⚠ names the way this ships a worse defect than it cures.

## Read in order, and why

1. **`wat/rete/oracle/stratify.wat:10-28`** — the assumption in the oracle's own words: *"correct
   within a stratum (**monotone**…)"*. **An accumulate is not monotone.** That paragraph is the
   defect, stated by the file that relies on it.
2. **`wat/rete/oracle/fire.wat:238`, `fire-fixpoint`** — `merge-facts(old, derived)` is add-only,
   and the loop stops on `length(new) == length(old)`. **Both halves are in scope.**
3. **`wat/rete/oracle/accum-pass.wat`** — where an accumulate result becomes a token. Its header
   explains why the dispatch is inlined per-fold (invariant parametric types), which will shape
   whatever you do here.
4. **`src/rete/kernel/fire/pass/accumulate.rs`** — read it to understand *what correct looks like*,
   then **do not port it.** ★ decision.
5. **The three probes, banked beside this file** — `probe-always-empty.wat.txt` (all three engines
   agree, `n=0` — your regression guard), `probe-two-changes.wat.txt` (native 1, oracle 3), and
   `clara-two-changes.clj`, run exactly as the grid runs Clara:
   `clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["."]}' -M -m t2`

## The order

1. Land the probes as a test. **Confirm RED**: oracle 3 where Clara says 1. Quote it.
2. **Re-run the Clara reference yourself** — do not take DESIGN's table on trust. It is one command
   and it is the only authority in this strike.
3. Fix the oracle.
4. GREEN on all three shapes, **including the always-empty one** — that is the regression guard,
   and over-correcting into "never emit" would pass the two changing shapes and break it.
5. Mutation-prove: restore the monotone `merge-facts` → the changing shapes redden, the empty one
   stays green.

## STOP triggers

1. **If the fix requires porting native's delta logic, STOP.** ★ decision — that makes every future
   differential vacuous.
2. **If you change the termination test, say so loudly and prove it.** `length(new) == length(old)`
   cannot survive retraction; a set comparison must replace it, and a fixpoint that terminates
   early is a silent wrong answer, which is worse than the defect.
3. **If the fix reaches a third oracle file, STOP and surface it.**
4. **If the always-empty shape stops emitting `n=0`, STOP.** All three engines agree it should.

## What the report must show

Per shape, the **four-way** result: Clara, native, oracle-before, oracle-after. Clara is the
authority; native's agreement is corroboration, not the definition. If the oracle now agrees with
Clara by a route you cannot explain, say so — that is not a pass.
