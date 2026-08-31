;; probe-handle-to-surface-relation.wat — DISCONFIRMING PROBE for the "rung 3" wall.
;;
;; THE PROPOSED WALL: a `Peer` may not outlive the scope that owns its service's `Handle`. The
;; whole design rests on ONE assumption, and this file exists to try to break it:
;;
;;   ⟹ from a value of type `<svc>::Handle`, the checker can derive THAT SERVICE'S surface types
;;     (`<S>::Op` / `<S>::Reply`) — precisely enough to tell one service's peer from another's.
;;
;; If it can, the wall is a local type-level relation between a Handle-typed binding and a
;; Peer-typed escape, needing no lifetimes, no linearity and no brands. If it cannot, the design
;; is dead as drawn and must find another route — better to learn that here than in a stone.
;;
;; TWO services, deliberately: a single service proves nothing, because a checker that erased the
;; relation entirely would still accept one correct annotation by luck. Discrimination BETWEEN two
;; is the real question, and it is the same question `Dialable/coord` already answers — it hands
;; each satisfier its OWN typed `(Address :- [S R])` so a wrong-service dial is a compile-time
;; error (wat/capability.wat). This probe asks whether that machinery reaches a Handle.
;;
;; GREEN (this file type-checks) = the relation is computed and precise for BOTH services →
;;   the assumption holds, draw the stone.
;; RED = the checker cannot get from a Handle to its service's surface types → the wall as drawn
;;   is not buildable; the error names exactly which step is missing.
;;
;; The NEGATIVE half — annotating alpha's coord with BETA's address type, which MUST be rejected —
;; cannot live here: `every_wat_scripts_file_loads` type-checks this tree, so a deliberately-red
;; file would turn the floor red. Run as a transient copy; its verdict, verbatim, 2026-08-31:
;;
;;   #wat.check/ReturnTypeMismatch
;;     ":hs::alpha-addr-is-typed: body produces (:wat::kernel::Address :- [:hs::Alpha::Op :hs::Alpha::Reply]);
;;      signature declares (:wat::kernel::Address :- [:hs::Beta::Op :hs::Beta::Reply])"
;;
;; ★ RESULT — THE ASSUMPTION HOLDS. Given ONLY `h <- :hs::alpha::Handle`, the checker derived
;; alpha's own concrete `Op`/`Reply` and discriminated them from beta's. So the Handle -> (S,R)
;; relation is computed, per-service precise, and reachable through BOTH `Dialable/coord` and the
;; handle's own `<svc>::Handle/addr`.
;;
;; It also settles WHERE the wall's first case can live: that error is a ReturnTypeMismatch, which
;; means the checker holds the param types and the return type AT THE SAME MOMENT and can already
;; compute the service relation between them. Case 1 of the wall — a `Peer` escaping through a
;; function signature whose param is that service's `Handle` — is therefore reachable with the
;; machinery that produced this message, not with new machinery.
;;
;; ⚠ WHAT THIS PROBE DOES NOT ESTABLISH: case 2, the `let`-scope escape (a tail call carrying a
;; Peer out of the scope that binds its Handle). That is a question about whether the checker sees
;; a let's BINDING LIST and its TAIL EXPRESSION together, which is scope-tracking, not the type
;; relation this file measured. Unproven here — do not brief it as covered.

;; ── service ALPHA ────────────────────────────────────────────────────────────────────────────
(:wat::core::defsurface :hs::Alpha :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :hs::Alpha::PingRequest [])
   (:wat::core::defenum :hs::Alpha::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :hs::Alpha  req <- :hs::Alpha::PingRequest] -> :hs::Alpha::PingResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :hs::alpha
  :satisfies :hs::Alpha
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :hs::alpha::Record] -> :hs::alpha::State
          (:hs::alpha::State :durable record))
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Reply s (:hs::Alpha::PingResponse::Pong)))])

;; ── service BETA — a DIFFERENT surface, so its address type is distinguishable from alpha's ──
(:wat::core::defsurface :hs::Beta :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :hs::Beta::PokeRequest [])
   (:wat::core::defenum :hs::Beta::PokeResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(poke [self <- :hs::Beta  req <- :hs::Beta::PokeRequest] -> :hs::Beta::PokeResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :hs::beta
  :satisfies :hs::Beta
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :hs::beta::Record] -> :hs::beta::State
          (:hs::beta::State :durable record))
  :impls
  [(poke [s ctx req] (:wat::service::Outcome::Reply s (:hs::Beta::PokeResponse::Ok)))])

;; ── THE ASSAY ────────────────────────────────────────────────────────────────────────────────
;; Each binding carries an EXACT annotation naming that service's own Op/Reply. An annotation is a
;; claim the checker must verify, so these type-checking IS the evidence: if the Handle -> (S,R)
;; relation were erased, a precise annotation could not be satisfied.
(:wat::core::defn :hs::alpha-addr-is-typed
  [h <- :hs::alpha::Handle] -> (:wat::kernel::Address :- [:hs::Alpha::Op :hs::Alpha::Reply])
  (:wat::capability::Dialable/coord h))

(:wat::core::defn :hs::beta-addr-is-typed
  [h <- :hs::beta::Handle] -> (:wat::kernel::Address :- [:hs::Beta::Op :hs::Beta::Reply])
  (:wat::capability::Dialable/coord h))

;; …and the same relation reached through the Handle's OWN accessor, which is the path a caller
;; actually writes (`(Handle/addr h)`), not just through the Dialable up-cast.
(:wat::core::defn :hs::alpha-handle-addr-is-typed
  [h <- :hs::alpha::Handle] -> (:wat::kernel::Address :- [:hs::Alpha::Op :hs::Alpha::Reply])
  (:hs::alpha::Handle/addr h))

;; ⛔ CORRECTED 2026-08-31 — this function was committed under the name
;; `:hs::the-escape-the-wall-must-reject`, and that label was WRONG. It is the ordinary `conn`
;; helper every caller writes, and it is SAFE: the handle arrives as a PARAM, so the CALLER still
;; owns it and the peer cannot outlive it. A wall keyed on "param is a Handle, return is a Peer"
;; would reject this and every `conn` in the corpus.
;;
;; The discriminator is not the parameter, it is WHO CREATED THE HANDLE — see
;; `:hs::dial-and-drop-is-the-real-escape` below. Recorded rather than quietly renamed: a wrong
;; acceptance criterion is how a wall gets built against the wrong shape.
(:wat::core::defn :hs::conn-is-safe-the-caller-owns-the-handle
  [h <- :hs::alpha::Handle] -> (:wat::kernel::Peer :- [:hs::Alpha::Op :hs::Alpha::Reply])
  (:wat::core::match (:wat::kernel::connect (:hs::alpha::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; THE REAL CASE-1 ESCAPE has MOVED, and its location was a specification error worth recording.
;; It lived here, in wat-scripts/, while EXPECTATIONS row 1 demanded this file be REJECTED — but
;; `every_wat_scripts_file_loads` type-checks every .wat under wat-scripts/, so those two
;; requirements contradict each other by construction and the floor went red the moment the wall
;; started working. The repo already had the home for a must-be-rejected file:
;; `docs/arc/2026/06/278-rules-engine/probes/red-*.wat`.
;;
;; It now lives at `docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-creation-escape.wat`,
;; driven by `tests/services/probe_ex002_creation_escape.rs`. What stays HERE is the GREEN
;; feasibility half — the Handle -> surface-type relation — which the loader gate keeps proving.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "handle->surface relation: both services annotate precisely"))
