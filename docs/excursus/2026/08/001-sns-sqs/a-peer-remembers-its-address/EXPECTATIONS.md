# EXPECTATIONS — a peer remembers its address

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`cdfc245ab`): floor **5247**/5247, 0 FAIL, **535.808 s** (orchestrator's own, quiet box).
clippy 0.

⚠ **The executor's floor wall clock is not comparable to this baseline.** Three stones running:
executor +29.2 / +29.6 / +18.0 s where the orchestrator measured +7.5 / +0.96 / +0.24 s on the same
trees. Report the number; the orchestrator's run is the one that grades row 8.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭐ **a dialed peer remembers** | dial a service, read `dialed-from` | `Some addr` |
| 2 | ⭑⭑ **the round trip** | `connect` that address; use the fresh peer | it serves — a real second connection |
| 3 | ⛔ **the observation steals nothing** | use the ORIGINAL peer after reading | it still serves. `dialed-from` is a peek |
| 4 | ⛔⛔ **an ACCEPTED peer reports `None`** (STOP-1) | server side, from `accept` | `None`. A `Some` here is a painted brick and the stone fails |
| 5 | ⛔ **a thread-tier peer reports `None`** | `from_thread` | `None` — no address exists |
| 6 | **the self-peer and the dead sentinel report `None`** | read the diff | `None` passed *deliberately* at each, not defaulted |
| 7 | ⚠ **`Peer` is still not a wire type** | assert, don't assume | unchanged. An `Address` is portable; a `Peer` must not become so |
| 8 | floor | `./scripts/floor.sh` → **Summary** | `5247 passed` or higher, 0 FAIL. A SHRINK is a finding |
| 9 | clippy | `cargo clippy --all-targets -D warnings` | 0 |
| 10 | tests compile | `cargo nextest run --release --no-run` | clean |
| 11 | ⛔ **`.wat` diff is ZERO** | `git status -- wat/ wat-scripts/` | empty. Nothing calls this yet — stones 2 and 3 do |
| 12 | scope stated | the SCORE's headline | a peer REMEMBERS. No behaviour changed, no deadline repaired, no publisher touched |

## Runtime prediction

**60–90 minutes.** One field, five constructor decisions, one intrinsic, one inference function with a
worked precedent (`infer_recv_prime`), one probe. The cost is the five sites and the type threading.

## Trap-door risks, ranked

1. ⭑⭑ **Row 4 inverted.** The single way this stone does harm is an accepted peer reporting a name the
   caller then dials. The remote bound no listener; the dial fails or, worse, reaches something else.
   The whole value of the stone is that `None` is honest there.
2. **A defaulted constructor parameter.** If a site can silently forget to decide, the dialer stops
   remembering in six months — which is precisely the defect being repaired one layer down.
3. **Row 3.** A peek that consumes would break every existing peer user, loudly. Cheap to get right,
   catastrophic to get wrong.
4. **Row 11 non-zero.** Touching `call-by-deadline` here would bundle stone 2 and make neither
   measurable on its own.
