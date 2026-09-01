# EXPECTATIONS — the deferred reply

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ client wakes client | the gate service: A parks, B wakes | A's blocked `recv'` returns B's value. **RED today** |
| 2 | ★ **a TIMER wakes a client** | an internal arm returning `ReplyTo` | it works. STOP-2's target — if this fails the stone is inert and rows 3–10 still pass |
| 3 | one call wakes several | two parked, one wake with two `Directed` | both return. The vector earning its shape |
| 4 | a vanished waiter is survivable | park, drop the client, wake | the service keeps serving; no raise, no desync |
| 5 | no `Peer` reaches an arm | `git diff wat/service.wat` | arms still take `[s ctx req]` / `[s ctx]`; `conn-id` only (STOP-1) |
| 6 | existing outcomes unmoved | the floor + every service gate | `Reply`/`NoReply`/`ReplyAndArm`/`NoReplyAndArm` identical (STOP-4) |
| 7 | the internal assertion still catches `Reply` | an internal arm returning `Reply` | still the located assertion — `ReplyTo` is the exemption, not a hole |
| 8 | no runtime change | `git diff --stat src/` | empty |
| 9 | no existing service edited | `git diff wat/` minus `service.wat` | empty |
| 10 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, FLOOR=0 |

**Runtime prediction:** 60–120 minutes. The enum and resolution are small; the two-client gate is the
work, and it must not use a sleep — park/wake is wire-ordered by construction.

## Trap doors, named in advance

- **The internal arm still rejecting `ReplyTo`.** Rows 1, 3–10 all pass; only row 2 catches it, and
  long polling is impossible without it. **This is the failure mode of this stone.**
- **Passing a peer to the arm** to make resolution easy. Row 5, and STOP-1.
- **Raising on an absent conn-id.** A long-polling client giving up is normal, not exceptional.
- **A sleep in the gate.** Park/wake is ordered by the wire — A blocks in `recv'` until B's call
  causes the send. If a gate needs a sleep, the mechanism is not doing what it claims.
- **Firing on nothing:** a `ReplyTo` that compiles and resolves nothing passes rows 4–10.
