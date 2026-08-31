;; red-tail-escape.wat — the DELIBERATELY-RED target for excursus 002 stone 2.
;;
;; This file MUST NOT type-check. It is the wall's acceptance criterion: a `let` that CREATES a
;; Handle, in tail position, whose tail expression is a user-function call carrying that service's
;; Peer out while the scope — and the handle — die underneath it.
;;
;; `tests/services/probe_ex002_tail_escape.rs` drives it and asserts the rejection structurally —
;; that the wall names `:red::tail-escape`, and that it does NOT name `:red::held` (drive sits in a
;; binding, so the let is in tail position but the tail expression is not a user-function call) or
;; `:red::builtin-head` (the tail head is `:wat::i64::+`, which emits no TailCall).
;;
;; ⚠ WHY IT LIVES HERE AND NOT IN wat-scripts/.
;; `every_wat_scripts_file_loads` type-checks EVERY .wat under wat-scripts/, so a must-be-rejected
;; file there turns the floor red for as long as the wall works. That was stone 1's specification
;; error. Deliberately-red probes live under a `probes/` directory in their own arc/excursus.
;;
;; ⚠ A `rune:` ON THIS FILE IS AN AUTOMATIC FAIL. It would silence the wall's only proof that it
;; fires, producing a green floor from a wall that catches nothing. Rune the instrument
;; (`probe-self-sched-bisect.wat`); never rune the acceptance criterion.

(:wat::core::defsurface :red::Alpha :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :red::Alpha::PingRequest [])
   (:wat::core::defenum :red::Alpha::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :red::Alpha  req <- :red::Alpha::PingRequest] -> :red::Alpha::PingResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :red::alpha
  :satisfies :red::Alpha
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :red::alpha::Record] -> :red::alpha::State
          (:red::alpha::State :durable record))
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Reply s (:red::Alpha::PingResponse::Pong)))])

;; ✅ MUST KEEP COMPILING — the ordinary `conn` helper. Handle arrives as a PARAM.
(:wat::core::defn :red::conn
  [h <- :red::alpha::Handle] -> (:wat::kernel::Peer :- [:red::Alpha::Op :red::Alpha::Reply])
  (:wat::core::match (:wat::kernel::connect (:red::alpha::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :red::consume-peer
  [c <- (:wat::kernel::Peer :- [:red::Alpha::Op :red::Alpha::Reply])] -> :wat::core::i64
  (:wat::core::match (:red::Alpha/ping c (:red::Alpha::PingRequest))
    ((:wat::kernel::RecvOutcome::Message __r) 1)
    ((:wat::kernel::RecvOutcome::Lost __c) -1)
    (:wat::kernel::RecvOutcome::Stopped -2)
    (:wat::kernel::RecvOutcome::Closed -3)))

;; ✅ MUST KEEP COMPILING — drive sits in a BINDING. The let is in tail position of the function,
;; but its tail expression is the symbol `n`, not a user-function call. If the wall names this,
;; condition 2 was ignored (or "the let's body is a call" was used as a stand-in) and every
;; non-tail let is a false positive.
(:wat::core::defn :red::held [] -> :wat::core::i64
  (:wat::core::let
    [h (:red::alpha/start :locus (:wat::spawn::thread) :record (:red::alpha::Record :n 0))
     c (:red::conn h)
     n (:red::consume-peer c)]
    n))

;; ✅ MUST KEEP COMPILING — D-body-nontail. The tail head is `:wat::i64::+`, a builtin/defclause
;; that emits no TailCall, so the let's scope survives across the drive. Measured green in the
;; bisect. If the wall names this, user-function detection used `env.get` (which also finds
;; builtins) instead of `sym.has_function`.
(:wat::core::defn :red::builtin-head [] -> :wat::core::i64
  (:wat::core::let
    [h (:red::alpha/start :locus (:wat::spawn::thread) :record (:red::alpha::Record :n 0))
     c (:red::conn h)]
    (:wat::i64::+ (:red::consume-peer c) 0)))

;; ⛔ MUST BE REJECTED — `h` is CREATED here, the let is the function body (tail position), and
;; the tail expression is a user-function call taking the Peer's type. The scope ends before
;; `:red::consume-peer` runs.
(:wat::core::defn :red::tail-escape [] -> :wat::core::i64
  (:wat::core::let
    [h (:red::alpha/start :locus (:wat::spawn::thread) :record (:red::alpha::Record :n 0))
     c (:red::conn h)]
    (:red::consume-peer c)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "this file must never freeze"))
