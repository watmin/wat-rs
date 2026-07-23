# BRIEF — send' OUTCOME WALL, Strike 1: the foundation (type + eval; kill both raises)

> **Tier:** sonnet shadowdancer. **Arc:** 278 — the send'-wall campaign (see
> `DESIGN-send-outcome-wall.md`). **HEAD:** `4543ef7a`. This is Phase 1 only: the `SendOutcome` type +
> the `send'` eval returning it. **Do NOT sweep the 183 call sites** (that's the Phase 2 codemod) and
> **do NOT add the checker force** (Phase 3). Foundation + a probe; leave uncommitted.

## Why (one paragraph)

`send'` raises reason-free `MalformedForm`s on a gone peer (`runtime.rs` `eval_peer_send_prime`:
`"peer already closed"` / `"send failed: channel disconnected"`). That raise masks every peer problem
and unwinds past the reader — the last raise-that-masks, the send-side of R53. This strike replaces the
raises with a matchable `SendOutcome` value (the recv' wall's twin), and proves it on a probe. The
183-site sweep + the checker force are later phases; this one just makes the eval return a value instead
of raising.

## Read in order

1. `docs/arc/2026/06/278-rules-engine/DESIGN-send-outcome-wall.md` — the campaign; the `SendOutcome` shape.
2. `src/types.rs:1168-1188` — the `RecvOutcome` builtin registration (the template to mirror).
3. `src/runtime.rs:25823` `eval_peer_send_prime` — the four tier arms (Thread' / Process' / Peer' thread /
   Peer' socket), each returning `nil` and raising on `None` (use-after-close) / `Err` (send failed). This
   is what you convert.
4. `src/runtime.rs:24808` `message_only_failure` — build the `Lost` cause with this (a Record; do NOT
   `struct-new` — the arc-278 wall now forbids it).

## Phase 1 — the two pieces

**(a) Register `:wat::kernel::SendOutcome`** as a builtin enum, mirroring `RecvOutcome` (`types.rs:1168`),
but **non-parametric** (no `<O>` — send' carries no payload):

```
:wat::kernel::SendOutcome  (Purity::Impure)
  :Sent   []
  :Closed []
  :Lost   [cause <- :wat::kernel::Failure]
```

**(b) Convert `eval_peer_send_prime`** (`runtime.rs:25823`), all four tier arms, to RETURN a `SendOutcome`
Value (build the aggregate the way other builtin enum values are built in runtime.rs — grep how
`RecvOutcome::{Message,Closed,Lost}` values are constructed for the exact idiom):
- success (`peer.send(...) → Ok`) → `SendOutcome::Sent`
- `None` (use-after-close) → `SendOutcome::Closed`
- `Err(_)` (send failed) → `SendOutcome::Lost` with cause = `message_only_failure("send': peer disconnected")`

**No `RuntimeError`/`MalformedForm` from the send path.** The fn now returns `Ok(SendOutcome value)`.

## Expected cascade (this is normal — do NOT try to fix it here)

Because `send'` now returns `SendOutcome` instead of `nil` and no longer raises, some existing tests will
flip RED — specifically any test that ASSERTED the `send'` raise, or that pattern-matched `send'`'s old
`nil`. **That RED is expected and is the Phase-2/3/4 worklist, not a Phase-1 bug.** Do NOT sweep sites or
add the checker force to make them green. Report which tests flipped (that list seeds the sweep).

## The probe (prove the raise → value flip)

Add a scratch probe `wat-scripts/scratch-pad/probe-send-outcome-wall.wat`: build a peer, close/drop its
far end, `(:wat::kernel::send' p m)`, and `match` the result — assert it is `SendOutcome::Closed` or
`::Lost` (a **value**), NOT a raise. `--check` it + run it. This is the disconfirming proof: before, this
raised; now it returns a matchable value. (If you cannot easily drop a peer's far end in wat, prove it via
the probe fixture the item-c test already exercises — a `send'` to a gone peer now returns a value.)

## STOP triggers

- **STOP-0:** you start writing the Phase-2 codemod or sweeping the 183 `(send' …)` sites, or adding the
  Phase-3 checker force — STOP. This strike is the type + the eval + the probe only.
- **STOP-1:** constructing a `SendOutcome` value in the eval isn't mechanical (the builtin-enum-value
  construction idiom differs from what you expected) — STOP, report how `RecvOutcome` values are actually
  built so the idiom is grounded.

## Verify (weigh by your own re-run)

1. `cargo build --release` compiles (the type + eval change).
2. The probe: `./target/release/wat --check` clean + running it shows a `SendOutcome::{Closed|Lost}`
   **value**, not a raise.
3. Run the floor: `cargo nextest run --release 2>&1 | tee /tmp/claude-scout/sendwall_p1_floor.log` — READ
   the Summary. Report the count + **the list of tests that flipped RED** (expected — the sweep worklist).
   Do NOT try to green them.

## Deliverable

The `SendOutcome` type + the converted `eval_peer_send_prime` + the probe. Report: (1) the type + the four
arms' final form; (2) the probe result (value, not raise); (3) the floor Summary + the RED-flip list.
Do NOT commit (the wall isn't complete — committing now would leave a swallow window). Leave in the tree.

## Blast radius

`src/types.rs` (the type) + `src/runtime.rs` (`eval_peer_send_prime`) + the scratch probe. NO site sweep,
NO checker force, NO commit. Scratch logs → `/tmp/claude-scout/`.
