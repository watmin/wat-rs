;; probe-m1-worker-setup.wat — DISCONFIRMING PROBE for M1-pool shape (a), the ONE new composition:
;; a worker recv's a Setup(addr) over the wire → DIALS-and-HOLDS the service (admitted via grant) →
;; serves Work(item) messages using the HELD peer, reused across items. Isolates the runner
;; setup-dial-hold-reuse-over-a-union pattern (the rest of shape (a) is already-proven pieces).
;;
;; EXPECT (green):  echo:a echo:b   (two Work items served through one held, granted connection)

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse
                (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

;; the union the worker recv's: Setup hands the address; Work is one unit of work.
(:wat::core::defenum :probe::Msg :wat::enum::Pure
  :Setup [addr <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>]
  :Work  [s    <- :wat::core::String])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh   (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea   (:probe::echo'::Handle/addr eh)
     worker (:wat::kernel::spawn-program' (:wat::spawn::process)
              (:wat::core::forms
                ;; child fresh world — re-declare the surface + the union it dials/receives
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
                (:wat::core::defenum :probe::Msg :wat::enum::Pure
                  :Setup [addr <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>]
                  :Work  [s    <- :wat::core::String])
                ;; the serve loop: threads the held service peer (Option, None until Setup)
                (:wat::core::defn :probe::serve
                  [self <- :wat::kernel::Peer'<wat::core::String,probe::Msg>
                   held <- (:wat::core::Option :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>)]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::recv' self) -> :wat::core::nil
                    ((:probe::Msg::Setup addr)
                      (:probe::serve self (:wat::core::Some (:wat::kernel::connect' addr))))   ;; DIAL-and-HOLD
                    ((:probe::Msg::Work s)
                      (:wat::core::let
                        [c  (:wat::core::Option/expect held "Work before Setup")
                         er (:probe::Echo/echo c (:probe::Echo::EchoRequest s))               ;; via the HELD peer
                         _  (:wat::kernel::send' self (:probe::Echo::EchoResponse/reply er))]
                        (:probe::serve self held)))))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String :probe::Msg)]
                    (:probe::serve self :wat::core::None)))))
     out  (:wat::core::match (:wat::kernel::peer-pid worker) -> :wat::core::String
            ((:wat::core::Some p)
              (:wat::core::let
                [_  (:probe::echo'/grant eh (:wat::core::Vector :wat::core::i64 p)) ;; grant BEFORE the setup dial
                 _  (:wat::kernel::send' worker (:probe::Msg::Setup ea))            ;; worker dials-and-holds (admitted)
                 _  (:wat::kernel::send' worker (:probe::Msg::Work "a"))
                 r1 (:wat::kernel::recv' worker)
                 _  (:wat::kernel::send' worker (:probe::Msg::Work "b"))
                 r2 (:wat::kernel::recv' worker)]
                (:wat::core::string::concat r1 (:wat::core::string::concat " " r2))))
            (:wat::core::None
              (:wat::kernel::assertion-failed! "peer-pid None on process worker"
                :wat::core::None :wat::core::None)))]
    (:wat::kernel::println out)))
