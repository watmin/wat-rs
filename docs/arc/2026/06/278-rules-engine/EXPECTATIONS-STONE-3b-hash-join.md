# EXPECTATIONS — Stone 3b: `HashJoinNode` (the two-sided join)

Independent scorecard, fixed BEFORE the strike. Orchestrator re-runs each row + reads the diff. This is THE
HEART — weigh the join semantics hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | the join unifies + drops correctly | `cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored` | **3/3 GREEN** (2 match + 1 no-match guard) |
| 2 | root-join still green | `cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored` | 3/3 |
| 3 | alpha pass still green | `cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored` | 3/3 |
| 4 | data model / compile / matcher | `…1a_data_model / …1b_compile / …2a_alpha_match -- --include-ignored` | 1/1 · 2/2 · 3/3 |
| 5 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 6 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931/36 |
| 7 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 |
| 8 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings |

## Trap-doors named — HAZARD #1 (weigh hardest)
- **Cross direction.** LEFT = the upstream beta node's Tokens; RIGHT = the join-condition alpha's Elements. A
  swap reads as "works" on a 1×1 case but breaks on N×M. Read the code: tokens from beta-memory, elements from
  alpha-memory.
- **Compatibility = shared-var AGREEMENT.** Not "all of one side's vars present in the other"; not "any shared
  key exists." A var present on only one side must NOT cause a drop. The no-match probe (`?loc` Oslo vs Bergen
  → 0 tokens) is the canary; ALSO reason about a hypothetical extra var.
- **Extend, don't replace.** New token = `conj` the support tuple (matches grows to length 2) + `merge` the
  element's bindings into the token's (both ?t and ?w survive). A token that loses ?t, or whose matches stays
  length 1, is wrong.
- **No silent duplicate.** One Temp + one Wind at the same loc → exactly ONE joined token (not 2). Probe row 1
  asserts length 1.
- **alpha-feeding correctness.** The reverse-lookup must find the RIGHT condition's alpha (the WindSpeed alpha
  for the WindSpeed hash-join), not the Temperature alpha. If it grabs the wrong alpha, ?w wouldn't bind.
- **Termination.** The propagation fixpoint must terminate; confirm no unbounded loop (monotone + finite).
- **No scope creep.** No production firing, no cascade, no join-bindings index, no binding-keys precompute,
  no 1a/1b record change.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-8 myself; 6/7 EXACTLY baseline.
2. Read the join code line by line: cross direction, compatibility (fold element.bindings, agreement check),
   extend (conj tuple + merge bindings), alpha-feeding reverse-lookup, the fixpoint.
3. Mentally run a 2×2 case (two Temps, two Winds, two locs) → confirm only same-loc pairs join (no cross-loc
   leakage). If the probe doesn't cover it and the code looks risky, add a quick check.
4. Commit SCOPED on green; push.
