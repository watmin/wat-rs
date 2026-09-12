# EXPECTATIONS — the gate outcome outlives its file

**Written before the strike.** Graded against the orchestrator's OWN reads and runs.

## Scorecard

| # | what | check | expected |
|---|---|---|---|
| 1 | the enum is Rust-registered | `grep -n '":wat::service::GateOutcome"' src/types.rs` | **1** hit |
| 2 | …and the `defenum` is gone from wat | `grep -c 'defenum :wat::service::GateOutcome' wat/service.wat` | **0** |
| 3 | the name did NOT change | `grep -rho 'GateOutcome' --include=*.wat --include=*.rs . \| wc -l` | still ~28; **zero** `kernel::GateOutcome` |
| 4 | the capability surface faces the value | read `wat/capability.wat` | all four `grant`/`revoke` features `-> :wat::service::GateOutcome`; **no `-> :wat::core::nil`** on them |
| 5 | ⭑ **a capability-tier grant RETURNS a value** | the scratch probe | prints a `GateOutcome` (e.g. `Applied`) — **not** a raise, and not merely "the floor is green" |
| 6 | `grantable-extend` + `typedcap-extend` both unwrapped | read | neither wraps `require-granted`; ⭑ **both** — the sibling check |
| 7 | the helpers survive | `grep -c 'require-granted\|gate-faced' wat/service.wat` | **> 0** — unused-but-deliberate, per STOP-4 |
| 8 | purity preserved | read the registration | `Purity::Pure`, matching `:wat::enum::Pure` |
| 9 | floor | `./scripts/floor.sh` → Summary | `5237 passed`, 22 skipped, **0 FAIL** |
| 10 | tests compile | `nextest --release --no-run` | exit 0 |
| 11 | clippy still 0 | `clippy --release --workspace --all-targets -- -D warnings` | **exit 0** |
| 12 | happy path | `… 2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 13 | chaos | `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` | exit 0, `distinct=100;dup=0` |
| 14 | `StopOutcome`/`CallOutcome` NOT moved | `git diff src/types.rs` | neither appears — unless STOP-1 fired with evidence |

## Runtime prediction

**30–60 minutes.** One registration, one deletion, four signatures, two extend bodies. The floor is
~500 s and the rebuild is mandatory (frozen stdlib).

## What makes this PARTIAL rather than passed

- **Row 5 missing** → the headline is unproven. Zero userland callers means a green floor is *weak*
  evidence; without a probe showing a value coming back, nothing demonstrates the tier changed.
- **Row 6 half-done** → `grantable-extend` unwrapped and `typedcap-extend` left wrapped is the sibling
  failure this campaign hit five times in one day. It is named in advance here for that reason.
- **Row 3 violated** → a rename churned 28 references the DESIGN rejected churning.

## Trap-doors

1. **Zero userland callers** → the floor may not execute the changed surface at all. Row 5 exists because
   of this, and it is the row I will grade hardest.
2. **`typedcap-extend` is the sibling of `grantable-extend`.** Ask about it before finishing, not after.
3. **Frozen stdlib** — `wat/*.wat` is baked in at build time; rebuild before running any `.wat`.
4. **`cargo build --release` does not compile tests.**
5. **Adding to `src/types.rs` can wake `large_enum_variant`** (there is already an `#[expect]` for it on
   `SurfaceMember`). `GateOutcome`'s widest variant is 2 fields, so it should be clear — verify.
