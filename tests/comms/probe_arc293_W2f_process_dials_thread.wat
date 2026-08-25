;; 293.W.2f FM 2-bis — a process runner handed a thread handle.
;;
;; RED at HEAD: this TYPE-CHECKS (the Address type lies). The live MCP
;; then dies in EDN (`RustOpaque` at dial-runner).
;; GREEN after 2f: startup is a CheckError — a process may not dial
;; a shared-memory address.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest [msg <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo
  :durable []
  :ephemeral []
  :impls
  [(echo [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::Echo::EchoResponse::Ok
         (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]]
  -> :wat::core::String
  (:wat::core::match (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item))
    ((:wat::kernel::RecvOutcome::Message recvd)
      (:wat::core::match recvd
        ((:probe::Echo::EchoResponse::Ok reply) reply)
        ((:probe::Echo::EchoResponse::RequestTooLarge _b _c)
          (:wat::kernel::assertion-failed! "too-large" :wat::core::None :wat::core::None))
        ((:probe::Echo::EchoResponse::RequestMalformed _p _e _g)
          (:wat::kernel::assertion-failed! "malformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause)
        :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::illegal [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [eh (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))]
    (:wat::bracket::map (:wat::spawn::process) ["a"] :probe::work :echo eh)))
