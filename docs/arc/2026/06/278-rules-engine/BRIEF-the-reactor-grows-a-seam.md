# BRIEF — the reactor grows a seam

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`08bd0c941`, tree clean. Read `DESIGN-the-reactor-grows-a-seam.md` first.

## THE WORK

`wat/service.wat`'s template sends a reply at ten sites with the same four-arm `SendOutcome` match.
Extract it to one parametric top-level `defn` and call it from those sites. **No behaviour change.**
The floor — 5214 tests, all of which expand through this macro — is the proof.

## ⛔ YOUR FIRST ACT — the probe I did not write

**Can a parametric top-level `defn` take a peer and a protocol-reply payload and be called from the
generated code?** The payload type is per-service (`~proto-reply-ty-ann`). `:- [R]` parameterisation
exists, so it should be expressible — **but I did not probe it.**

Write the ten-line probe. If it will not express, **STOP-1**: report the exact checker error and do
not improvise a different seam.

## ROOMS — read in this order

1. **`wat/fix.wat`** — the **BOOTSTRAP / STASH-DANCE** header. `wat/service.wat` is stdlib, frozen
   into the binary at build time. This is the first stone this campaign that needs it.
2. **`wat/service.wat:1657-1664`** — shape A, the `ReplyTo` path: `send peer (Directed/reply d)`.
3. **`wat/service.wat:1782-1789`** — shape B, direct reply:
   `send (second (nth selectables idx)) resp`. **Identical four arms.**
4. **All ten:** `:1659 :1697 :1784 :1811 :1828 :1854 :1939 :1950 :2006 :2012`. The last two are
   `Stopped`/`Hibernated` status sends — **check whether they share the shape; if not, leave them.**
5. **`wat/service.wat:64`** — *"A vanished waiter (absent conn-id, or send Closed/Lost) is not an
   error — keep serving."* That sentence is the `Closed → true` arm. **Do not change it.**
6. **`wat/service.wat:67-95`** — the eight sibling top-level forms your `defn` joins.

## STOP TRIGGERS

1. **The parametric helper will not express.** Report the checker error verbatim. Do not improvise.
2. **You are about to change any of the four arm dispositions.** `Sent`/`Closed` → true,
   `Stopped` → false, `Lost` → true. They are the contract. STOP.
3. **You are about to add the drop.** Next stone. STOP.
4. **The floor moves off 5214/5214.** Every test expands through this macro; a red is the extraction
   being unfaithful. Capture whole, name the arm, do not re-run.
5. **You are about to touch `src/`.** This is `wat/service.wat` only. STOP.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim.

⚠ **Do not write `(:wat::core::None <Type>)`** — phantom form, type-checks and raises. Bare
`:wat::core::None`. See `docs/arc/2026/04/109-kill-std/NOTE-none-is-not-a-function.md`.

⚠ S24 is live: `refused_subscriber_is_retried_not_dropped` can fail loudly with `after-drain=got`.

Leave your work uncommitted. Prior comparable result: `SCORE-the-vocabulary-stops-mumbling.md`.

## REPORT

- **the probe from your first act**, and whether the parametric form expressed
- the floor Summary line verbatim
- `grep -n 'kernel::send' wat/service.wat` after — the call should survive only in the helper, plus
  any site you correctly left alone, named
- the circuit: five runs, `total`/`distinct`/`dup`
- **whether the seam can carry a rate-gated drop as-is**, or what widening it needs. Say; do not build
- every STOP that fired
- **the honest deltas.** My citations drift and my censuses have been wrong seven times this
  campaign — including, in this stone's own DESIGN, calling this file "one top-level form" when it
  has nine.
