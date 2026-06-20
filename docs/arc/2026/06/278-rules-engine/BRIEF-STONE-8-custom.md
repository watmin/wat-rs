# BRIEF — Stone 8-custom: custom accumulators (user fold fn over the gather) + the fence

**Single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `./target/release/wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.
**After ANY `wat/rete.wat` edit, run a rete.wat-loading test** (`cargo test --release -p wat --test
probe_arc278_8a_accumulate_oracle`) — `cargo build` does NOT type-check wat.

## The work (one paragraph)
Today the accumulator slot only accepts the 8 built-in folds; a non-built-in head panics ("unknown
accumulator"). Generalize the dispatch so the slot accepts **any user fold fn**: `(?r <- (:my-fold ?v) :from
(…))` gathers the `?v` values into a `PV<T>` and applies `my-fold : (PV<T>) -> R`, binding `?r = R`. Add a
**compile fence**: a user fold must be `pure ∧ deterministic` (the same 6a fence `where` uses) or the rule is
rejected at compile. Ship oracle + native + the differential in this one strike. Contract:
`DESIGN-STONE-8-custom.md`.

## The dispatch rule (both impls)
On the acc-form head keyword: **head ∈ the built-in set (`:wat::rete::acc::count` … `group-by`) → the existing
fast-path fold. Else → the head is a USER fn name → evaluate `(head-fn <gathered-PV>)`** where `<gathered-PV>`
is the bound `?var` values collected into a `PV<T>` (the same gather `acc::sum`/`distinct` already do). The
result is the bound value. v1 = value-folds only (fact-folds are a separate item).

## Read in order (the rooms)
1. `DESIGN-STONE-8-custom.md` — the contract (gate on `pure∧det`; `total?` is OUT, separate).
2. **`src/rete/kernel.rs` `accumulate_value` (`:1213`–`:1313`)** — the `match head` with built-in arms + the
   `other` arm at `:1313` (`panic!("unknown accumulator")`). The `other` arm becomes: resolve the user fn from
   `sym`, gather the `?var` values into a `PV` Value, **eval `(user-fn PV)`**, return the result.
   - **`accumulate_value` needs `sym` (and the eval env) threaded in** — it currently takes `(acc_form,
     gathered)`. Add `sym: &SymbolTable` (+ env if needed). Update its caller (the accumulate-pass in
     `fire_fixpoint_delta`) to pass them — the filter-pass already has `sym` in scope there.
3. **`src/rete/matcher.rs` `eval_test_core`** — the MODEL, and it gives you the concrete mechanism: it builds
   a child env binding each `?var`, then `eval_inner`s the expr. **Do exactly that for custom-accum: bind the
   gathered `PV` to a synthetic var in a child env (e.g. `"__acc__"`), then eval the expr `(user-fn __acc__)`
   via the same `eval_inner` path.** This is the *proven* `where` mechanism (so the capability demonstrably
   exists — see STOP-1, which should NOT trigger). Build the `(user-fn __acc__)` call AST once from the
   acc-form's head fn-name.
4. **`wat/rete.wat` `accumulate-pass-for-token` (`~:1752`)** — the oracle dispatch (per-fold). Add the
   unknown-head arm: **apply the user fn to the gathered PV** (a normal wat fn-application over the gathered
   values). Mirror the native verdict exactly (same gather, same result).
5. **`wat/rete.wat` `compile-condition` accumulate-branch (`~:599`–`:685`)** + the **where-branch fence
   (`:534`–`:540`)** — the MODEL for the fence. In the accumulate-branch, when the acc-form head is NOT a
   built-in `:wat::rete::acc::*`, assert `(and (:wat::rete::pure? <fn-or-form>) (:wat::rete::deterministic?
   <fn-or-form>))` and raise (`Option/expect` → None) if false, message "custom accumulator must be pure and
   deterministic". A built-in head skips the fence.
6. `tests/probe_arc278_8custom_native_differential.rs` — the contract, RED now (4 tests: 3 differential folds
   + 1 fence-reject). Do NOT weaken it.

## Built-in detection
The built-in set is the existing `:wat::rete::acc::{count,sum,min,max,mean,distinct,all,group-by}`. Detect
"is built-in" by the head string prefix `:wat::rete::acc::` (or an explicit set) — anything else is a user fn.

## Blast radius (bounded)
`wat/rete.wat` + `src/rete/kernel.rs` ONLY (+ the `accumulate_value` signature + its one caller). No
`matcher.rs` *edits* (read it as the model), no `runtime.rs`/`check.rs`. No `total?` (separate item).

## STOP triggers (halt + surface; do not improvise)
1. If evaluating a user fn over the gathered PV from the accumulate-pass has **no clean path** (can't reach
   the fn-application / `eval_inner` with `sym` from there) — STOP, report exactly what's missing (this is the
   load-bearing mechanism).
2. If the `pure?`/`deterministic?` fence can't be applied to a user fold fn at compile — STOP, report.
3. If the differential stays RED (native ≠ oracle) and you can't localize it — STOP; report native vs oracle
   + hypothesis. Do NOT weaken the probe.
4. If greening needs anything beyond `wat/rete.wat` + `kernel.rs` — STOP.

## Done = green
`cargo test --release -p wat --test probe_arc278_8custom_native_differential` → 4/4. AND no regressions:
`--test probe_arc278_8a_accumulate_oracle` → 5/5 ; `--test probe_arc278_8b_accumulate_native_differential` →
5/5 ; `--test probe_arc278_7exists_native_differential` → 5/5 ; `--test
probe_arc278_northstar_cold_and_windy -- --include-ignored` → 1/0. Then `cargo build --release` clean +
`cargo test --release -p wat --lib -- --test-threads=1 | grep result` → 941/36.

## Report back
The exact diffs to `wat/rete.wat` + `src/rete/kernel.rs` (esp. the `other`-arm fn-eval + the signature change
+ the compile fence), every test count from Done (verbatim), and any STOP. Your final message is all I see.
