# DESIGN — both ends bound the same handshake

Builder: *"let's get symmetry, then."*

**Drawn 2026-09-15. NOT STRUCK.** Executor: a spawned **Opus** subagent (grok's credits exhausted).

## WHY — one end of a handshake was bounded and the other was not

`a-wait-that-should-be-bounded` (`07e481f82`) classified all 23 live bare-`recv` sites and produced a
**four-site** defect list — one shape, both ends of a spawn handshake:

```
wat/spawn.wat:531   ThreadOpts  launch  — "Crash-aware readiness barrier"
wat/spawn.wat:588   ProcessOpts launch  — awaits the child-minted launch status
wat/test.wat:329    :wat::test::spawn-thread-program
wat/test.wat:438    :wat::test::spawn-hermetic-program
```

⭐ **`child-main` is the CHILD end of that same wire and was bounded at `7686bea24`. `launch` is the
PARENT end and was not.** So a child that neither crashes nor announces readiness hangs its launcher —
and an unbounded wait **cannot be stopped by SIGTERM** (measured this session: 125 s ignored, `kill -9`
required).

⭑ All four already carry the unreachable `RecvOutcome::TimedOut → assertion-failed! "the peer is alive
and silent"` arm. **The code names the failure it cannot observe.** Bounding the wait makes the arm true.

## ⛔ THE ONE CONTRACT DECISION

**ONE constant, in `wat/spawn.wat`, named by all four sites AND by `child-main`.**

⛔ **Load order forces the home, and for once that is a gift:**

```
171  wat/spawn.wat      ← launch (the parent end)
268  wat/test.wat       ← the two harness sites
341  wat/service.wat    ← where :wat::service::CHILD-MAIN-STARTUP-DEADLINE-MS lives TODAY
```

A stdlib file cannot name a type or value declared after it, so `spawn.wat` (171) **cannot** see a
constant defined in `service.wat` (341). Therefore the constant moves to `spawn.wat`, where **every** end
can name it — and that is not a workaround, it is the correct home: **the number describes the spawn
handshake, not the service macro.**

⭐⭐ **And it makes the symmetry structural rather than coincidental.** Two constants with the same value
can drift apart in a later edit; **one constant cannot.** That is the difference between "both ends happen
to wait 30 s" and "both ends bound the same handshake."

⚠ `:wat::service::CHILD-MAIN-STARTUP-DEADLINE-MS` was introduced **three hours ago** (`7686bea24`) and is
removed here. It was in the wrong home because the parent end had not been examined yet — say so, do not
leave two constants.

### The number, and why all four share it

**30000 ms**, unchanged, and its justification carries over: the owner ships immediately after spawn, a
healthy handshake is milliseconds, and 30 s sits inside the window before an operator reaches for
`kill -9`. ⚠ **The two `test.wat` sites share it deliberately** — all four are *"something was just spawned
and must announce itself."* If a hermetic test program legitimately needs longer, **that is a finding to
report, not a second constant to guess at.**

## Out of scope = REJECTED

- **The 11 TIMER and 7 PARK sites.** Classified as non-defects, with evidence. Bounding a timer would put
  a timeout on a timeout; bounding a park would undo `queue-long-poll`.
- **The UNKNOWN** (`wat/spawn.wat:619` `recv-all-loop`, a drain-to-EOF). Bounding it would change its
  **return contract**, not merely add a deadline. Its own stone, if any.
- **`send` (215), `readln` (161), `accept` (8), `poll` (14 UNKNOWN).** Same invariant, different
  primitives, and each needs its own intent classification first — the lesson this classification taught.
- **Changing what happens on timeout.** The existing arm raises. Supervision stays deferred.

## Blast radius

```
wat/spawn.wat     +1 constant; 2 recv sites bounded          ⛔ STDLIB, manifest position 171
wat/test.wat      2 recv sites bounded                       ⛔ STDLIB, position 268
wat/service.wat   child-main names the new constant; the 3-hour-old local one is REMOVED
```

## Trap-doors named up front

1. ⛔⛔ **`wat/spawn.wat` is manifest position 171 — nearly everything loads after it.** A mistake here
   breaks the whole stdlib, and it is frozen into the binary at build time. Read `wat/fix.wat`'s
   BOOTSTRAP/STASH-DANCE header first.
2. ⛔ **No arm changes should be needed** — all four sites already have a `TimedOut` arm, and
   `recv-by-deadline` returns the same `RecvOutcome` as `recv` (measured from the enum in
   `a-peer-wait-can-be-bounded-too`, where **both `@ret` doc comments turned out to be wrong**). If an arm
   must change, STOP and report.
3. ⚠ **Arity is 2** — `(recv-by-deadline peer ms)`. The timer payload is a kernel-internal sentinel
   discriminated **by idx, never by value**, so no `inert` argument exists to supply.
4. **Verify no other reference to the removed constant** before deleting it.
5. **`cargo build --release` does not compile tests.** And ⛔ use `timeout -k` on anything that may block.
