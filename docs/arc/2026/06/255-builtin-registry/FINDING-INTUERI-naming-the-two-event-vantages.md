# FINDING — INTUERI names the two `ServiceEvent` vantages (E2)

**Cast 2026-09-25**, read-only, at HEAD `aff88844a`. The spell was embedded verbatim. The builder ruled E2
(two event types, picked by element family) with four YES, and *"let intueri name them"*. The orchestrator
verified the two non-naming findings on disk (marked ✓).

## The names (every survivor is four YES)

| thing | survivor | rejected (the NO) |
|---|---|---|
| the peer-vantage event type (`poll`; `select` over peers) | **`PeerEvent :- [I O A]`** | `ServiceEvent` (9 of 23 peer call sites are timers, not services), `PeerPollEvent` (`select` returns it too), `ServeEvent` (a timer nap is not serving) |
| the owner-vantage event type (`select` over `Spawned`) | **`OwnerEvent :- [R]`** | `SupervisorEvent` (bracket's collect-loop is a pool collector), `SpawnedEvent`/`ChildEvent` (they read as the child's own vantage) |
| the owner dropped its handle | **`OwnerClosed`** | `Shutdown` (3 facts today), `OwnerGone` (a second word for Closed), `Orphaned` (the drop is the designed shutdown) |
| a stop, as a value | **`Stopped`**: *honest only if a producer exists* (below) | — |
| a peer was admitted | **`Accepted [peer]`** (= `AcceptOutcome.Accepted`) | `Connection` (a bare noun, and one fact with two names), `Connected` (the dialer's fact) |
| an over-budget frame | **`Oversized [idx cause]`** | `Rejected` (it names the loop's verdict, and collides with `PeerRecvOutcome.Rejected`) |
| the reasonless crash notice (peer) | **`Crashed [idx]`** (= `PeerRecvOutcome.Crashed`) | `Died [idx]` (the owner's name with the reason removed) |
| the reasoned death (owner) | **`Died [idx cause <- LociDiedError]`** (= `OwnerRecvOutcome.Died`) | `Lost` (it folds oversized, malformed and io into a death), `Crashed [cause]` (a collision, and `LociDiedError` also carries non-crashes) |

Unchanged: `Message`, `Closed`, `Admin`, `Malformed`, `Failed`. **`Oversized` meets the prior cast's condition
for `PeerRecvOutcome.Rejected`.**

## Fact sets, checked against the producing arms (`src/kernel/message.rs`)

- **`PeerEvent`:** `Admin` · `OwnerClosed` · `Stopped` (see ⚠) · `Accepted` · `Message` · `Closed` · `Crashed`
  · `Malformed` · `Oversized` · `Failed`. **No `Died`, and no `LociDiedError`**: the supervisor ruling.
  The serve loop's current `Lost` arm (`service.wat` ~:1973) already reaps exactly like `Closed`, so the two
  merge.
- **`OwnerEvent`:** `Message` · `Closed` · `Died [LociDiedError]` (today `Lost` with a stringified cause, a type
  change) · `Stopped` (today `Shutdown` from `PeerDeath::Shutdown`) · `Oversized`/`Malformed`/`Failed`
  (process only; today inside `Lost`). **No `Admin`, `Accepted`, `OwnerClosed` or `Crashed`.**

## ⛔ Defects found, with the verified ones marked ✓

- ✓ **The re-poll drops an admin message** (`message.rs:2019-2025`): `if idx2.0 == 0` builds `Shutdown`
  **without reading `res2`**. An `AllowPeer` arriving on that path is dropped, the service exits, and the
  owner waiting on its ack sees `Closed`. **A real bug.**
- ✓ **The remedy picks by string** (`check.rs:8222`): `ty_name.contains("ServiceEvent")`. Neither new name
  contains it, so both would fall to the `send` remedy. Its arm list is already stale.
- ⚠ **A stop in `poll` is a RAISE, not a value** (`:1592/:1755/:2003`; peer `select` `:1214/:1333`). The value
  `Shutdown` meaning "a stop" exists only on owner `select`. **`PeerEvent.Stopped` has no producer unless
  those raises become values.** (Recv/Send already report a stop as a value.) Otherwise the variant is an
  unbuilt promise.
- The raw peer `select` delivers the `PeerCrashed` sentinel as `Message` (`:1196/:1314`). `Crashed` needs a
  producer there. No corpus site selects over client-side peers today.
- `Closed` wildcards absorb `Malformed`, `FrameTooLarge` and io (`process.rs:1907` turns io into
  `Disconnected`). `Failed` needs `select_raw` to carry `RecvError::Failed`.
- The `Oversized` cause text (`:1877/:2080`) writes the serve loop's disposition ("connection closed")
  into a fact that `select` consumers also read.
- The owner timer arm (`:1036`) is dead: `ProcessSelectable::Timer` is never constructed (a job for purgare).
- A latent collision: if `poll`'s listener-gone and accept-failure raises become values, they need
  `ListenerClosed`/`AcceptFailed`, not the per-peer `Closed`/`Failed`.
- The carried collision `OwnerEvent.Closed` (the child left) vs `CloseOutcome.Closed [exit]` (I reaped it).
- **Gates the rename can break:**
  - `MUST_USE_PARAMETRIC_HEADS` (`check.rs:8190`) must list both heads, and **no peer-vantage must-use fixture
    exists**;
  - exact-string tests (`probe_arc214_stone46b_select_prime.rs:97`, `probe_supervisor_select_lost.rs:166/171`,
    `probe_arc255_32_lost_arm_expansion.rs:31`).

## Migration size (a heuristic census)

- **151 arms in 19 files:** 143 peer vantage, 8 owner vantage (bracket only).
- **Call sites:** `poll` 14 (the generator `service.wat:1882` covers every service), `select` over peers 9
  (timers), `select` over owner handles 6.
- **Rust:** 150 lines in 20 files.
- The checker can already pick the family (`owner_or_peer_handle`, `check.rs:11230-11237`), so C1 is gone.
