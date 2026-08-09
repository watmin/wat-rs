;; Arc 278 — THE ACCEPTANCE GATE for the call context: an OPT-IN third arm parameter
;; `[s ctx req]`, carrying a five-field pure context, with a stable monotonic caller id minted
;; in the generated serve loop. See docs/arc/2026/06/278-rules-engine/BRIEF-the-call-context.md
;; and DESIGN-STONE-the-call-context.md.
;;
;; Modelled on tests/services/probe_arc278_per_op_request_too_large.{rs,wat} (connect/round-trip
;; shape) and tests/types/probe_arc278_opaque_purity_wall.* (the acceptance-gate framing).
;;
;; ⚠ `:wat::service::CallCtx` is a PLACEHOLDER type name (STOP-5, DESIGN-STONE-the-call-context.md):
;; an intueri cast is OWED. Do not read this identifier as ratified.
;;
;; Three things, and the third is the one that matters most (the brief, verbatim):
;;   1. A 3-param arm receives a populated ctx — namespace/operation/caller-id.
;;   2. A 2-param arm in the SAME service still works — proving opt-in, not migration.
;;   3. ★ THE STABILITY GATE — the id survives an eviction: connect three clients, disconnect
;;      the MIDDLE one, then have a SURVIVOR call an op and assert it still sees its ORIGINAL id.
;;      A position-keyed implementation passes every other test and fails only this one.

;; arc 278 #74 — `<Op>Response` is LAW: a serviceable op's response type MUST be named
;; `<VariantPascal>Response` exactly (checker-enforced at `defsurface` registration), so
;; `whoami`'s response is `WhoamiResponse` itself (no extra wrapper record) and `ping`'s is
;; `PingResponse`.
(:wat::core::defsurface :probe::CallCtx3 :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::CallCtx3::WhoamiRequest [])
   (:wat::core::defenum :probe::CallCtx3::WhoamiResponse :wat::enum::Pure
     :Ok               [caller-id <- :wat::core::i64  namespace <- :wat::core::keyword  operation <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::CallCtx3::PingRequest [])
   (:wat::core::defenum :probe::CallCtx3::PingResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(whoami [self <- :probe::CallCtx3  req <- :probe::CallCtx3::WhoamiRequest] -> :probe::CallCtx3::WhoamiResponse :max-request-bytes 524288)
   (ping   [self <- :probe::CallCtx3  req <- :probe::CallCtx3::PingRequest]   -> :probe::CallCtx3::PingResponse   :max-request-bytes 524288)])

;; The satisfier — `whoami` is the NEW 3-param `[s ctx req]` arm (opt-in ctx); `ping` stays the
;; ORDINARY 2-param `[s req]` arm, UNTOUCHED shape, in the SAME service (test 2's proof).
(:wat::service::defservice :probe::callctx3svc
  :satisfies :probe::CallCtx3
  :durable   []
  :ephemeral []
  :impls
  [(whoami [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::CallCtx3::WhoamiResponse::Ok
         (:wat::service::CallCtx/caller-id ctx)
         (:wat::service::CallCtx/namespace ctx)
         (:wat::service::CallCtx/operation ctx))))
   (ping [s req]
     (:wat::service::Outcome::Reply s (:probe::CallCtx3::PingResponse::Ok true)))])

;; ── helpers shared by every driver below ──────────────────────────────────────────────────
(:wat::core::defn :probe::connect! [h <- :probe::callctx3svc::Handle] -> :probe::CallCtx3
  (:wat::core::match (:wat::kernel::connect (:probe::callctx3svc::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; Round-trips `whoami` on `c` and returns the caller-id (asserting on transport failures — a
;; probe driver, not the subject under test).
(:wat::core::defn :probe::whoami-id [c <- :probe::CallCtx3] -> :wat::core::i64
  (:wat::core::match (:probe::CallCtx3/whoami c (:probe::CallCtx3::WhoamiRequest))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:probe::CallCtx3::WhoamiResponse::Ok caller-id _namespace _operation) caller-id)
        ((:probe::CallCtx3::WhoamiResponse::RequestTooLarge _b _c) (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:probe::CallCtx3::WhoamiResponse::RequestMalformed _p _e _g) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "whoami: stop before reply" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "whoami: closed before reply" :wat::core::None :wat::core::None))))

;; A bounded, event-driven wait (NOT a sleep-guess — arc 278's own precedent, DESIGN-STONE-the-
;; call-context.md's reproduction: "wait 60ms via a select'-on-after nap"): arm a one-shot timer
;; and block on its OWN recv. Gives the server's single-threaded serve loop room to have already
;; processed a dropped client's Closed event by the time the caller proceeds.
(:wat::core::defn :probe::nap! [] -> :wat::core::nil
  (:wat::core::let
    [t (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Millisecond 60) :tick)]
    (:wat::core::match (:wat::kernel::recv t)
      ((:wat::kernel::RecvOutcome::Message _tick) nil)
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped nil)
      (:wat::kernel::RecvOutcome::Closed nil))))

;; ── (1) a 3-param arm receives a POPULATED ctx ────────────────────────────────────────────
;; Returns a Tuple(caller-id, operation) — the harness checks caller-id >= 0 (present) and
;; operation == "whoami" (the op's own kebab name, spliced as a compile-time literal). namespace
;; is checked separately (below) since it is a KEYWORD, not an i64/String the harness can pack
;; alongside these two in one return value without a THIRD accessor round-trip.
(:wat::core::defn :user::ctx-populated-id-and-op [] -> :(wat::core::i64,wat::core::String)
  (:wat::core::let
    [h (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::CallCtx3/whoami c (:probe::CallCtx3::WhoamiRequest))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::CallCtx3::WhoamiResponse::Ok caller-id _namespace operation)
            (:wat::core::Tuple caller-id operation))
          ((:probe::CallCtx3::WhoamiResponse::RequestTooLarge _b _c) (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
          ((:probe::CallCtx3::WhoamiResponse::RequestMalformed _p _e _g) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "whoami: stop before reply" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "whoami: closed before reply" :wat::core::None :wat::core::None)))))

;; namespace equals the service's own fqdn — a keyword equality check, kept as its own bool-
;; returning driver (the harness above already proves caller-id/operation).
(:wat::core::defn :user::ctx-namespace-is-fqdn [] -> :wat::core::bool
  (:wat::core::let
    [h (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::CallCtx3/whoami c (:probe::CallCtx3::WhoamiRequest))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::CallCtx3::WhoamiResponse::Ok _caller-id namespace _operation)
            (:wat::core::= namespace :probe::callctx3svc))
          ((:probe::CallCtx3::WhoamiResponse::RequestTooLarge _b _c) (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
          ((:probe::CallCtx3::WhoamiResponse::RequestMalformed _p _e _g) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "whoami: stop before reply" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "whoami: closed before reply" :wat::core::None :wat::core::None)))))

;; ── (2) a 2-param arm in the SAME service still works (proving OPT-IN, not migration) ────
(:wat::core::defn :user::two-param-arm-still-works [] -> :wat::core::bool
  (:wat::core::let
    [h (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::CallCtx3/ping c (:probe::CallCtx3::PingRequest))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::CallCtx3::PingResponse::Ok ok) ok)
          ((:probe::CallCtx3::PingResponse::RequestTooLarge _b _c) (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
          ((:probe::CallCtx3::PingResponse::RequestMalformed _p _e _g) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "ping: stop before reply" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "ping: closed before reply" :wat::core::None :wat::core::None)))))

;; ── (3) ★ THE STABILITY GATE ───────────────────────────────────────────────────────────────
;; Connect c1, c2, c3 IN ORDER (each `connect'` is a blocking handshake, so the server has
;; processed connection N before the client's connect' call for N+1 even begins — the mint
;; order is deterministic by construction) — ALL THREE simultaneously connected, matching the
;; brief's literal scenario. c3 is the one whose POSITION shifts when the MIDDLE client (c2) is
;; evicted (c1 does not shift — a buggy positional scheme would still pass if tested on c1,
;; which is exactly the trap: c3 is the only survivor this can catch on).
;;
;; c2 is confined to `stability-connect-phase`'s OWN stack frame: it is connected, round-tripped
;; (so the server has genuinely processed its Connection event, not merely the socket handshake),
;; and then simply never returned — unlike a `let`-binder that shares the CALLER's frame (which
;; would keep c2 reachable, hence alive, for the caller's entire body). Mirrors the corpus idiom
;; ("Scope-exit drops `svc` → RAII drain", tests/services/probe_arc209_c2_defservice_dispatch.wat)
;; at CLIENT-peer scope instead of whole-service scope. A `:user::`/`:probe::` fn cannot call
;; `:wat::kernel::close` directly (RAII-only, `restricted_to` — the user never holds the rope), so
;; scope-confinement is the only lever available. c1 and c3 (and c3's baseline id) DO escape (the
;; return tuple), so by the time this call returns, c1/c3 are alive in the CALLER's frame and c2
;; is the only one gone.
(:wat::core::defn :probe::stability-connect-phase [h <- :probe::callctx3svc::Handle]
  -> :(wat::core::i64,probe::CallCtx3,probe::CallCtx3)
  (:wat::core::let
    [c1 (:probe::connect! h)
     _  (:probe::whoami-id c1)
     c2 (:probe::connect! h)
     _  (:probe::whoami-id c2)
     c3 (:probe::connect! h)
     id-before (:probe::whoami-id c3)]
    (:wat::core::Tuple id-before c1 c3)))

;; Returns Tuple(id-before, id-after) — the harness asserts they are EQUAL (and, as a second,
;; independent proof, that id-after is the ANALYTICALLY correct value 2 — the third id ever
;; minted — not merely "unchanged from whatever id-before happened to be").
(:wat::core::defn :user::stability-gate [] -> :(wat::core::i64,wat::core::i64)
  (:wat::core::let
    [h         (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record))
     phase     (:probe::stability-connect-phase h)
     id-before (:wat::core::first phase)
     c1        (:wat::core::second phase)
     c3        (:wat::core::third phase)
     ;; c2 is ALREADY gone (its frame returned above) — nap gives the server's single-threaded
     ;; serve loop room to have processed its Closed event before the second read.
     _         (:probe::nap!)
     id-after  (:probe::whoami-id c3)
     ;; c1 is touched too — a bonus, non-load-bearing witness that the OTHER survivor also still
     ;; works after the middle eviction (not asserted on by the harness; a raise here would fail
     ;; the test regardless, same as every other `assertion-failed!` transport guard in this file).
     _         (:probe::whoami-id c1)]
    (:wat::core::Tuple id-before id-after)))
