# EXPECTATIONS — D3: the beta-write door

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the four bypasses are gone | `grep -rn 'beta_written' src/rete/` shows it ONLY inside the doors and the census decl |
| 2 ★ | the bypass is a COMPILE ERROR | re-introduce one → `error[E0616]` (or equivalent). **Quote it.** A test failure is not the proof |
| 3 ★ | the doc's claim is now TRUE | `pass/mod.rs:27-28` says a future site cannot push without counting — and now it cannot |
| 4 ★ | the census still counts the same | every `*_cost` / census / `round_census` gate green, **numbers unmoved** |
| 5 | round reset survives | `delta.rs`'s `.beta.clear()` has a named door; no `&mut BetaMemory` escapes |
| 6 | floor | `./scripts/floor.sh` → **0 failed** |
| 7 | clippy | rc=0 |
| 8 | blast radius | `src/rete/kernel/` only |

★ load-bearing. **Row 6 is the deliverable.**

## Runtime prediction

45–75 min. Privatising the field and threading the doors is the work; the four call sites are copies.

## Trap doors, named in advance

- **A moved census number.** These four sites are where `beta_written` fires. If a `*_cost` gate's
  number changes, the cure changed behaviour — STOP-3, and it is a finding.
- **`left_activate_join` uses `extend`, the door uses `reserve` + `extend_from_slice`.** Behaviourally
  identical, but if a cost gate moves, that is why — say so rather than adjusting the gate.
- **A door that hands out `&mut`.** `writer()`-style is fine; a `beta_mut()` accessor defeats the
  whole strike.
- **The existing census gate still cannot reach these sites** (no `:where` in any census world). That
  is CUT, not fixed — do not let a green census read as coverage of the filter path. Say it plainly.
- **Re-run the floor at FINAL state.** Three gates have fired unexpectedly across this session's
  strikes; two were caught only by re-running after the last edit.

## What would make me reject the result

- Row 2 answered with anything but a compiler error.
- Four call sites replaced and the field left public — that is the check rung sold as the shape rung.
- A `beta_mut()` escape hatch.
- A census number moved and explained away rather than surfaced.
- A red floor of any size.
