;; probe-roundtrip-cost.wat — WHERE does the circuit's time go? Interpretation, or the wire?
;;
;; The fan-out circuit runs its store, queues and workers at :locus process, so every store op is an
;; IPC round-trip with an EDN encode/decode on each side. "It is slow because it is interpreted" is a
;; hypothesis; so is "it is slow because every op crosses a process boundary as text". They are
;; separable: run the SAME trivial service call at both loci and compare.
;;
;;   thread  = interpreter + in-process channel  (no encode, no pipe)
;;   process = interpreter + EDN encode/decode + pipe
;;
;; The delta between them is the wire's price. What remains in the thread number is the interpreter's.

(:wat::core::defsurface :rt::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :rt::Echo::PingRequest [])
   (:wat::core::defenum :rt::Echo::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :rt::Echo  req <- :rt::Echo::PingRequest] -> :rt::Echo::PingResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :rt::echo
  :satisfies :rt::Echo
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :rt::echo::Record] -> :rt::echo::State
          (:rt::echo::State :durable record))
  :impls
  [(ping [s ctx req] (:wat::service::Outcome::Reply s (:rt::Echo::PingResponse::Pong)))])

(:wat::core::defn :rt::ping-n
  [c <- (:wat::kernel::Peer :- [:rt::Echo::Op :rt::Echo::Reply])  n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= n 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match (:rt::Echo/ping c (:rt::Echo::PingRequest))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil))]
      (:rt::ping-n c (:wat::i64::- n 1)))))

;; a pure interpreter loop of the same shape and count — no service, no channel, no wire.
;; What this costs is the interpreter's own per-iteration price, with nothing else in it.
(:wat::core::defn :rt::spin [n <- :wat::core::i64  acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= n 0)
    acc
    (:rt::spin (:wat::i64::- n 1) (:wat::i64::+ acc 1))))

(:wat::core::defn :rt::time-thread [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h  (:rt::echo/start :locus (:wat::spawn::thread) :record (:rt::echo::Record :n 0))
     c  (:wat::core::match (:wat::kernel::connect (:rt::echo::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _p (:rt::ping-n c n)
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     ms (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :rt::time-process [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h  (:rt::echo/start :locus (:wat::spawn::process) :record (:rt::echo::Record :n 0))
     c  (:wat::core::match (:wat::kernel::connect (:rt::echo::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _p (:rt::ping-n c n)
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     ms (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :rt::time-spin [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     _s (:rt::spin n 0)
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     ms (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [sp (:rt::time-spin 2000)
     th (:rt::time-thread 2000)
     pr (:rt::time-process 2000)]
    (:wat::kernel::println
      (:wat::string::interpolate
        "2000 iterations -> pure-interpreter-loop={a}ms | thread-locus round-trips={b}ms | process-locus round-trips={c}ms"
        :a sp :b th :c pr))))
