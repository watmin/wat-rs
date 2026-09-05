;; Arc 278 — THE ACCEPTANCE GATE for ctx-is-mandatory: EVERY public op arm now takes the
;; MANDATORY third param `[s ctx req]`, and every internal (`-`) op arm takes `[s ctx]` — no
;; longer opt-in. See docs/arc/2026/06/278-rules-engine/BRIEF-ctx-is-mandatory.md and
;; DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md (which SUPERSEDES BRIEF-the-call-context.md /
;; DESIGN-STONE-the-call-context.md — this file used to be that strike's gate; it is now this
;; one's, upgraded arm-by-arm rather than replaced, per "extend, do not start a new one").
;;
;; Modelled on tests/services/probe_arc278_per_op_request_too_large.{rs,wat} (connect/round-trip
;; shape) and tests/types/probe_arc278_opaque_purity_wall.* (the acceptance-gate framing).
;;
;; `:wat::service::Invocation` (public) / `:wat::service::SelfInvocation` (internal) — the
;; ratified names (2026-08-09).
;;
;; Four things, and the second is the one that matters most (STOP-0's own words — the test that
;; would have caught it, and nothing else in the suite could):
;;   1. A public arm receives a populated `Invocation` — namespace/operation/conn-id.
;;   2. ★ An internal arm receives a populated `SelfInvocation`, read THROUGH the ctx binder —
;;      `operation`/`namespace`, with NO `conn-id` field to even ask for (structural, not a
;;      runtime absence).
;;   3. A 2-param public op arm is now a LOCATED COMPILE ERROR naming the op — a `.wat.bad`
;;      fixture (tests/services/probe_arc278_call_context_two_param_public_arm.wat.bad), red IS
;;      pass (a hole-demonstration cannot live where everything must load).
;;   4. ★ THE STABILITY GATE — the id survives an eviction: connect three clients, disconnect
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
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::CallCtx3::PingRequest [])
   (:wat::core::defenum :probe::CallCtx3::PingResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   ;; arc 278 ctx-is-mandatory item (2) — `arm-mark` (client-callable) arms the INTERNAL `-mark`
   ;; op via a one-shot Alarm; `-mark` (never on the wire, no request/response) stamps what its
   ;; OWN `SelfInvocation` ctx said into durable state; `peek-mark` (client-callable) reads it
   ;; back. This is the ONLY way to observe an internal arm's ctx: it has no client to reply to.
   (:wat::core::defrecord :probe::CallCtx3::ArmMarkRequest [])
   (:wat::core::defenum :probe::CallCtx3::ArmMarkResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::CallCtx3::PeekMarkRequest [])
   (:wat::core::defenum :probe::CallCtx3::PeekMarkResponse :wat::enum::Pure
     :Ok               [seen-op <- :wat::core::String  seen-ns <- :wat::core::keyword]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(whoami   [self <- :probe::CallCtx3  req <- :probe::CallCtx3::WhoamiRequest]   -> :probe::CallCtx3::WhoamiResponse   :max-request-bytes 524288)
   (ping     [self <- :probe::CallCtx3  req <- :probe::CallCtx3::PingRequest]     -> :probe::CallCtx3::PingResponse     :max-request-bytes 524288)
   (arm-mark [self <- :probe::CallCtx3  req <- :probe::CallCtx3::ArmMarkRequest]  -> :probe::CallCtx3::ArmMarkResponse  :max-request-bytes 524288)
   (peek-mark [self <- :probe::CallCtx3  req <- :probe::CallCtx3::PeekMarkRequest] -> :probe::CallCtx3::PeekMarkResponse :max-request-bytes 524288)])

;; The satisfier — every public arm is `[s ctx req]` (MANDATORY, arc 278 ctx-is-mandatory); the
;; internal `-mark` arm is `[s ctx]`, ctx : `SelfInvocation` (item (2)'s subject).
(:wat::service::defservice :probe::callctx3svc
  :satisfies :probe::CallCtx3
  :durable   [seen-op <- :wat::core::String  seen-ns <- :wat::core::keyword]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::callctx3svc::Record] -> :probe::callctx3svc::State
          (:probe::callctx3svc::State :durable record))
  :impls
  [(whoami [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::CallCtx3::Reply::Whoami (:probe::CallCtx3::WhoamiResponse::Ok
         (:wat::service::Invocation/conn-id ctx)
         (:wat::service::Invocation/namespace ctx)
         (:wat::service::Invocation/operation ctx)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::CallCtx3::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::callctx3svc::Op])])))
   (ping [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::CallCtx3::Reply::Ping (:probe::CallCtx3::PingResponse::Ok true))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::CallCtx3::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::callctx3svc::Op])])))
   ;; client op: arm the ONE-SHOT `-mark` (no re-arm — one fire is all this gate needs).
   (arm-mark [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:probe::CallCtx3::Reply::ArmMark (:probe::CallCtx3::ArmMarkResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::CallCtx3::Reply])]) [(:wat::service::Alarm :delay (:wat::time::Milliseconds 5) :op :-mark)]))
   ;; ★ item (2) — the INTERNAL op. Its ctx (`SelfInvocation`) is read THROUGH the ctx binder and
   ;; stamped into durable state — the only channel available, since an internal op has no client
   ;; to reply to. Never `Invocation`, never a `conn-id` (STOP-3: SelfInvocation has no such field
   ;; to even ask for).
   (-mark [s ctx]
     (:wat::core::let
       [rec (:probe::callctx3svc::Record
              :seen-op (:wat::service::SelfInvocation/operation ctx)
              :seen-ns (:wat::service::SelfInvocation/namespace ctx))
        s'  (:probe::callctx3svc::State :durable rec)]
       (:wat::service::SelfOutcome::Continue s' (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::CallCtx3::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::callctx3svc::Op])]))))
   (peek-mark [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::CallCtx3::Reply::PeekMark (:probe::CallCtx3::PeekMarkResponse::Ok
         (:probe::callctx3svc::Record/seen-op (:probe::callctx3svc::State/durable s))
         (:probe::callctx3svc::Record/seen-ns (:probe::callctx3svc::State/durable s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::CallCtx3::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::callctx3svc::Op])])))])

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
    (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "whoami: closed before reply" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))

;; A bounded, event-driven wait (NOT a sleep-guess — arc 278's own precedent, DESIGN-STONE-the-
;; call-context.md's reproduction: "wait 60ms via a select'-on-after nap"): arm a one-shot timer
;; and block on its OWN recv. Gives the server's single-threaded serve loop room to have already
;; processed a dropped client's Closed event by the time the caller proceeds.
(:wat::core::defn :probe::nap! [] -> :wat::core::nil
  (:wat::core::let
    [t (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Milliseconds 60) :tick)]
    (:wat::core::match (:wat::kernel::recv t)
      ((:wat::kernel::RecvOutcome::Message _tick) nil)
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped nil)
      (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil))))

;; ── (1) a 3-param arm receives a POPULATED ctx ────────────────────────────────────────────
;; Returns a Tuple(caller-id, operation) — the harness checks caller-id >= 0 (present) and
;; operation == "whoami" (the op's own kebab name, spliced as a compile-time literal). namespace
;; is checked separately (below) since it is a KEYWORD, not an i64/String the harness can pack
;; alongside these two in one return value without a THIRD accessor round-trip.
(:wat::core::defn :user::ctx-populated-id-and-op [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [h (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record :seen-op "" :seen-ns :probe::none))
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
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "whoami: closed before reply" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; namespace equals the service's own fqdn — a keyword equality check, kept as its own bool-
;; returning driver (the harness above already proves caller-id/operation).
(:wat::core::defn :user::ctx-namespace-is-fqdn [] -> :wat::core::bool
  (:wat::core::let
    [h (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record :seen-op "" :seen-ns :probe::none))
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
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "whoami: closed before reply" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ── a SECOND public op in the SAME service, also `[s ctx req]` — unremarkable on its own, but
;; it keeps the service honest that ctx isn't special-cased to whichever op happens to be first
;; declared. (This used to be "a 2-param arm still works, proving opt-in" — that framing died with
;; the opt-in design; ctx is unconditional now, so `ping` carries it exactly like `whoami` does.)
(:wat::core::defn :user::second-public-arm-also-works [] -> :wat::core::bool
  (:wat::core::let
    [h (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record :seen-op "" :seen-ns :probe::none))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::CallCtx3/ping c (:probe::CallCtx3::PingRequest))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::CallCtx3::PingResponse::Ok ok) ok)
          ((:probe::CallCtx3::PingResponse::RequestTooLarge _b _c) (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
          ((:probe::CallCtx3::PingResponse::RequestMalformed _p _e _g) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "ping: stop before reply" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "ping: closed before reply" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ── (2) ★ THE test — an INTERNAL arm receives a populated `SelfInvocation` ────────────────────
;; `arm-mark` (a public op) arms the one-shot internal `-mark`; `-mark` stamps its OWN ctx into
;; durable state (the only channel: an internal op has no client to reply to — STOP-0's own
;; mechanism, now closed: the old internal branch dropped this binder silently, so a body that
;; read it would have compiled and returned the durable record's ZERO-VALUE defaults forever,
;; never firing red). `peek-mark` reads it back. A bounded, event-driven poll (mirrors
;; probe_arc278_self_scheduling.wat's `poll-until` — NOT a sleep-guess) waits for the async fire.
(:wat::core::defn :probe::peek-mark! [c <- :probe::CallCtx3] -> (:wat::core::Tuple :- [:wat::core::String :wat::core::keyword])
  (:wat::core::match (:probe::CallCtx3/peek-mark c (:probe::CallCtx3::PeekMarkRequest))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:probe::CallCtx3::PeekMarkResponse::Ok seen-op seen-ns) (:wat::core::Tuple seen-op seen-ns))
        ((:probe::CallCtx3::PeekMarkResponse::RequestTooLarge _b _c) (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:probe::CallCtx3::PeekMarkResponse::RequestMalformed _p _e _g) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "peek-mark: stop before reply" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "peek-mark: closed before reply" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))

;; peek-until — bounded retry, event-driven backoff (`:probe::nap!`), terminates on the OBSERVED
;; seen-op becoming non-empty (i.e. `-mark` has genuinely fired and its ctx landed in state).
(:wat::core::defn :probe::peek-until
  [c <- :probe::CallCtx3  attempts <- :wat::core::i64] -> (:wat::core::Tuple :- [:wat::core::String :wat::core::keyword])
  (:wat::core::if (:wat::i64::<= attempts 0)
    (:wat::kernel::assertion-failed! "peek-until: bound exhausted — -mark never fired" :wat::core::None :wat::core::None)
    (:wat::core::let [got (:probe::peek-mark! c)]
      (:wat::core::if (:wat::core::= (:wat::core::first got) "")
        (:wat::core::let [_ (:probe::nap!)]
          (:probe::peek-until c (:wat::i64::- attempts 1)))
        got))))

;; Returns Tuple(operation-is-dash-mark, namespace-is-fqdn) — both booleans, computed here (a
;; keyword/String equality check on the ctx facts the INTERNAL arm actually saw).
(:wat::core::defn :user::internal-arm-ctx-populated [] -> (:wat::core::Tuple :- [:wat::core::bool :wat::core::bool])
  (:wat::core::let
    [h (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record :seen-op "" :seen-ns :probe::none))
     c (:probe::connect! h)
     _ (:wat::core::match (:probe::CallCtx3/arm-mark c (:probe::CallCtx3::ArmMarkRequest))
         ((:wat::kernel::RecvOutcome::Message __r) __r)
         ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "arm-mark: stop before reply" :wat::core::None :wat::core::None))
         (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "arm-mark: closed before reply" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     seen (:probe::peek-until c 40)
     seen-op (:wat::core::first seen)
     seen-ns (:wat::core::second seen)]
    (:wat::core::Tuple (:wat::core::= seen-op "-mark") (:wat::core::= seen-ns :probe::callctx3svc))))

;; ── (4) ★ THE STABILITY GATE ───────────────────────────────────────────────────────────────
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
  -> (:wat::core::Tuple :- [:wat::core::i64 :probe::CallCtx3 :probe::CallCtx3])
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
(:wat::core::defn :user::stability-gate [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [h         (:probe::callctx3svc/start :locus (:wat::spawn::process) :record (:probe::callctx3svc::Record :seen-op "" :seen-ns :probe::none))
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
