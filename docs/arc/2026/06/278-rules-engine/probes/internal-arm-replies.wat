;; internal-arm-replies.wat — the disconfirming probe for item 2 of THE ORDER
;; (DESIGN-STONE-the-outcome-composes.md). RUN 2026-09-01. Result recorded below.
;;
;; ONE QUESTION: may an INTERNAL arm — `[s ctx]`, no request, no caller — construct
;; `Outcome::Reply`, replying to a caller it does not have?
;;
;; `wat/service.wat:127` says the INPUT side already forbids the confusion: an internal arm
;; receives a `SelfInvocation`, "never an `Invocation` (it has no connection, so it has no
;; `conn-id` field)". The OUTPUT side is one type for both.
;;
;; ══ MEASURED ═══════════════════════════════════════════════════════════════════════════
;; The answer depends on HOW YOU SPELL THE REPLY, which is the finding.
;;
;;   (Outcome::Reply s (:probe::Solo::PingResponse::Ok))                    -> REJECTED
;;   (Outcome::Reply s (:probe::Solo::Reply::Ping (…PingResponse::Ok)))     -> COMPILES
;;
;; The rejection is NOT a rule about internal arms. It is a type collision inside the
;; generated serve loop, and its diagnostic names neither the arm nor the reason:
;;
;;   :wat::core::foldl: parameter #1 expects [bool (Directed :- [:probe::Solo::PingResponse]) :-> bool];
;;                                       got [bool (Directed :- [:probe::Solo::Reply])        :-> bool]
;;
;; The narrow reply forces Outcome's `:R` to the per-op response type; the serve loop's
;; Directed machinery needs the union. Spell the reply as the UNION and `:R` unifies, the
;; collision evaporates, and the file compiles. **The check-time wall is an inference
;; accident with a one-token bypass, not a rule.**
;;
;; ── AND THEN IT RUNS ──────────────────────────────────────────────────────────────────
;; The compiling form was executed. Output:
;;
;;   #wat.kernel/AssertionFailure
;;     "defservice: an internal (-) op returned Outcome::Reply, but an internal op has no
;;      client to reply to (return NoReply / NoReplyAndArm / ReplyTo)"
;;   "r1=ok;r2=lost"
;;
;; So a REAL guard exists — deliberate, well-worded, three variants of it at
;; `wat/service.wat:1666-1674` (Reply, Stop, ReplyAndArm) — but it lives in the SERVE LOOP,
;; at run time. The service DIES: the first ping is `ok`, the tick fires, and the second
;; ping on the same connection comes back `lost`.
;;
;; ══ THE LADDER, MEASURED ═══════════════════════════════════════════════════════════════
;;   rung 2 (a check that fires)  — WHERE IT IS TODAY, and only when that path executes.
;;                                  A `-tick` that replies is a latent crash that ships.
;;   rung 3 (no form for it)      — where `SelfOutcome` puts it: an internal arm has no
;;                                  `reply` field, so the mistake cannot be written down.
;;
;; And the accidental check-time rejection actively HURTS: for the narrow spelling the
;; author is shown a `:wat::core::foldl` / `Directed` type mismatch, which hides the correct,
;; well-worded diagnostic sitting twenty lines away in the same file.
;;
;; To reproduce the REJECTED half: change `-tick` in :probe::loud to reply with
;; `(:probe::Solo::PingResponse::Ok)` directly. That form cannot live in this file, because
;; the file must compile in order to answer the runtime half below.
;;
;; ⚠ WHY IT LIVES HERE AND NOT IN wat-scripts/. `every_wat_scripts_file_loads` type-checks
;; every .wat under wat-scripts/; a probe whose job is to find out whether it is rejected
;; cannot be gated on being accepted.

(:wat::core::defsurface :probe::Solo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Solo::PingRequest [])
   (:wat::core::defenum :probe::Solo::PingResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::Solo  req <- :probe::Solo::PingRequest]
     -> :probe::Solo::PingResponse :max-request-bytes 524288)])

;; ✅ CONTROL — an internal arm that replies to nobody.
(:wat::service::defservice :probe::quiet
  :satisfies :probe::Solo
  :durable   []
  :ephemeral []
  :impls
  [(ping  [s ctx req] (:wat::service::Outcome::Reply s (:probe::Solo::PingResponse::Ok)))
   (-tick [s ctx]     (:wat::service::Outcome::NoReply s))])

;; ⛔ THE LIVE HOLE — `ping` arms a tick; the tick replies, with no caller in existence.
;; This COMPILES. The runtime half asks what it then does.
(:wat::service::defservice :probe::loud
  :satisfies :probe::Solo
  :durable   []
  :ephemeral []
  :impls
  [(ping  [s ctx req]
     (:wat::service::Outcome::ReplyAndArm s (:probe::Solo::PingResponse::Ok)
       [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)]))
   (-tick [s ctx]
     (:wat::service::Outcome::Reply s
       (:probe::Solo::Reply::Ping (:probe::Solo::PingResponse::Ok))))])

;; ── the runtime half ───────────────────────────────────────────────────────────────────
(:wat::core::defn :probe::nap-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

(:wat::core::defn :probe::dial
  [a <- (:wat::kernel::Address :- [:probe::Solo::Op :probe::Solo::Reply])] -> :probe::Solo
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    (_ (:wat::kernel::assertion-failed! "dial: refused" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::tag
  [rr <- (:wat::kernel::RecvOutcome :- [:probe::Solo::PingResponse])] -> :wat::core::String
  (:wat::core::match rr
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:probe::Solo::PingResponse::Ok) "ok")
        (_ "other")))
    ((:wat::kernel::RecvOutcome::Lost _c) "lost")
    (:wat::kernel::RecvOutcome::Stopped "stopped")
    (:wat::kernel::RecvOutcome::Closed "closed")))

;; ping (arms the tick) -> wait past the tick -> ping again on the SAME connection.
;; r2 is the question: did the caller-less reply corrupt the stream or kill the service?
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h  (:probe::loud/start :locus (:wat::spawn::thread) :record (:probe::loud::Record))
     c  (:probe::dial (:probe::loud::Handle/addr h))
     r1 (:probe::Solo/ping c (:probe::Solo::PingRequest))
     _  (:probe::nap-ms 60)
     r2 (:probe::Solo/ping c (:probe::Solo::PingRequest))]
    (:wat::kernel::println
      (:wat::core::format "r1={a};r2={b}" :a (:probe::tag r1) :b (:probe::tag r2)))))
