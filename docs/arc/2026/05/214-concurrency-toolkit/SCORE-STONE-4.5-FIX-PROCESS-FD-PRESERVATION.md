# SCORE — Stone 214.4.5-fix (process fd preservation)

Scored by the orchestrator against an INDEPENDENT re-run (not sonnet's report).
Runtime: ~9.5 min (under the 15–25 min band). Mode A.

## Scorecard

| # | Claim | Verdict | Evidence (orchestrator's own run) |
|---|---|---|---|
| 1 | comms `Sender`/`Receiver` expose complete owned-fd accessors | ✅ | `raw_fds` on both; `Receiver` returns `[read_fd, ring.borrow().as_raw_fd()]` (read `process.rs`) |
| 2 | **`close_inherited_fds_above_stdio` honors the FULL skip-list** (class kill) | ✅ | Read the body: sort+dedup, sweep `[3,first-1]` + each gap + `[last+1,MAX]`, saturating arith. `skip[0]` gone from executable code (only a doc comment remains). |
| 3 | `child_post_fork_init_preserving` is the single impl; bare init delegates `&[]` | ✅ | Read `fork.rs:522`; `child_post_fork_init` is now the thin wrapper |
| 4 | `:process` child passes its comms fds to the preserving init | ✅ | Read `spawn.rs` child closure: `input_rx.raw_fds() ∪ output_tx.raw_fds()` |
| 5 | **Both `:process` probes PASS single-threaded** (the fix works) | ✅ | `spawn_program_prime_process -- --ignored --test-threads=1` → **2 passed; 0 failed** (0.23s) |
| 6 | 4.4 reference still green | ✅ | `peer_process_round_trip -- --ignored` → **1 passed** |
| 7 | Library band green | ✅ | `cargo test --release --lib -p wat` → **940 passed; 0 failed; 1 ignored** |
| 8 | No new clippy warnings in touched files | ✅ | comms/process.rs: 0 clippy hits. fork.rs's 7 hits are PRE-EXISTING, shifted ~55 lines down by the insertion (620−55=565, 858−55=803, 1164−55=1109, …), all outside the 380–430 diff. Lib-wide 237 = standing flat-quarry debt. |
| 9 | Tree dirty (sonnet did not commit) | ✅ | `git status` showed 4 modified files |

## Honest deltas / corrections

- **DESIGN over-claimed "both touched files are WARDED homes."** Reality: `comms/`
  IS warded (stamp `comms/mod.rs:1`, clippy-zero) and `comms/process.rs` stayed
  clippy-clean → the home stamp HOLDS, no re-cast needed for a small additive
  accessor. `src/fork.rs` is a **FLAT quarry** (no vigilatum stamp), like
  runtime.rs — no stamp to drift; the bar is "add no new clippy warnings,"
  which held.
- **Serialization is documented, not structural.** The two `:process` probes each
  fork; cargo runs them on parallel threads (fork-from-multithreaded-parent, FM
  7-ter). Sonnet's fix: `--test-threads=1` in the `#[ignore]` strings + docs.
  `integration-run.sh` does NOT pass `--ignored`, so it never runs these probes —
  the only run vector is manual `--ignored`, where the documented flag applies.
  **Affirmatively bounded:** the structural "forking integration tests run
  serially by construction" belongs to the test-surface settling (the
  ignore-drawdown / Slice 7–8), NOT this stone. This is a convention, honestly
  labeled — not a hidden deferral.

## Verdict

**GREEN.** Stone 214.4.5 is now COMPLETE: both `:thread` and `:process` tiers of
`spawn-program'` round-trip green. The `:process` fd-lifecycle bug is fixed AND
the silent-`skip[0]` class is annihilated (a class kill, not a site patch).
Next: **4.6 polymorphic verbs** (`send'`/`recv'`/`select'`/`close'` multimethod
over the peer types).
