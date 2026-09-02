;; probe-tail-scope-sees-bindings.wat — DISCONFIRMING PROBE, case 2 of the "rung 3" wall.
;;
;; probe-handle-to-surface-relation.wat settled case 1 (a Peer escaping through a FUNCTION
;; SIGNATURE) and explicitly did NOT settle this one. Case 2 is the escape that actually cost 38
;; days: a Peer carried out of a `let` scope by a TAIL CALL, which ends that scope — and with it
;; the `Handle` it binds — before the call runs.
;;
;; The wall for case 2 would have to fire at the tail expression and ask: "does this scope bind a
;; Handle whose service matches a Peer I am carrying out?" That needs ONE thing to be true, and
;; this file exists to try to break it:
;;
;;   ⟹ while checking a `let`'s TAIL EXPRESSION, the checker still holds the types of that let's
;;     BINDINGS.
;;
;; It is a different question from case 1's. Case 1 was a type RELATION (Handle -> S,R), proven
;; reachable because a ReturnTypeMismatch names a param type and a return type together. This is
;; SCOPE-TRACKING: whether the two facts are co-present at the moment a tail call is inferred.
;; Structurally they ought to be — `let` inference binds its bindings into scope and then infers
;; the body — but "ought to" is the reasoning that produced the 38-day claim, so: measure it.
;;
;; ★ RESULT 1 — THE ASSUMPTION HOLDS. The discriminating case is a transient copy whose tail
;; expression demands an i64 and is handed `h`. Verdict, verbatim, 2026-08-31:
;;
;;   #wat.check/TypeMismatch
;;     ":c2::needs-an-i64: parameter #1 expects :wat::core::i64;
;;      got (:c2::alpha::Handle :- [:wat::kernel::Shared])"
;;
;; Naming the binding's type AT the tail call is the proof: the checker still holds the let's
;; binding types while inferring the tail expression, so both facts case 2's wall needs are
;; co-present. (It also shows a Handle carries a transport param, `:wat::kernel::Shared` — the
;; wall's rule must match on the Handle HEAD, not a bare path, or it will miss every handle.)
;;
;; ★ RESULT 2 — UNSOUGHT, AND IT MATTERS MORE. Running the two targets, ten times:
;;
;;   6 x "tail-escape=-1 held=1"      (-1 = Lost: the severed notice arrived)
;;   4 x "tail-escape=-3 held=1"      (-3 = Closed: the MUTE — no notice at all)
;;
;; The severed notice is genuinely racy in the tightest shape, and this is the first measurement
;; of it. `LociDiedError::Severed`'s doc already says arrival is best-effort and its ABSENCE
;; proves nothing; that caveat is now a number rather than a caution.
;;
;; HYPOTHESIS, not established: the broadcast walks the peers the serve loop has ALREADY accepted
;; into its set, so a client that has connected but whose `ServiceEvent::Connection` is not yet
;; processed is absent from it and gets the mute. Do not repeat as cause without measuring — that
;; is exactly the move that cost 38 days upstream of this file.
;;
;; ⚠ WHY THIS SHARPENS THE CASE FOR THE WALL rather than weakening it: a runtime notice that can
;; lose a race can never be THE answer to an ownerless service — only a backstop. A compile-time
;; wall does not race. The diagnostic buys a good error once the program has run; the wall stops
;; the program existing.
;;
;; The gate guarding the notice (tests/services/probe_severed_reaches_the_client.rs) uses the
;; scope-exit shape, not this tail shape, and was stress-run 90x with 0 failures. Strong evidence
;; for THAT shape; not a proof of determinism for the mechanism. Recorded as measured.

(:wat::core::defsurface :c2::Alpha :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :c2::Alpha::PingRequest [])
   (:wat::core::defenum :c2::Alpha::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :c2::Alpha  req <- :c2::Alpha::PingRequest] -> :c2::Alpha::PingResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :c2::alpha
  :satisfies :c2::Alpha
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :c2::alpha::Record] -> :c2::alpha::State
          (:c2::alpha::State :durable record))
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Continue s (:wat::core::Some (:c2::Alpha::Reply::Ping (:c2::Alpha::PingResponse::Pong))) (:wat::core::Vector :- [(:wat::service::Directed :- [:c2::Alpha::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:c2::alpha::Op])])))])

(:wat::core::defn :c2::conn
  [h <- :c2::alpha::Handle] -> (:wat::kernel::Peer :- [:c2::Alpha::Op :c2::Alpha::Reply])
  (:wat::core::match (:wat::kernel::connect (:c2::alpha::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; a plain consumer, so a tail call can carry a Peer out of a scope.
(:wat::core::defn :c2::consume-peer
  [c <- (:wat::kernel::Peer :- [:c2::Alpha::Op :c2::Alpha::Reply])] -> :wat::core::i64
  (:wat::core::match (:c2::Alpha/ping c (:c2::Alpha::PingRequest))
    ((:wat::kernel::RecvOutcome::Message __r) 1)
    ((:wat::kernel::RecvOutcome::Lost __c) -1)
    (:wat::kernel::RecvOutcome::Stopped -2)
    (:wat::kernel::RecvOutcome::Closed -3)))

;; ── THE PROBE OF SCOPE-VISIBILITY ────────────────────────────────────────────────────────────
;; `h` is bound HERE and used ONLY inside the tail expression. If the checker did not still hold
;; `h`'s type while inferring that tail call, this could not resolve at all. It type-checking is
;; the positive half; the transient negative below is what makes it discriminating.
;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: the visibility half of
;; this probe is itself a tail escape (start in the let, consume-peer in the body).
;; The file must keep running; rune the instrument, never the acceptance criterion.
(:wat::core::defn :c2::binding-is-visible-in-the-tail [] -> :wat::core::i64
  (:wat::core::let
    [h (:c2::alpha/start :locus (:wat::spawn::thread) :record (:c2::alpha::Record :n 0))]
    (:c2::consume-peer (:c2::conn h))))

;; ── THE SHAPE THE WALL MUST REJECT — case 2, and the one that cost 38 days. ──────────────────
;; Legal today, type-checks today. `h` binds alpha's Handle; the tail call carries alpha's Peer
;; out of the scope that owns it, so the scope ends, the handle drops, and `consume-peer` meets a
;; severed service. When the stone lands, THIS function is what must stop compiling — and
;; `:c2::held` directly below it, which differs only in that the drive sits in a binding, is what
;; must KEEP compiling. A wall that cannot tell these two apart is too blunt to ship.
;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: this is the shape the
;; wall rejects; the probe must construct it so user::main can print the race. The
;; acceptance criterion lives in probes/red-tail-escape.wat and is NOT runed.
(:wat::core::defn :c2::the-tail-escape-the-wall-must-reject [] -> :wat::core::i64
  (:wat::core::let
    [h (:c2::alpha/start :locus (:wat::spawn::thread) :record (:c2::alpha::Record :n 0))
     c (:c2::conn h)]
    (:c2::consume-peer c)))

(:wat::core::defn :c2::held [] -> :wat::core::i64
  (:wat::core::let
    [h (:c2::alpha/start :locus (:wat::spawn::thread) :record (:c2::alpha::Record :n 0))
     c (:c2::conn h)
     n (:c2::consume-peer c)]
    n))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::interpolate "tail-escape={a} held={b}"
      :a (:c2::the-tail-escape-the-wall-must-reject)
      :b (:c2::held))))
