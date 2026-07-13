;; probe-m1-ann-erase.wat — can ann-form erase concrete Address'<S,R> -> bare Address',
;; store in Vector<Address'>, then send it (bare) to a child that recv's it concrete + connects?
;; The parent's PoolMsg<Address',...> and child's PoolMsg<Address'<S,R>,...> are SEPARATE
;; typecheck universes; only the wire bytes must match.
;; EXPECT (green): "echo:z"

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
              (:probe::Echo::EchoResponse :reply
                (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

;; PARENT-side PoolMsg with BARE Address' payload (erased D).
(:wat::core::defenum :probe::PMsg<I> :wat::enum::Pure
  :Setup [addr <- :wat::kernel::Address']
  :Work  [s    <- :I])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh   (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea   (:probe::echo'::Handle/addr eh)
     ;; ERASE concrete Address'<Op,Reply> -> bare Address' via ann-form:
     eab  (:wat::core::ann-form ea :wat::kernel::Address')
     erased (:wat::core::Vector :wat::kernel::Address' eab)
     worker (:wat::kernel::spawn-program' (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
                ;; CHILD-side PoolMsg with CONCRETE Address'<Op,Reply> payload.
                (:wat::core::defenum :probe::CMsg :wat::enum::Pure
                  :Setup [addr <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>]
                  :Work  [s    <- :wat::core::String])
                (:wat::core::defn :probe::serve
                  [self <- :wat::kernel::Peer'<wat::core::String,probe::CMsg>
                   held <- (:wat::core::Option :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>)]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::recv' self) -> :wat::core::nil
                    ((:probe::CMsg::Setup addr)
                      (:probe::serve self (:wat::core::Some (:wat::kernel::connect' addr))))
                    ((:probe::CMsg::Work s)
                      (:wat::core::let
                        [c  (:wat::core::Option/expect held "Work before Setup")
                         er (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg s))
                         _  (:wat::kernel::send' self (:probe::Echo::EchoResponse/reply er))]
                        (:probe::serve self held)))))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String :probe::CMsg)]
                    (:probe::serve self :wat::core::None)))))
     out  (:wat::core::match (:wat::kernel::peer-pid worker) -> :wat::core::String
            ((:wat::core::Some p)
              (:wat::core::let
                [_  (:probe::echo'/grant eh (:wat::core::Vector :wat::core::i64 p))
                 ;; parent sends a BARE-typed Setup; child decodes into concrete slot.
                 _  (:wat::kernel::send' worker (:probe::PMsg::Setup (:wat::core::first erased)))
                 _  (:wat::kernel::send' worker (:probe::PMsg::Work "z"))
                 r1 (:wat::kernel::recv' worker)]
                r1))
            (:wat::core::None
              (:wat::kernel::assertion-failed! "peer-pid None" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println out)))
