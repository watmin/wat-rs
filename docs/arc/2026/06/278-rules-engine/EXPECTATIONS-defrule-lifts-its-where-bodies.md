# EXPECTATIONS — `defrule` lifts its `where` bodies

**Written BEFORE the strike so the result cannot move the goalposts.** Scored against the
orchestrator's own re-run, never the rider's report.

Baseline at brief time: **HEAD `9ffbf9c7`**, floor **4391 passed / 0 failed / 262 skipped**, clippy
**0**.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | **The delivery defect is FIXED** — a fn called only from a `where` now ships | `./target/release/wat wat-scripts/scratch-pad/probe-arc278-where-body-dep-not-shipped.wat` | SUBJECT count **rises to equal POSITIVE-CONTROL** (today `PC 6 · BASE 5 · SUBJECT 5`). **This is the load-bearing row** — it is the bug the whole stone exists to close. |
| 2 | **Multi-`where` works** (the unmeasured case) | run the NEW probe from Deliverable 2 | two distinct lifted defns, both deps shipped, conditions in declared order; its single-`where` control still passes |
| 3 | **Rules still fire identically** — the engine is observationally unchanged | `cargo nextest run --release -E 'binary(rete)'` | all green, **no test edited to make it pass** |
| 4 | **Discovery survives** — `collect-rules` still finds rules, and does NOT pick up lifted bodies | `cargo nextest run --release -E 'test(collect_rules)'` | green; a namespace's rule count is **unchanged** (lifted defns return `:bool`, so they are excluded) |
| 5 | **The oracle and the kernel still agree** | `cargo nextest run --release -E 'test(differential)'` | green — a divergence here is the one thing the dual-impl exists to catch |
| 6 | **Both producers moved** | `grep -n "make-rule" wat/rete.wat wat/query.wat` | both emit the lifted shape; neither still quotes a bare `where` body |
| 7 | **Scratch still loads** | `cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'` | green, incl. the new multi-`where` probe |
| 8 | **The floor** | `./scripts/floor.sh` → read the **Summary line** | `4391+ passed / 0 failed` (the new probe adds no Rust test; count moves only if a rider adds one) |
| 9 | **Clippy** | `cargo clippy --release --all-targets` | **0** — no `src/` change, so any warning means the blast radius was exceeded |
| 10 | **Blast radius held** | `git diff --stat` | `wat/rete.wat`, `wat/query.wat`, and `wat-scripts/scratch-pad/` only. **Zero `src/` files.** |

## Runtime prediction

**25–45 minutes** for a rider. The macro change is one template plus a fold over the `:when` vector;
the mechanism is already proven and the worked reference (`core.wat:1147-1167`) is a direct copy. The
multi-`where` probe is the larger half of the work.

If it runs past an hour, the likely cause is the `?var`-to-parameter derivation (STOP-3) rather than
the emission — which is a signal to stop and report, not to push through.

## Trap doors — named in advance

- **The `where` body's `?var`s are pattern variables bound by the NETWORK, not lexically.** The
  lifted defn's parameter list has to come from the `<-` binders in the preceding conditions. A rule
  whose `where` references a var bound in a condition the macro hasn't walked yet will not resolve.
  → STOP-3.
- **Hygiene gate E will fire if the mention's binder is written literally** in the template
  (`ProgramBodyIntroducesName`, arc 249 stone 249.2b-ii). It must be spliced via
  `~(:wat::core::symbol-node "$where0")`, exactly as `core.wat:1163` does. This *will* be hit; the
  probe already carries the fix.
- **The index is UNMEASURED.** Everything proven so far used exactly one `where` per rule. Two
  conditions in one rule is genuinely new ground — hence Deliverable 2, and hence row 2 being the
  row most likely to surprise.
- **A `.wat`-only sweep is blind to inline wat in Rust test strings.** If row 3 or 8 goes red naming
  a `src/**/*.rs` fixture, that is this known class (recorded 2026-07-24), not a mystery.
- **Row 3 has an obvious wrong way to pass it:** editing a rete test until it agrees. If a rete test
  needs changing to accommodate the lift, that is a finding to report, not a fix to apply.

## How this is scored

Every row re-run by the orchestrator, on a quiescent tree, before anything is credited. The floor is
read from the **Summary line** — never a piped exit code, never a rider's report. On any red: do not
re-run; capture the failing block verbatim; name the exact arm; surface it.
