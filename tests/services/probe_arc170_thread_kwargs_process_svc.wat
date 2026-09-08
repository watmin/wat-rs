;; Thread `bracket/map` + kwargs tail against a PROCESS service.
;;
;; The surface is loci-agnostic: `(map (thread) items :work :echo eh)` must
;; Setup-dial and hold the client peer. A thread worker can reach a process
;; service (same pid as the owner). EXPECT ["echo:a" "echo:b" "echo:c"].

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
     (:wat::service::Outcome::Reply {:state s
       :reply (:probe::Echo::EchoResponse::Ok
         (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))}))])

(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]]
  -> :wat::core::String
  (:wat::core::match (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item))
    [:wat::kernel::RecvOutcome::Message {:msg recvd}
      (:wat::core::match recvd
        [:probe::Echo::EchoResponse::Ok {:reply reply} reply]
        [:probe::Echo::EchoResponse::RequestTooLarge {:bytes _b :cap _c}
          (:wat::kernel::assertion-failed! :message "work: unexpected RequestTooLarge")]
        [:probe::Echo::EchoResponse::RequestMalformed {:path _p :expected _e :got _g}
          (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])]
    [:wat::kernel::RecvOutcome::Lost {:cause cause}
      (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
    [:wat::kernel::RecvOutcome::Stopped {}
      (:wat::kernel::assertion-failed! :message "recv': stopped")]
    [:wat::kernel::RecvOutcome::Closed {}
      (:wat::kernel::assertion-failed! :message "recv': peer closed")]))

(:wat::core::defn :probe::run [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [eh (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))]
    (:wat::bracket::map (:wat::spawn::thread) ["a" "b" "c"] :probe::work :echo eh)))
