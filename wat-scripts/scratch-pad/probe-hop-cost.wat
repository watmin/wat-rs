;; probe-hop-cost.wat — what does ONE request/reply hop cost, at each locus?
;;
;; WHY. The circuit's drain is ~70s for 8000 deliveries = 8.75 ms each. Each delivery is roughly
;; four request/reply hops (topic->adapter->queue->store and back). The store was measured and
;; swapped: it accounts for ~1% of the circuit. If a bare hop is tens of microseconds, then ~8.7 ms
;; per delivery is unexplained and the cost is structural. If a bare hop is milliseconds, the cost
;; IS the hops and there is nothing mysterious. This measures the baseline instead of carrying it
;; from memory.
;;
;; A do-nothing service: one op, replies immediately, no state, no I/O.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::PingRequest [])
   (:wat::core::defenum :probe::Echo::PingResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::Echo  req <- :probe::Echo::PingRequest]
     -> :probe::Echo::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Echo::Reply::Ping (:probe::Echo::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defn :probe::dial-echo
  [a <- (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])] -> :probe::Echo
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "dial-echo failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::spin [c <- :probe::Echo  n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::nil  _i <- :wat::core::i64] -> :wat::core::nil
            (:wat::core::match (:probe::Echo/ping c (:probe::Echo::PingRequest))
              ((:wat::kernel::RecvOutcome::Message _r) nil)
              ((:wat::kernel::RecvOutcome::Lost cause)
                (:wat::kernel::assertion-failed!
                  (:wat::string::concat "ping LOST: " (:wat::kernel::LociDiedError/message cause))
                  :wat::core::None :wat::core::None))
              (:wat::kernel::RecvOutcome::Stopped
                (:wat::kernel::assertion-failed! "ping STOPPED" :wat::core::None :wat::core::None))
              (:wat::kernel::RecvOutcome::Closed
                (:wat::kernel::assertion-failed! "ping CLOSED" :wat::core::None :wat::core::None))))
          nil
          (:wat::core::range 0 n))
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) n)))

(:wat::core::defn :probe::thread-ns [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h  (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))
     c  (:probe::dial-echo (:probe::echo::Handle/addr h))
     ns (:probe::spin c n)]
    ns))

(:wat::core::defn :probe::process-ns [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     c  (:probe::dial-echo (:probe::echo::Handle/addr h))
     ns (:probe::spin c n)]
    ns))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::format "thread={t}ns process={p}ns" :t (:probe::thread-ns 200) :p (:probe::process-ns 200)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::format "per-hop: thread={t}ns  process={p}ns"
      :t (:probe::thread-ns 5000) :p (:probe::process-ns 5000))))
