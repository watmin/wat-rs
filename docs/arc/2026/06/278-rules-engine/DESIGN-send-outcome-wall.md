# DESIGN — the `send'` OUTCOME WALL (the send-side twin of R53's recv' wall)

> **Origin (2026-07-23):** chasing the self-scheduling item-c death, `send'` on a gone peer kept masking
> every peer problem behind a reason-free `MalformedForm` raise — `"send failed: channel disconnected"` /
> `"peer already closed"` (`runtime.rs:25861/25867`). The builder: *"we do not fear refactors — we fear
> ignorance, we annihilate ignorance."* This is the send-side completion of the no-hidden-failures LAW:
> `recv'` already faces its failures as a matchable value (R53 `RecvOutcome`); `send'` still RAISES — the
> last raise-that-masks. Annihilate it: `send'` returns a matchable `SendOutcome`, never raises, and the
> checker forces every caller to face it (no swallow).

## The type

```clojure
;; runtime builtin, mirrors :wat::kernel::RecvOutcome (types.rs:1168). Impure (an I/O outcome).
;; NON-parametric — send' carries no received payload, so no type-param (unlike RecvOutcome<O>).
(:wat::core::defenum :wat::kernel::SendOutcome :wat::enum::Impure
  :Sent   []                                   ;; delivered
  :Closed []                                   ;; peer already cleanly closed (the None / "peer already closed" case)
  :Lost   [cause <- :wat::kernel::Failure])    ;; disconnected mid-send (the send-Err / "channel disconnected" case)
```

`Sent` replaces recv's `Message[msg]` (no payload on the send side). `Closed`/`Lost` mirror recv' exactly.
The `Lost` cause is a `message-only-failure` — **`send'` structurally cannot know WHY the peer died** (the
crash reason is on the owner peer's crash channel, faced via the recv' wall). `send'` says *THAT* the peer
is gone; the owner's `recv'` says *WHY*. Honest, not fabricated. (Do NOT invent a fake detailed cause.)

## The eval change (`eval_peer_send_prime`, `runtime.rs:25823`)

Four tier arms (Thread' / Process' / Peer' thread / Peer' socket). In each:
- success `peer.send(...) → Ok` → return `SendOutcome::Sent`.
- `None` (use-after-close) → `SendOutcome::Closed` (was the "peer already closed" raise).
- `Err(_)` (send failed) → `SendOutcome::Lost[message-only-failure("send': peer disconnected")]` (was the
  "channel disconnected" raise).

**No more `RuntimeError`/`MalformedForm` raise from the send path.** Return a `SendOutcome` value.

## The sweep (183 sites / 69 `.wat` files) — a wat-fix codemod

`send'` returns `Unit` today; the 183 sites do `(send' p m)` and drop the result (fire). Once `send'`
returns `SendOutcome`, dropping it is a **swallow** (worse than the raise). So the checker must FORCE the
match (Phase 3), and the codemod (Phase 2) must give every site an explicit facing:

```clojure
;; the default codemod rewrite — face all three arms explicitly (LAW-compliant: visible + chosen, not swallowed):
(:wat::core::match (:wat::kernel::send' p m)
  ((:wat::kernel::SendOutcome::Sent)     <continue>)
  ((:wat::kernel::SendOutcome::Closed)   <default>)     ;; per-site refine: serve-reply → continue; req/reply → recv' faces it
  ((:wat::kernel::SendOutcome::Lost _c)  <default>))
```

The DEFAULT (`Closed`/`Lost` → the same continuation as `Sent`, or a located `assertion-failed!` where a
gone peer is genuinely fatal) is **explicitly faced**, not swallowed — the mask is gone. Per-site review
then refines the two failure arms where "continue" is wrong (a serve-reply to a gone client → keep serving;
a request whose reply the caller `recv'`s → the recv' wall faces the death, so send'-Lost → proceed).
**This is per-site judgment for many of the 183 — arc-scale, like the recv' cascade. Not a blind one-shot.**

## The checker force (Phase 3)

`send'`'s return type is `SendOutcome`; a bare `(send' p m)` in statement position with an unhandled
`SendOutcome` is a compile error (exhaustiveness / must-face). Mirrors how `recv'`'s `RecvOutcome` is
forced. This is what makes swallow **unrepresentable**.

## Atomic landing (the STASH-DANCE)

The eval change + the type + the sweep + the checker force must land TOGETHER — there is no green state
where `send'` returns `SendOutcome` but sites still drop it (that's a swallow window). Follow `wat/fix.wat`'s
STASH-DANCE: build the pieces, stash the checker-force, run the codemod under the old checker, unstash,
land atomically. The floor green is the completion proof.

## Phases (driven as a campaign — the recv' wall's proven playbook)

1. **Foundation** — register `SendOutcome`; convert `eval_peer_send_prime`'s four arms to return it (kill
   both raises); a probe proving `send'` to a dead peer returns `SendOutcome::Lost`, not a raise. *(Provable
   in isolation; the sweep/force follow before commit.)*
2. **The codemod** — `wat-scripts/fixes/send-prime-to-outcome-match.wat`: wrap every `(send' …)` in the
   default facing. Dry-run + diff on a `/tmp` copy; apply to the 183 sites.
3. **The checker force** — make an unhandled `SendOutcome` a compile error.
4. **Per-site refinement** — walk the cascade the force lights; give each failure arm its right behavior
   (serve-reply, req/reply, fire). Weigh the floor green, atomic.

## Boundary

- `send'`'s `Lost` cause is `message-only` (honest — the WHY is the owner's recv' wall; item 4). This
  campaign does NOT also build "the owner reads the crash channel" — that's a separate, smaller follow
  (the fixture/doctrine reading the owner peer `svc` from the Handle). Tracked, not folded in.
- The self-scheduling **item-c** idx-shift (`service.wat:958/961`) is a SEPARATE near-one-liner (the client
  peer wrongly evicted by `remove-at`), landable independently; the wall makes its failure legible.
