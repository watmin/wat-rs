;; probe-what-a-process-impl-can-call.wat — WHERE can a shared deadline helper live?
;;
;; Arc 278. Three of the worker's four client calls have no deadline (Seen/mark, Queue/ack,
;; Queue/receive -- whose `:wait` bounds the SERVER, not the client). Only Seen/check does, and
;; it costs ~40 lines: raw `kernel::send` + `select [peer tmr]` + redial, hand-rolled inline.
;; The undeadlined path is a one-liner. That asymmetry IS the defect.
;;
;; The cure is one shared helper. The question is where it can live, and the tree says two
;; contradictory things:
;;
;;   sqs.wat:152  "Closed over `store`. The one receive path -- process children do not see
;;                 sibling defns, so the body lives here, called via State/take."
;;   circuit.wat  the worker impl (PROCESS locus) calls `:fanout::Seen/mark`, a top-level
;;                 defsurface-generated method, and it works in production today.
;;
;; Both cannot be the whole rule. ⛔ THE DISCONFIRMING QUESTION: from inside a service impl at
;; PROCESS locus, which of these is actually reachable?
;;
;;   A. a plain top-level `defn` in the same file
;;   B. a defsurface-GENERATED client method
;;   C. a stdlib `defn` from wat/ (frozen into the binary)
;;
;; If A works, the helper lives in circuit.wat and the stone is small.
;; If only B and C work, the helper belongs in wat/service.wat beside send-keep-serving?.
;; If only B works, it must be a closure per service, like sqs.wat's `take` and `depth`.
;;
;; The SAME service is started at BOTH loci so the answer is a difference, not a reading.
;;
;; ⛔ MEASURED 2026-09-05 — RUNNING THIS FILE FAILS, BY DESIGN. That failure IS the result:
;;
;;   THREAD  locus -> the impl calls :pc::plain-helper and replies. Fine.
;;   PROCESS locus -> the spawned child dies at startup:
;;       #wat.check/UnknownCallee  unknown callee: :pc::plain-helper
;;     surfaced as LociDiedError/StartupError from :pc::probe/start$impl-process.
;;
;; So the answer is A=NO, B=YES, C=YES:
;;   A. a plain sibling defn      -- UNREACHABLE at process locus (this file proves it)
;;   B. a defsurface method       -- reachable (circuit.wat's worker impl calls Seen/mark today)
;;   C. a stdlib defn from wat/   -- reachable (sqs.wat's impls call :wat::edn::write today)
;;
;; sqs.wat:152's comment is exactly right and this is its independent confirmation. A shared
;; helper for service CLIENTS therefore cannot live in a userland file as a plain defn; it
;; belongs in wat/, beside :wat::service::send-keep-serving?.
;;
;; ⚠ SECOND FINDING, from the type-checker rather than the run: the two loci cannot share a
;; code path at all. A process handle is (Handle :- [Wire]); a thread handle is
;; (Handle :- [Shared]). `if` cannot unify them. THE LOCUS IS IN THE TYPE -- which is why this
;; file has two `try` functions and not one with a flag.
;;
;; The corpus gate (tests/lint/wat_scripts_fixes_load.rs) uses startup_from_source: it parses
;; and type-checks main's body but does NOT execute it. This file is green to the floor and
;; only speaks when run by hand.

(:wat::config::set-redef! true)

;; A — a plain top-level defn. Nothing exotic; the thing sqs.wat says is unreachable.
(:wat::core::defn :pc::plain-helper [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ x 1))

(:wat::core::defsurface :pc::Probe :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :pc::Probe::AskRequest [n <- :wat::core::i64])
   (:wat::core::defenum :pc::Probe::AskResponse :wat::enum::Pure
     :Ok [plain <- :wat::core::i64  stdlib <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ask [self <- :pc::Probe  req <- :pc::Probe::AskRequest]
     -> :pc::Probe::AskResponse :max-request-bytes 524288)])

(:wat::service::defservice :pc::probe
  :satisfies :pc::Probe
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :pc::probe::Record] -> :pc::probe::State
          (:pc::probe::State :durable record))
  :impls
  [(ask [s ctx req]
     (:wat::core::let
       [n (:pc::Probe::AskRequest/n req)
        ;; A — the plain sibling defn
        a (:pc::plain-helper n)
        ;; C — a stdlib wat defn, frozen into the binary from wat/
        c (:wat::edn::write (:wat::time::at-nanos 0))
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:pc::Probe::Reply])])
        alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:pc::probe::Op])])]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:pc::Probe::Reply::Ask (:pc::Probe::AskResponse::Ok a c)))
         sends alarms)))])

(:wat::core::defn :pc::say [r <- (:wat::kernel::RecvOutcome :- [:pc::Probe::AskResponse])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:pc::Probe::AskResponse::Ok plain stdlib)
          (:wat::core::format "plain={a};stdlib-ok={b}"
            :a plain :b (:wat::core::if (:wat::i64::> (:wat::string::length stdlib) 0) "yes" "NO")))
        (_ "malformed")))
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED") (:wat::kernel::RecvOutcome::TimedOut "LOST")))

;; ⚠ The two loci cannot share a code path: a process Handle is (Handle :- [Wire]) and a
;; thread Handle is (Handle :- [Shared]). The LOCUS IS IN THE TYPE, so `if` cannot unify them.
;; That is why these are two functions and not one with a flag -- found by the checker, here.
(:wat::core::defn :pc::try-thread [] -> :wat::core::String
  (:wat::core::let
    [h (:pc::probe/start :locus (:wat::spawn::thread) :record (:pc::probe::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:pc::probe::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "pc: thread dial failed" :wat::core::None :wat::core::None)))]
    (:pc::say (:pc::Probe/ask p (:pc::Probe::AskRequest :n 41)))))

(:wat::core::defn :pc::try-process [] -> :wat::core::String
  (:wat::core::let
    [h (:pc::probe/start
         :locus (:wat::spawn::process/post-spawn
                  (:wat::core::fn [_pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil))
         :record (:pc::probe::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:pc::probe::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "pc: process dial failed" :wat::core::None :wat::core::None)))]
    (:pc::say (:pc::Probe/ask p (:pc::Probe::AskRequest :n 41)))))

(:wat::core::defn :pc::run [] -> :wat::core::String
  (:wat::core::let
    [t (:pc::try-thread)
     p (:pc::try-process)]
    (:wat::core::format "THREAD {a} || PROCESS {b} || same={c}"
      :a t :b p :c (:wat::core::if (:wat::core::= t p) "yes" "NO-ASYMMETRY"))))

(:wat::core::defn :user::compute [] -> :wat::core::String (:pc::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:pc::run)))
