# BRIEF — Arc 206 Slice 3: retire telemetry's duplicate UUID impl; close arc

**Predecessors:** Slice 1 (`4ff2b72`) minted `:wat::core::uuid::v4`; Slice 1.5 (`b56e272`) minted `:wat::core::uuid::v5`; Slice 2 (`74d7fea`) shipped INSCRIPTION + USER-GUIDE + 058 row claiming arc closure.

**User direction 2026-05-17:** *"arc 206 is falsely closed - new small edition to make telemetry use the correct calls - the dependency on edn is no longer necessary."*

Slice 2's INSCRIPTION wrongly framed "separate-impl wins over alias-chain" as the correct backward-compat pattern. User overruled. The crack: `wat-telemetry` shipped a duplicate `:rust::telemetry::uuid::v4` Rust shim pulling its own `wat_edn = { features = ["mint"] }` Cargo dep solely for UUID minting, while the substrate-core path (`:wat::core::uuid::v4`) does the same thing one layer up. Telemetry should delegate at the wat-side AND drop the wat-edn dep entirely.

## DECAY DISCLOSURE — orchestrator-side WIP exists; treat as untrusted input

**The working tree is dirty with orchestrator-written edits** for slice 3. They were applied directly by the orchestrator (Opus), violating the "code changes delegate to sonnet" protocol. User direction: *"don't discard good work - just continue while adhering to the protocol."* So this BRIEF treats the existing edits as a STARTING-POINT HYPOTHESIS sonnet verifies independently — not as ground truth. Sonnet's first action is the verification gate below; if any orchestrator edit is wrong, sonnet corrects it.

The 5 orchestrator edits on disk (uncommitted):

| File | Orchestrator change |
|---|---|
| `crates/wat-telemetry/Cargo.toml` | Drop `wat-edn = { features = ["mint"] }`; add `uuid = "1"` |
| `crates/wat-telemetry/wat/telemetry/uuid.wat` | Rewrite to delegate `:wat::telemetry::uuid::v4` → `:wat::core::uuid::v4` |
| `crates/wat-telemetry/src/workunit.rs` | Switch `wat_edn::new_uuid_v4()` → `uuid::Uuid::new_v4()` at construction site |
| `crates/wat-telemetry/src/lib.rs` | Drop `pub mod shim;` + `shim::register(builder);` + update doc comments |
| `crates/wat-telemetry/src/shim.rs` | DELETED (was just the retired `:rust::telemetry::uuid::v4` registration) |

## Verification gate (sonnet's first action)

Before any new code edits, sonnet:

1. **Baseline check on disk state.** Run `git status --short` from `/home/watmin/work/holon/wat-rs/`. Confirm the 5 changed/deleted files above are present and no other files appear. If extra files appear, surface as STOP-trigger 1.
2. **Independent FM 9 baseline.** `git stash push -u -m "slice 3 verify-baseline"` → `cargo test --release --workspace --no-fail-fast 2>&1 | grep FAILED` → record the failure set. Expected: 4 pre-existing baseline failures (`lifeline_pipe_zero_orphans_across_100_trials`, `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`). Pop the stash. Re-run with edits — expected: ≤4 failures, none introduced by slice 3.
3. **Verify wat-telemetry crate green.** `cargo test --release -p wat-telemetry --no-fail-fast` should pass 36/36.
4. **Grep for stale references to `:rust::telemetry::uuid::v4`.** `grep -rn "rust::telemetry::uuid" --include="*.rs" --include="*.wat" --include="*.toml" .` — expect NO hits inside `crates/wat-telemetry/` after edits; any external references (consumer crates / docs / lab) must be surfaced.
5. **Grep for `wat_edn::new_uuid_v4` callers outside wat-edn.** `grep -rn "wat_edn::new_uuid_v4" --include="*.rs" .` — expect zero hits in `crates/wat-telemetry/`; any other consumers are out of slice 3's scope but worth surfacing.

If verification gate passes, proceed. If any orchestrator edit is wrong, fix it and re-verify before moving on.

## EDN-roundtrip proof requirement (the user's load-bearing ask)

User direction: *"we need to prove we can still comm uuids over edn seralization correctly if we haven't."*

The wat-edn layer's UUID handling (`Value::Uuid`, `#uuid "..."` parse + write, JSON serialization at `src/json.rs:157` + `:474`, accessor tests, full conformance) is INTACT — slice 3 only retires the *minting-shim-from-telemetry* path; serialization machinery is untouched. Existing coverage:

- `crates/wat-edn/tests/spec_conformance.rs::uuid_canonicalized` — EDN parse roundtrip
- `crates/wat-edn/src/json.rs::inst_and_uuid` — JSON roundtrip including `#uuid`
- `crates/wat-edn/tests/uuid_v4_mint.rs` — arc 092 minting roundtrip
- `crates/wat-edn/tests/accessors.rs::as_uuid_matches` — accessor coverage

**Sonnet audits this coverage.** If it does prove "a UUID minted somewhere serializes through EDN end-to-end," cite the specific test(s) as the proof in SCORE — no new test needed. If there's a gap (e.g., no test mints via `:wat::core::uuid::v4` AND roundtrips through EDN), add ONE small test:

- Suggested location: `tests/wat_arc206_uuid_edn_roundtrip.rs` (wat-level test driving substrate-core UUID minting + wat-edn serialization at runtime)
- Suggested shape: mint via `(:wat::core::uuid::v4)` → write via `(:wat::edn::write ...)` → read via `(:wat::edn::read ...)` → assert equality
- Whether this test belongs at the wat-rs root tests/ or at `crates/wat-edn/tests/` is sonnet's call based on what surface the test exercises

Sonnet's judgment on whether the existing tests cover the requirement. If they do, no new test ships; SCORE row cites the existing coverage.

## Closure paperwork

After verification + (optional) roundtrip test:

1. **Write `docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION-SLICE-3.md`** — forward-correction inscription. Names slice 2's "separate-impl wins over alias-chain" lesson as WRONG (per `feedback_inscription_immutable`: slice 2 INSCRIPTION stays unchanged on disk; slice 3 INSCRIPTION supersedes the lesson). Documents what slice 3 retired: the duplicate `:rust::telemetry::uuid::v4` shim, the `wat-edn = { features = ["mint"] }` Cargo dep, the `src/shim.rs` file. Names the EDN-serialization-still-works proof explicitly with citation. Affirms arc 206 closure conditions: substrate-core verbs ship + telemetry delegates + no `wat-edn` dep on telemetry crate + EDN roundtrip proven.
2. **Update `docs/arc/2026/05/206-uuid-substrate-promotion/DESIGN.md`** — DESIGNs are living per FM 13. Update the Slices table to include slice 3. Update "Out of arc 206 scope" sections if any item moved into scope. Correct any text claiming "separate-impl is the cleaner backward-compat pattern" — that lesson reversed. Do NOT amend INSCRIPTION-SLICE-2 (immutable).
3. **Update `docs/USER-GUIDE.md` § 11 "Identifiers — UUID generation"** — the "Backward-compat note" subsection currently says "`:wat::telemetry::uuid::v4` still works; the substrate-level promotion in arc 206 added `:wat::core::uuid::v4` without retiring the telemetry alias." Update to reflect slice 3's reality: telemetry's wat verb now DELEGATES to substrate-core; the duplicate Rust shim retired; consumers should reach for `:wat::core::uuid::v4` directly but the telemetry alias keeps compiling.
4. **Append a correction row to `/home/watmin/work/holon/holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`** — dated 2026-05-17; names the slice 3 correction; cites the slice 2 row as the artifact being forward-corrected (NOT edited — append-only per `feedback_inscription_immutable`). Format mirrors the existing arc 200/201/202 rows in the same file.
5. **FM 11 pre-INSCRIPTION grep** on `INSCRIPTION-SLICE-3.md` per the canonical pattern (full pattern list at `docs/COMPACTION-AMNESIA-RECOVERY.md` § Failure mode 11). MUST come back empty. If any match, rewrite to affirmative-out-of-scope language before commit.
6. **Two atomic commits** — one in wat-rs (slice 3 code + paperwork), one in lab repo (058 row). Push both.

## HARD constraints

- DO NOT touch `crates/wat-edn/` — the wat-edn UUID handling is intact and slice 3 is about telemetry retiring its duplicate, not modifying wat-edn
- DO NOT amend slice 2's INSCRIPTION (`docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION.md`) — historical record per `feedback_inscription_immutable`; slice 3 INSCRIPTION names the correction
- DO NOT edit prior 058 rows — append-only
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks
- DO NOT operate in `.claude/worktrees/` — illegal per FM 7-bis (recovery doc); cwd anchor is `/home/watmin/work/holon/wat-rs/`; use `git -C <path>` for any cross-repo git op
- DO NOT extend scope to other consumers of `wat_edn::new_uuid_v4` (e.g., if you grep and find some lab-side caller, surface as out-of-scope finding; arc 206 is telemetry-only)
- DO NOT introduce backward-compat aliases beyond what's already in the wat-side `uuid.wat` file (the telemetry alias delegating to substrate-core IS the backward compat)
- DO NOT add the UUID roundtrip test unless gap audit shows existing coverage is insufficient — name the existing test in SCORE if it's already covered

## STOP triggers (surface immediately, don't work around)

1. **Verification gate finds orchestrator edit is wrong** — surface the specific edit + diagnostic; correct + re-verify before proceeding
2. **Workspace baseline regresses beyond 4 pre-existing failures** — the 4 are `lifeline_pipe`, `tmp_totally_bogus`, `t6_spawn_process`, `startup_error_exit_3`; anything new is slice 3's fault
3. **`wat-telemetry` crate tests drop below 36/36** — slice 3 should preserve this exactly
4. **External consumer references `:rust::telemetry::uuid::v4`** outside `crates/wat-telemetry/` — surface; do NOT silently delete/migrate that consumer
5. **EDN UUID roundtrip cannot be proven** via either existing tests or a 1-test addition — surface; it's load-bearing for closure per user direction

## SCORE methodology

`docs/arc/2026/05/206-uuid-substrate-promotion/SCORE-SLICE-3.md` with these rows (atomic YES/NO; no "medium"):

| Row | Evidence |
|---|---|
| A — Verification gate passed (5 checks above) | Each check's command + result inscribed |
| B — `wat-telemetry` crate 36/36 green after edits | `cargo test --release -p wat-telemetry` output |
| C — Workspace baseline preserved at ≤4 pre-existing failures | `cargo test --release --workspace --no-fail-fast` failure list matches expected baseline |
| D — EDN UUID roundtrip proven | Citation of existing test OR new test added at named location |
| E — No `wat-edn` dep on `wat-telemetry` crate | `grep wat-edn crates/wat-telemetry/Cargo.toml` returns empty |
| F — No `wat_edn::*` calls in `crates/wat-telemetry/src/` | `grep wat_edn crates/wat-telemetry/src/` returns empty |
| G — No `:rust::telemetry::uuid::v4` references inside `crates/wat-telemetry/` | grep returns empty |
| H — INSCRIPTION-SLICE-3.md FM 11 grep clean | Pattern-list grep returns empty |
| I — DESIGN.md forward-corrected for slice 3 | Diff inscribed in SCORE |
| J — USER-GUIDE backward-compat note updated | Diff inscribed in SCORE |
| K — 058 row appended (lab repo) | Commit hash + file path inscribed |
| L — Both commits pushed | Remote tip hashes inscribed |

## Honest delta watch

Surface honestly in SCORE if:
- Any orchestrator edit needed correction
- The roundtrip-test gap was real (had to add a test) vs. existing coverage was sufficient
- Any consumer of `wat_edn::new_uuid_v4` exists outside the slice's scope (`grep -rn "wat_edn::new_uuid_v4" --include="*.rs" .` — if any hits outside `crates/wat-edn/`)
- Workspace baseline shifted shape (e.g., `lifeline_pipe` toggled — that one's flaky in known ways per the FD-multiplex inscription history)

## Time-box

Predicted 30-60 min sonnet. Hard stop 75 min. The verification gate + paperwork are most of the work; code edits are minimal (orchestrator already wrote them; sonnet verifies + corrects + ships).

You are launching now. T-minus 0.
