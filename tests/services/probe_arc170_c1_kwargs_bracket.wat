;; C1 DISCONFIRMING PROBE — a KWARGS work-fn invoked via the companion :key val call, on the
;; PROVEN M1 dial-hold shape (probe-m1-worker-setup). The ONLY new thing vs M1: the Work arm calls
;;   (:probe::work s :echo held)            ← the companion :key val form
;; instead of the direct (Echo/echo held req). Proves the crux for Strike C: a kwargs work-fn runs
;; when invoked with a runtime-dialed peer bound to a :key. The AST-walk's job becomes: synthesize
;; exactly this call (item + :key <held-peer> per uses field). EXPECT (green): echo:a echo:b echo:c

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse::Ok
                (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defenum :probe::Msg :wat::enum::Pure
  :Setup [addr <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>]
  :Work  [s    <- :wat::core::String])

(:wat::core::defn :probe::run [] -> :wat::core::String
  (:wat::core::let
    [eh   (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea   (:probe::echo'::Handle/addr eh)
     worker (:wat::kernel::spawn-program' (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
                     :Ok              [reply <- :wat::core::String]
                     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defenum :probe::Msg :wat::enum::Pure
                  :Setup [addr <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>]
                  :Work  [s    <- :wat::core::String])
                ;; ── the KWARGS work-fn: item positional, `echo` a :key Peer' kwarg ──
                (:wat::core::defn :probe::work
                  [item <- :wat::core::String
                   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]]
                  -> :wat::core::String
                  (:wat::core::match (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item)) -> :wat::core::String
                    ((:probe::Echo::EchoResponse::Ok reply) reply)
                    ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
                      (:wat::kernel::assertion-failed! "work: unexpected RequestTooLarge"
                        :wat::core::None :wat::core::None))))
                ;; ── serve loop: Work arm invokes via the COMPANION :key val call ──
                (:wat::core::defn :probe::serve
                  [self <- :wat::kernel::Peer'<wat::core::String,probe::Msg>
                   held <- (:wat::core::Option :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>)]
                  -> :wat::core::nil
                  (:wat::core::match (:wat::kernel::recv' self) -> :wat::core::nil
                    ((:probe::Msg::Setup addr)
                      (:probe::serve self (:wat::core::Some (:wat::kernel::connect' addr))))
                    ((:probe::Msg::Work s)
                      (:wat::core::let
                        [c (:wat::core::Option/expect held "Work before Setup")
                         r (:probe::work s :echo c)                       ;; ← companion :key val, held peer
                         _ (:wat::kernel::send' self r)]
                        (:probe::serve self held)))))
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String :probe::Msg)]
                    (:probe::serve self :wat::core::None)))))
     out  (:wat::core::match (:wat::kernel::peer-pid worker) -> :wat::core::String
            ((:wat::core::Some p)
              (:wat::core::let
                [_  (:probe::echo'/grant eh [p])
                 _  (:wat::kernel::send' worker (:probe::Msg::Setup ea))
                 _  (:wat::kernel::send' worker (:probe::Msg::Work "a"))
                 r1 (:wat::kernel::recv' worker)
                 _  (:wat::kernel::send' worker (:probe::Msg::Work "b"))
                 r2 (:wat::kernel::recv' worker)
                 _  (:wat::kernel::send' worker (:probe::Msg::Work "c"))
                 r3 (:wat::kernel::recv' worker)]
                (:wat::core::string::concat r1
                  (:wat::core::string::concat " "
                    (:wat::core::string::concat r2
                      (:wat::core::string::concat " " r3))))))
            (:wat::core::None
              (:wat::kernel::assertion-failed! "peer-pid None on process worker"
                :wat::core::None :wat::core::None)))]
    out))
