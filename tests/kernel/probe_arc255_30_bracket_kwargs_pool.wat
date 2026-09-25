;; Stone 255.30 — the bracket thread pool with a kwargs work-fn. PoolMsg.Setup
;; carries a Shared address. The pool runs.
(:wat::core::defsurface :p30::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :p30::Echo::EchoRequest [msg <- :wat::core::String])
   (:wat::core::defenum :p30::Echo::EchoResponse :wat::enum::Pure
     :Ok [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(echo [self <- :p30::Echo req <- :p30::Echo::EchoRequest] -> :p30::Echo::EchoResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :p30::echo
  :satisfies :p30::Echo
  :durable []
  :ephemeral []
  :impls
  [(echo [s ctx req]
     (:wat::service::Outcome.Reply {:state s
       :reply (:p30::Echo::EchoResponse.Ok
         {:reply (:wat::string::concat "echo:" (:p30::Echo::EchoRequest/msg req))})}))])

(:wat::core::defn :p30::work
  [item <- :wat::core::String
   & [echo <- (:wat::kernel::Peer :- [:p30::Echo::Op :p30::Echo::Reply])]]
  -> :wat::core::String
  (:wat::core::match (:p30::Echo/echo echo (:p30::Echo::EchoRequest :msg item))
    [:wat::kernel::RecvOutcome.Message {:msg recvd}
      (:wat::core::match recvd
        [:p30::Echo::EchoResponse.Ok {:reply reply} reply]
        [:p30::Echo::EchoResponse.RequestTooLarge {:bytes _b :cap _c}
          (:wat::kernel::assertion-failed! :message "too-large")]
        [:p30::Echo::EchoResponse.RequestMalformed {:path _p :expected _e :got _g}
          (:wat::kernel::assertion-failed! :message "malformed")])]
    [:wat::kernel::RecvOutcome.Lost {:cause c}
      (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message c))]
    [:wat::kernel::RecvOutcome.Stopped {}
      (:wat::kernel::assertion-failed! :message "stopped")]
    [:wat::kernel::RecvOutcome.Closed {}
      (:wat::kernel::assertion-failed! :message "closed")]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh (:p30::echo/start :locus (:wat::spawn::thread) :record (:p30::echo::Record))
     out (:wat::bracket::map (:wat::spawn::thread) ["a" "b" "c"] :p30::work :echo eh)]
    (:wat::kernel::println out)))
