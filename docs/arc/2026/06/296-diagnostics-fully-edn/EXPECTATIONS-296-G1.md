# EXPECTATIONS — 296 G-1

Written **before** the strike, so the result cannot move the goalposts. Every row is scored against
the orchestrator's own independent re-run, never the rider's report.

Baseline on disk at brief time: **HEAD `35586f51`, floor 4417/4417 passed / 263 skipped, clippy 0**
(weighed at `75e62e8e`; the two commits since are doc-only — verified by `git show --stat`).

---

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the carrier exists | `sed -n '/pub struct AggregateValue/,/^}/p' src/value/value.rs` | a `names: Arc<Vec<String>>` field, documented, sitting before `fields` |
| 2 | all three constructors take it | `grep -n "pub fn struct_\|pub fn record\|pub fn holon_record" src/value/value.rs` | `names` in the same position in all three |
| 3 | **the floor holds** | `scripts/floor.sh` → Summary line | `4417 passed` (± only the golden in row 6), `0 failed` |
| 4 | clippy stays silent | CI's invocation (`--workspace`, `-D warnings`) | 0 |
| 5 | **no human typed a field name** ★ | `grep -rn 'names.*vec!\["' src/` + read every class-C site's diff | zero literal name vectors; every class-C site traces to a `wat_field_names_from!` const |
| 6 | the predicted golden moved, and only it | `git diff --stat tests/` | `probe_arc234_stone1_wat_record_variant.rs` updated to include `names`; no other test's *assertion* weakened |
| 7 | **the generic constructor carries the registry's names** ★ | read `src/runtime.rs:15834/15837/15843` | all three arms pass `agg.names_arc()`; the `:15812` unregistered-class error is untouched |
| 8 | class B carries the source's own | read the 5 rebuild sites | `names` cloned from the same binding that `class` is cloned from |
| 9 | the fallback is still standing | `grep -c 'format!("field-{}", i)' src/edn_shim.rs` | **7** — G-2 deletes these, not G-1 |
| 10 | no `.wat` file changed | `git status --short wat/` | empty |
| 11 | STOP-1 surfaced, not improvised | read `src/channel/transfer.rs:352,359` | reported unresolved — **not** given a literal, an empty vec, or a fresh wat declaration on the rider's own authority |

★ = **load-bearing.** Row 5 is the reason this stone exists at all; row 7 is the site every user
aggregate flows through. A green everywhere else with either of these wrong is a failed strike.

---

## What "row 5 green" actually requires

Row 5 cannot be scored by grep alone, and saying so up front is the point — a `grep` census on this
arc has been wrong five times, most recently the design's own file table. Scoring it means **reading
every class-C diff hunk** and confirming each name vector's provenance is a const generated from a
`.wat` declaration. The grep is a fast negative check (it can prove a violation exists); only the
read can support the positive claim.

---

## Runtime prediction

**45–90 minutes.** ~78 constructor call sites across 15 files, plus a struct literal and a golden.
Mechanical once classified, and the classification is supplied. Time-box at **2× the upper bound =
180 minutes**; on overrun, stop and score as a time violation — the overrun is itself data about
whether the brief's classification held.

---

## Trap-doors named in advance

1. **Parametric declarations are ORDINARY** (builder's ruling, 2026-08-15 — *"structs and records are
   parametric, should not be an issue"*). The rule is simply *name the type as it is declared*. If a
   bare path is passed for a parametric type the macro errors out loudly, which is correct. **The
   failure mode to watch for is the rider "fixing" `field_names_of` into a prefix match.** That would
   be the fourth instance on this crusade of a name comparison with one side normalized and the other
   not — the class that tore `State<K,V>` into `State<K` + `V>`. Exact-match is the safe direction
   and stays.
2. **`:wat::kernel::ThreadPeer` (STOP-1)** is the one site with no honest source. The tempting moves
   are an empty `names` vector (silently wrong, arity mismatch waiting) or a two-element literal
   (the exact thing the builder stopped). Either one scores row 11 red even if the floor is green.
3. **The first build's error count will be large.** The risk is not the count; it is a rider reading
   it as a crisis and proposing a revert. The count is the progress meter.
4. **The two doors have never been invoked.** `names_arc()` has zero call sites and
   `wat_field_names_from!`'s ~20-line wrapper has never expanded (its *reader* is well-proven — 13
   invocations via `wat_record_from!`). **The first converted site is their first real exercise**, so
   a failure there is a door defect, not a site defect — and it belongs in the report as such rather
   than being worked around at the call site.

---

## What would make this a Mode B

- The floor is not green and the reds are not the predicted golden.
- Row 5 fails: any field name typed by hand into Rust.
- STOP-1 resolved by the rider instead of surfaced.
- `edn_shim.rs`'s fallbacks deleted (that is G-2; deleting them here makes both cascades' screams
  ambiguous, which is the one thing the split exists to prevent).
