# EXPECTATIONS — Arc 203 Slice 2: Counter/Client capability proof

**BRIEF:** `BRIEF-SLICE-2.md`
**Drafted:** 2026-05-17, pre-spawn.

## Independent prediction

**Runtime band:** 30-60 min sonnet.

Reasoning:
- Pure consumer slice; ZERO substrate edits
- One new wat-tests file (~80-130 lines: enum decls + Client struct-restricted + 5-6 defns + 1 deftest body)
- Counter actor pattern is established — `wat-tests/counter-actor-proof-thread.wat` shipped at `9b0c517` serves as the structural template
- Main NEW shape: replacing bare ThreadPeer with `:counter::Client` struct-restricted bundling
- Substrate verification of `uuid::v4` FQDN: 30 sec grep
- Sonnet writes the file, runs cargo test on it, iterates if test fails

Smaller than slice 1's 60-90 min because no substrate work + template exists. Larger than zero because the capability wrapping is novel and sonnet may surface 1-2 honest deltas around exact wrapping patterns.

**Time-box:** 90 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — Client mints + deftest compiles | YES | high (slice 1 substrate proven to work for arbitrary structs) |
| B — End-to-end round-trip succeeds | YES | high (Counter actor pattern + assertions are established; transplant from prior proof) |
| C — Workspace failure count = baseline | YES | high (purely additive consumer test) |
| D — Capability pattern matches DESIGN | YES | medium-high (server-side accesses must be inside `:counter/`-prefixed defns; sonnet must place the dispatch + spawn under `:counter/` correctly) |

**4/4 PASS predicted; ~85% confidence overall.** High because consumer pattern is well-templated; lower than 100% because the dispatch fn's placement under `:counter/` namespace (so it can legally read restricted accessors) is a structural decision sonnet might trip on initially.

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **`uuid::v4` FQDN exact spelling** — arc 091 shipped `uuid::v4`; the exact path is `:wat::measure::uuid::v4` per arc 091 slice 2, but verify via grep of `crates/wat-measure/`. If sonnet's first guess differs from the actual FQDN, that's 1 minor iteration.

2. **`uuid::v4` return type** — sonnet needs to know if it returns `:wat::core::keyword` or some uuid-typed wrapper. The Client struct expects `server-id <- :wat::core::keyword`, so coercion may be needed (or the existing return type IS keyword and it composes directly).

3. **Recursive defn type-check** — `:counter/dispatch` is recursive (tail-calls itself); arc 166's defn supports this but sonnet may need to verify the pattern works under the new Client-carrying signature.

4. **Wrapper fn structure** — `:counter/get`/`:counter/increment`/etc. need to access `Client.in!`/`Client.out!` AND be callable from outside `:counter/` (test body invokes them). The wrappers themselves are under `:counter/` namespace so they can read restricted fields, but they need to be callable from anywhere. Since arc 198/203 restrictions are on the SYMBOL's CALL-SITE not its DEFINITION-SITE, wrappers defined under `:counter/` are callable from `:user::` (the deftest body's prefix). Verify this works as expected.

### Less likely surprises

5. **Form parse divergence** — slice 1 ships parser; if the BRIEF's specific Client form has a syntactic edge case (e.g., parametric type args in field types like `Sender<counter::Request>`), parser may surface a hint. Slice 1 tests covered varied shapes but not parametric-typed fields specifically.

6. **`:counter::Client/server-id` accessor name format** — substrate auto-synthesizes `Type/<field>` for accessors. For nested type names like `:counter::Client`, the accessor is `:counter::Client/server-id` (uses `/` separator between type and field per arc 109). Sonnet should follow established convention; surface if accessor naming differs from `:counter::Client/server-id`.

## Workspace baseline (verified post-slice-1 commit `26c9298`)

`cargo test --release --workspace --no-fail-fast` baseline: clean except 3 pre-existing stable failures:
- `deftest_wat_tests_tmp_totally_bogus`
- `startup_error_bubbles_up_as_exit_3`
- `t6_spawn_process_factory_with_capture_round_trips` (NB: this test deadlocks rather than fast-fails; cargo's no-fail-fast waits indefinitely unless manually reaped — orchestrator handles)

Post-slice-2 target:
- Pass count: ≥ baseline + 1 (one new deftest passes)
- Fail count: ≤ 3 (no regressions)

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 30-60 min | TBD | TBD |
| Scorecard rows | 4/4 PASS | TBD | TBD |
| Workspace fail count | ≤ 3 + variance | TBD | TBD |
| New deftest count | 1 | TBD | TBD |
| Substrate↔assumption gaps surfaced | 1-3 (uuid FQDN, accessor naming, wrapper placement candidates) | TBD | TBD |
| BRIEF corrections suggested | 0-2 | TBD | TBD |
| STOP-triggers fired | 0-1 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
