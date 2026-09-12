# EXPECTATIONS — the gate methods face an outcome

**Written before the strike.** Graded against the orchestrator's OWN reads and runs, never the report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | `GateOutcome` exists with three honest variants | `grep -o 'GateOutcome::[A-Za-z]*' wat/service.wat \| sort -u` | exactly `Applied`, `Gone`, `GaveUp` |
| 2 | grant's four raising arms are gone | read `grant-method-body` | only the `Message(other)` protocol-violation raise remains |
| 3 | **revoke got the same change** | read `revoke-method-body` | same three-way mapping; no four raising arms |
| 4 | the thread arm is a no-wait success | read the `(None)` arm | returns `Applied`, not `nil`, and does not enter the loop |
| 5 | ⛔ NO re-send of `AllowPeer`/`DenyPeer` | read the loop | `send` appears **once** per call; the loop only `recv`s |
| 6 | the bound is not duplicated | read | `owner-recv-loop` reused or generalized — not a second copy of the wall-clock logic |
| 7 | `Gone` only for a gone peer | read the arms | `Closed`/`Lost` → `Gone`; `TimedOut`/`Stopped`/`Malformed` → re-recv or `GaveUp`, never `Gone` |
| 8 | floor | `./scripts/floor.sh` → **Summary line** | `5237 tests run: 5237 passed`, 22 skipped, **0 FAIL** |
| 9 | tests compile | `cargo nextest run --release --no-run` | exit 0, no `^error` |
| 10 | happy path — and it exercises the grant ordering | `… 2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 11 | chaos | `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` | exit 0, `distinct=100;dup=0` |
| 12 | the outcome is trippable | hand-construct `GaveUp`, pass to the require-helper | RAISES, message names `waited-ms` and `last` |
| 13 | census honest | `grep -oE '\(:[a-z0-9:_-]+/(grant\|revoke) ' -r --include=*.wat . \| wc -l` | 72 total (68 + 4); `.rs`/`.jsonl` 0; SCORE says so if it differs |
| 14 | clippy stays at 0 | `cargo clippy --release --workspace --all-targets -- -D warnings` | **exit 0** — it was brought to 0 at `551ed1a41` and must not regress |

## Runtime prediction

**60–100 minutes.** Two generated bodies, one enum, 72 call sites, and the floor is ~500 s and will
likely run twice.

## What makes this PARTIAL rather than passed

- Row 3 red → `grant` fixed and `revoke` left: the sixth sibling-pair of the day, and the one the
  DESIGN named in advance.
- Row 5 red → a re-send crept in. The contract decision exists precisely because re-ask *looks* right
  here; a strike that switches without arguing has skipped the decision.
- Row 12 red → an outcome nobody can trip. Both previous stones needed this check and one of them
  (`StopOutcome::GaveUp`) came back only partly reachable.
- Row 10 red specifically → the post-spawn grant ordering broke; that doc-comment calls it load-bearing
  for a reason (the builder's circuit dials after granting).

## Trap-doors named before the strike

1. **`AllowPeer` IS re-askable** (idempotent set-add; the service recurs) — so re-ask looks correct and
   is rejected on the surplus-ack/desync argument, not on symmetry with `stop`. STOP-1.
2. **The thread tier has no round-trip.** STOP-4.
3. **`revoke` is the twin.** STOP-2.
4. **Three carriers.** `.rs`/`.jsonl` are 0 here — verified, not assumed. STOP-3.
5. **`cargo build --release` does not compile tests.**
6. ⚠ **`GaveUp` inherits a known limitation and the SCORE must not claim otherwise:** a bare `recv`
   never returns `TimedOut` (tracked at
   `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md`), so the
   bound holds between returning outcomes and **not against a silent peer**. Same as stone 2. Do not
   claim "cannot hang".
