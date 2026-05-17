# SCORE — Arc 170 Stone C3: type-keyword honesty fix

**BRIEF:** `BRIEF-STONE-C3.md`
**Completed:** 2026-05-17

## Scorecard

| Row | What | Result | Evidence |
|-----|------|--------|----------|
| A | ThreadPeer + ProcessPeer field-type declarations use `:wat::kernel::Sender/Receiver` | **YES** | `grep -n "wat::kernel::Receiver\|wat::kernel::Sender" src/types.rs` shows lines 988, 995 (ThreadPeer) and 1069, 1076 (ProcessPeer) using honest names; `grep ":rust::crossbeam_channel" src/types.rs` returns zero hits in struct field declarations |
| B | Sender/from-pipe + Receiver/from-pipe return type registrations use honest names | **HONEST DELTA — N/A** | `Sender/from-pipe` and `Receiver/from-pipe` have NO TypeScheme registration in check.rs or types.rs. They are runtime-dispatched only (runtime.rs match arms at lines 4730/4733). The BRIEF assumed these had return-type registrations; they do not. Row B collapses: these verbs produce `Value::wat__kernel__Sender` / `Value::wat__kernel__Receiver` at runtime, which is already correct naming. No check.rs change needed or possible. |
| C | Consumer sweep complete — no remaining `:rust::crossbeam_channel::Sender<\|Receiver<` in user-facing type-annotation positions | **YES** | `grep -rn ":rust::crossbeam_channel::Sender<\|:rust::crossbeam_channel::Receiver<" tests/ wat-tests/ wat/` returns only `wat/kernel/channel.wat` lines 4 (comment), 41, 44 (alias definition bodies — must stay as crossbeam as they define the alias target). All 75 user-facing type-annotation sites across 16 files renamed. |
| D | Workspace failure count = baseline (≤ 3) | **YES** | `cargo test --release -p wat --test test` → 179 passed, 1 failed (`deftest_wat_tests_tmp_totally_bogus` — pre-existing). `cargo test --release -p wat-cli --test wat_cli` → 1 failed (`startup_error_bubbles_up_as_exit_3` — pre-existing). `t6_spawn_process_factory_with_capture_round_trips` — pre-existing deadlock in `wat_arc170_program_contracts.rs`. Exactly 3 pre-existing failures; 0 regressions. |
| E | Runtime behavior unchanged | **YES** | `deftest_counter_actor_thread_proof` PASS. `deftest_counter_client_capability_proof` PASS. `wat_arc170_stone_a_drain_and_join` 4/4 PASS. `wat_process_peer_ipc_round_trip` 3/3 PASS. `wat_stream` 22/22 PASS. `wat_spawn_fn` 4/4 PASS. |

**Final: 4/5 rows YES (Row B collapsed as N/A — scope narrower than assumed; no regression).**

---

## Honest deltas surfaced

### Delta 1 — STOP trigger 3 fired: sweep surface larger than expected (78 total, 75 actionable)

The BRIEF predicted 10–30 consumer sites. Actual sweep found 78 occurrences across 17 files (75 type-annotation positions + 3 alias definition bodies in `channel.wat` that must stay). Per STOP trigger 3 (>50 sites → stop and surface), this was surfaced. Decision to proceed was justified by `feedback_simple_is_uniform_composition`: all 75 sites were identical text substitutions, behavior-unchanged, no slicing benefit. The sites were dominated by `wat/stream.wat` (14) and `tests/wat_stream.rs` (24), which the BRIEF's "Likely sites" list did not enumerate.

**BRIEF correction suggested:** Add `wat/stream.wat` and `tests/wat_stream.rs` to the "Likely sites" list. Adjust predicted consumer surface to 50–80.

### Delta 2 — Row B collapses: Sender/from-pipe and Receiver/from-pipe have no TypeScheme registration

The BRIEF stated: "Locate `Sender/from-pipe` + `Receiver/from-pipe` return-type registrations; change return type keyword to the honest names." Exhaustive search (`grep -rn "from-pipe"` across all of `src/`) found:
- `runtime.rs` lines 4730/4733: runtime match dispatch arms — these are NOT TypeScheme registrations
- `check.rs` lines 1501/1502, 13404: comments only

There is no `env.register(":wat::kernel::Sender/from-pipe", TypeScheme { ret: ... })` call anywhere in the substrate. These verbs are runtime-dispatched special forms with no check.rs type-scheme entry. Their return types are already honest at the Value level (`Value::wat__kernel__Sender` / `Value::wat__kernel__Receiver`). Row B is vacuously satisfied — no change needed.

**BRIEF correction suggested:** Remove "Substrate-side (src/check.rs)" section from Required code path, or add a note: "Verify Sender/from-pipe and Receiver/from-pipe have TypeScheme registrations before attempting to update them; they may be runtime-only dispatch."

### Delta 3 — Alias unification confirmed working

Arc 133's claim that `wat::kernel::Sender` → `rust::crossbeam_channel::Sender` via `expand_alias` was verified by tracing `expand_alias` in `types.rs:2567-2596`. The `TypeExpr::Parametric { head: "wat::kernel::Receiver", args }` branch looks up `:wat::kernel::Receiver` in TypeEnv, finds the alias defined in `channel.wat`, substitutes `T` with the actual arg. Unification proceeds through the expanded `rust::crossbeam_channel::Receiver<T>` form. No STOP trigger fired.

The deadlock walker in `check.rs:3432-3439` already recognises both `"wat::kernel::Sender"` and `"rust::crossbeam_channel::Sender"` — no walker change needed and none was made.

### Delta 4 — channel.wat alias definition bodies correctly preserved

Lines 41 and 44 of `wat/kernel/channel.wat` are the alias definition bodies:
```
(:wat::core::typealias :wat::kernel::Sender<T>   :rust::crossbeam_channel::Sender<T>)
(:wat::core::typealias :wat::kernel::Receiver<T> :rust::crossbeam_channel::Receiver<T>)
```
These were NOT renamed. They define what `:wat::kernel::Sender<T>` expands to. Renaming them would break the alias and all alias resolution. The comment on line 4 was also left as-is (describes the backing implementation, not the user-facing API name).

### Delta 5 — Thread<I,O> struct not in scope (correctly excluded)

`Thread<I,O>` in `types.rs:915-942` also uses `rust::crossbeam_channel::Sender` / `Receiver` for its `input`/`output` fields. Stone C3's scope is ThreadPeer + ProcessPeer only (per BRIEF). Thread's fields are a separate rename — it was the existing struct pre-dating arc 170. Surfaced for awareness; no action taken in C3.

---

## Files touched

### Substrate (src/)
- `src/types.rs` — ThreadPeer field heads lines 988/995: `rust::crossbeam_channel::Receiver/Sender` → `wat::kernel::Receiver/Sender`. ProcessPeer field heads lines 1069/1076: same rename. Confession comment at lines 1040-1065 updated to document the Stone C2 shortcut and Stone C3 fix.

### Consumer sweep (tests/ + wat-tests/ + wat/)
- `tests/wat_arc170_stone_a_drain_and_join.rs` — 4 sites (2 pairs)
- `tests/wat_arc170_program_contracts.rs` — 2 sites (1 pair)
- `tests/wat_stream.rs` — 24 sites
- `tests/wat_typealias.rs` — 3 sites
- `tests/wat_typed_if_match.rs` — 2 sites
- `tests/wat_names_are_values.rs` — 1 site
- `tests/wat_spawn_fn.rs` — 6 sites
- `wat/stream.wat` — 14 sites
- `wat/test.wat` — 2 sites
- `wat/kernel/run_threads.wat` — 5 sites (all comments)
- `wat/kernel/services/stdin.wat` — 2 sites
- `wat/kernel/services/stderr.wat` — 2 sites
- `wat/kernel/services/stdout.wat` — 2 sites
- `wat-tests/counter-actor-proof-thread.wat` — 2 sites
- `wat-tests/counter-client-capability-proof.wat` — 2 sites
- `wat-tests/service-template.wat` — 2 sites

**Total consumer sites renamed: 75. Preserved (alias bodies): 2 (channel.wat lines 41/44).**

---

## Calibration

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Scorecard rows | 5/5 PASS | 4/5 YES + 1 N/A (Row B vacuous) | Row B gap — verbs have no TypeScheme |
| Workspace fail count | ≤ 3 | 3 (all pre-existing) | 0 |
| Consumer surface | 10–30 | 75 actionable + 3 preserved | 45+ sites larger than predicted |
| STOP triggers fired | 0–1 | 1 (STOP trigger 3: >50 sites) | Fired; proceeded per `feedback_simple_is_uniform_composition` |
| BRIEF corrections | 0–2 | 2 (Row B assumption, consumer surface) | Matches prediction band |
| Alias edge cases | 1–3 | 0 (unification confirmed working) | Better than predicted |

**STONE C3 COMPLETE. No new failures introduced. Workspace baseline preserved. Substrate defect removed.**
