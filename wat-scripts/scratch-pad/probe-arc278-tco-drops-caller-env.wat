;; probe-arc278-tco-drops-caller-env.wat — the MECHANISM behind the "let-tail
;; reaps its binding" bug (companion to probe-arc278-let-tail-service-reaped.wat,
;; which merely EXHIBITS it).
;;
;; GROUNDED VERDICT (2026-07-25): it is not `let`. It is the TAIL-CALL TRAMPOLINE.
;;
;;   src/runtime.rs:3402 emit_tail_call — evaluates the tail call's ARGS, then returns
;;     Err(EvalBreak::Signal(EvalSignal::TailCall{func,args})). The callee's BODY has
;;     not run yet.
;;   src/runtime.rs:3522 eval_let_tail — `eval_tail(&body[last], &scope, sym)` returns
;;     that signal; `scope` (the let's Environment, holding every binding) is dropped
;;     ON RETURN.
;;   src/runtime.rs:20207/20223/20238 apply_function — `call_env` is loop-local; the
;;     TailCall arm `continue`s, dropping the caller's whole env chain, and only THEN
;;     does the next iteration build the callee's frame and evaluate its body.
;;
;; So every binding of the calling frame is dropped BEFORE the tail callee runs. For a
;; pure value that is invisible. For a value holding a LIVE RESOURCE (RAII drop) it is a
;; reap: the resource dies, the callee runs against a corpse, and the failure returns
;; through a legitimate-looking outcome variant carrying a false story.
;;
;; It is GENERAL, not service-specific — the raw-kernel `Listener'` row below reaps the
;; same way and manufactures `ConnectOutcome::Refused`.
;;
;; Expected output (each pair: identical call, non-tail then tail):
;;
;;   "service : non-tail => Message (served)"
;;   "service : let-TAIL => CLOSED"            <- false Closed (R53: means clean EOF)
;;   "listener: non-tail => CONNECTED"
;;   "listener: let-TAIL => REFUSED"           <- false Refused
;;
;; The four rows are the disproof of the "drops scheduled at the end of the BINDING
;; LIST" hypothesis too: in the non-tail rows the very same call sits AFTER the binding
;; list and is served. Only genuine tail position reaps.

(:wat::core::defsurface :tco::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :tco::Bag::PutRequest [n <- :wat::core::i64])
   (:wat::core::defenum :tco::Bag::PutResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :tco::Bag  req <- :tco::Bag::PutRequest]
     -> :tco::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :tco::bag-svc
  :satisfies :tco::Bag  :durable [n <- :wat::core::i64]  :ephemeral []
  :impls
  [(put [s ctx req] (:wat::service::Outcome::Reply s (:tco::Bag::PutResponse::Ok 1)))])

(:wat::core::defn :tco::try [c <- (:wat::kernel::Peer :- [:tco::Bag::Op :tco::Bag::Reply])
                            label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:tco::Bag/put c (:tco::Bag::PutRequest :n 1))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::kernel::println (:wat::string::concat label " => Message (served)")))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println (:wat::string::concat label " => LOST")))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::println (:wat::string::concat label " => STOPPED")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println (:wat::string::concat label " => CLOSED")))))

(:wat::core::defn :tco::dial [a <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])
                             label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p)
      (:wat::kernel::println (:wat::string::concat label " => CONNECTED")))
    ((:wat::kernel::ConnectOutcome::Refused f)
      (:wat::kernel::println (:wat::string::concat label " => REFUSED")))
    ((:wat::kernel::ConnectOutcome::Rejected f)
      (:wat::kernel::println (:wat::string::concat label " => REJECTED")))
    ((:wat::kernel::ConnectOutcome::Failed f)
      (:wat::kernel::println (:wat::string::concat label " => FAILED")))))

;; ── row 1: the service call is NOT in tail position (a form follows it) ──────────
(:wat::core::defn :tco::service-non-tail [] -> :wat::core::nil
  (:wat::core::let
    [h (:tco::bag-svc/start :locus (:wat::spawn::thread) :record (:tco::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:tco::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))]
    (:wat::core::do (:tco::try c "service : non-tail") nil)))

;; ── row 2: the SAME call, now the let's tail — TCO drops the frame first ─────────
(:wat::core::defn :tco::service-let-tail [] -> :wat::core::nil
  (:wat::core::let
    [h (:tco::bag-svc/start :locus (:wat::spawn::thread) :record (:tco::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:tco::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))]
    (:tco::try c "service : let-TAIL")))

;; ── rows 3+4: a NON-service live resource — a raw kernel Listener' ───────────────
(:wat::core::defn :tco::listener-non-tail [] -> :wat::core::nil
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l    (:wat::spawn::Bound/listener pair)
     a    (:wat::spawn::Bound/address pair)]
    (:wat::core::do (:tco::dial a "listener: non-tail") nil)))

(:wat::core::defn :tco::listener-let-tail [] -> :wat::core::nil
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l    (:wat::spawn::Bound/listener pair)
     a    (:wat::spawn::Bound/address pair)]
    (:tco::dial a "listener: let-TAIL")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:tco::service-non-tail)
    (:tco::service-let-tail)
    (:tco::listener-non-tail)
    (:tco::listener-let-tail)
    nil))
