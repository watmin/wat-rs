# DESIGN — the `send'` OUTCOME WALL (the send-side twin of R53's recv' wall)

> **Origin (2026-07-23):** chasing the self-scheduling item-c death, `send'` on a gone peer kept masking
> every peer problem behind a reason-free `MalformedForm` raise — `"send failed: channel disconnected"` /
> `"peer already closed"` (`runtime.rs:25861/25867`). The builder: *"we do not fear refactors — we fear
> ignorance, we annihilate ignorance."* This is the send-side completion of the no-hidden-failures LAW:
> `recv'` already faces its failures as a matchable value (R53 `RecvOutcome`); `send'` still RAISES — the
> last raise-that-masks. Annihilate it: `send'` returns a matchable `SendOutcome`, never raises, and the
> checker forces every caller to face it (no swallow).

## ✅ STATUS (2026-07-23) — THE WALL IS WHOLE. Phases 1+2 DONE+GREEN (raise annihilated); Phase 3 do-gate + 3a + 3b ALL DONE+GREEN (`53bdfb0a`, floor 4209/0). A discarded `send'`/`try-send'` outcome is a compile error in BOTH discard doors.

**Phases 1 + 2a + 2b complete — floor GREEN (4207/0, own `--release` re-run). `send'` returns
`:wat::kernel::SendOutcome` (never raises), and ALL 183 sites now FACE it (no `_`-swallow anywhere; no
swallow window). The last raise-that-masks is annihilated on the send side — committed as a green atomic
unit.** Phases: 1 = the type + eval (SendOutcome=Pure); 2a = the stdlib roots faced (66→19, the one-line
`test.wat` harness fix cleared the ~40 deftest tests, `service.wat` serve-replies → keep-serving); 2b = the
19 peer/wire test fixtures faced (19→0).

### ⚙ Phase 3 — the MUST-USE FORCE — do-gate + 3a + 3b ALL DONE + COMMITTED; THE WALL IS WHOLE (2026-07-23)

Makes a *discarded* outcome a **compile error** → swallow unrepresentable (R57 "unrepresentable > flagged").
There was **no** must-use mechanism in the checker (`recv'` is always expression-position); this built one.
**do-gate + Strike 3a are COMMITTED GREEN (`186ffb91`, floor 4208/0 by own `--release` re-run; RED probe
passes).** Four-questions RULED both:

- **do-gate — BUILT + working** (`src/check.rs`: `const MUST_USE_TYPES = [":wat::kernel::SendOutcome", …]`
  + a check in `infer_do` — a non-last expr whose type is must-use → located error; RED probe
  `probe_arc278_send_outcome_must_use_wall` PASSES). A *faced* send' types as `nil`, so the gate fires ONLY on
  a raw swallow — the floor staying green IS the proof of no swallows.
- **Strike 3a — `try-send'` → its own `TrySendOutcome` — DONE + COMMITTED (`186ffb91`, green).**
  Four-questions ruled A2 (own type) over adding `WouldBlock` to `SendOutcome` (re-breaks 183 matches; fails
  Obvious/Simple/Honest) and mapping to `Lost` (fails Honest). `try-send'` is NON-blocking → has an outcome
  `send'` cannot: **`WouldBlock`**. GROUNDED — it occurs on BOTH loci: thread = `crossbeam bounded(1)` slot
  full (`peer.rs`); process = pipe (~64KB) full → `O_NONBLOCK` `EAGAIN`/`EWOULDBLOCK` = live-peer-not-draining
  (`comms/process.rs:379-401`). The **process tier ALREADY returns `Result<(),TrySendError<T>>`** (Full vs
  Disconnected, `process.rs:402`); the THREAD tier collapsed it to a `bool` (`peer.rs:303`) — 3a brings the
  thread tier up to that honesty + unifies both under `TrySendError`. Built in the tree: `TrySendOutcome
  {Sent, WouldBlock, Closed, Lost[cause]}` (PURE); `peer.try_send`/`try_send_wire` enriched;
  `eval_peer_try_send_prime` returns it; `infer_try_send_prime` (stop reusing `infer_send_prime` at
  `check.rs:5112`); `TrySendOutcome` added to `MUST_USE_TYPES`; `wat/service.wat:1167` faced (all arms →
  evict + keep serving). `TrySendResult` made `pub` (the `private_interfaces` warning). **DONE:** floor
  weighed by own `--release` re-run = **4208/0**, RED probe passes, committed atomic (`186ffb91`). The
  ride-through doctrine held — the rider was reaped at the compaction, resumed via `SendMessage`, finished
  green in the field, banked ([[feedback_ride_through_compactions_with_shadowdancers_in_the_field]]).
- **Strike 3b — DONE + COMMITTED (`53bdfb0a`, floor 4209/0 own re-run; RED probe green).** The `let`-`_` gate
  (`process_let_binding`: `ident=="_"` && `is_must_use_type` → `push_must_use_error`, head `:wat::core::let`,
  reusing the do-gate helpers) + the swallow sweep. The sweep is a recorded wat-fix codemod
  (`face-underscore-bound-send-prime.wat`, idempotent, sha256-verified): walks let binding vectors, wraps every
  `_`-bound `send'` RHS in the `SendOutcome::{Sent,Closed,Lost}→nil` facing match (type → `nil` → gate passes).
  Faced **EVERY** `_`-bound `send'` swallow — **50 files** across `tests/`+`wat-scripts/`, NOT the ~19 the first
  grep undercounted (single-space regex missed alignment-padded bindings; a line-grep can't see the AST — the
  codemod was dry-run over the whole 1208-file corpus and the diff WAS the complete worklist: `wat/`+`wat-tests/`
  confirmed clean, 0 non-facing edits). RED probe: `probe_arc278_send_outcome_must_use_wall_let.wat.bad`. **THE
  WALL IS WHOLE** — a discarded outcome is a compile error in BOTH discard doors (`do`-non-final ✓, `let`-`_` ✓).
- **Companion (tracked):** the arc-277 raise-abuse rete-lint (discovery) + the raise-abuse audit of the other
  peer/IO verbs (`connect'`/`accept'`/`poll'`/`close'`).

The historical build record of Phases 1-2:

**Phase 1 (foundation) — done and proven sound (the build record):**
- `:wat::kernel::SendOutcome` registered **PURE** (`src/types.rs:1205`) — non-parametric, holds only nullary
  variants + a pure `Failure` record. NOT `Impure` (that was a copy-error from `RecvOutcome`, whose `Impure`
  is `O`-driven — its payload may be a live resource; `SendOutcome` has no payload → pure/EDN-crossable).
- `eval_peer_send_prime` (`src/runtime.rs:25823`), all four tier arms, RETURN `SendOutcome::{Sent, Closed,
  Lost[cause=message-only-failure]}` — **no raise.** Probe `wat-scripts/scratch-pad/probe-send-outcome-wall.wat`
  `--check`s clean.
- **Weighed by own `--release` re-run: 4141 passed / 66 failed / 0 RIPPLE** — every RED is a *transitive*
  send' user (confirmed: `int_modrem`/`call_site` fail on the harness path, not the arithmetic). RED-flip
  list captured at `/tmp/claude-scout/sendwall_p1_weigh.txt`.

**The 66 RED reduce to a FEW ROOT SITES — the sweep worklist (do the roots first, they clear most):**
1. **The deftest harness** — `run-thread'`/`run-hermetic'` (`wat/test.wat` / `wat/kernel/hermetic.wat`)
   `send'` their child → ~40 `deftest_wat_tests_*` + the `call_beside`-driven probes (incl. `int_modrem`,
   `call_site`) fail together. **Fix this root first — it clears the bulk.**
2. **The peer round-trip helpers** — arc-259 / arc-214 / arc-293 wire tests + their fixtures.
3. The remaining scattered `send'` sites (the codemod sweep covers all 183).

**Phase 2 (NEXT — resume here):** the wat-fix codemod faces `SendOutcome` at every `(send' …)`; start with the
two roots (they clear ~55 of the 66). Then Phase 3 (checker force → unfaced `SendOutcome` = compile error),
Phase 4 (per-site refinement), atomic green. Do NOT commit the code until the wall is green.

---

## The type

```clojure
;; runtime builtin, mirrors :wat::kernel::RecvOutcome (types.rs:1168) in SHAPE — but PURE, not Impure.
;; RecvOutcome<O> is Impure ONLY because of its payload O (a received message may be a live resource);
;; SendOutcome is NON-parametric and holds only pure data (two nullary variants + a pure Failure record,
;; Nature::Record) — fully EDN-reconstructable / wire-crossable. Marking it Impure would LIE.
(:wat::core::defenum :wat::kernel::SendOutcome :wat::enum::Pure
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
