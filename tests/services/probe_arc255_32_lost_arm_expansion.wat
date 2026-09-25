;; Stone 255.32 — expand a defservice at the form level and print the emitted
;; ServiceEvent.Lost arm. A defservice cannot be runtime-macroexpanded.
(:wat::core::defsurface :p32::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :p32::Echo::EchoRequest [msg <- :wat::core::String])
   (:wat::core::defenum :p32::Echo::EchoResponse :wat::enum::Pure
     :Ok [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(echo [self <- :p32::Echo req <- :p32::Echo::EchoRequest] -> :p32::Echo::EchoResponse
     :max-request-bytes 524288)])

(:wat::core::defn :p32::lost-arm [exp <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let
    [s    (:wat::core::write-forms exp)
     cut  (:wat::string::split s "ServiceEvent.Lost")]
    (:wat::core::if (:wat::core::< (:wat::core::count cut) 2)
      "<no ServiceEvent.Lost arm>"
      (:wat::core::let
        [rest (:wat::core::nth cut 1)
         end  (:wat::string::split rest "ServiceEvent.Malformed")]
        (:wat::string::concat "ServiceEvent.Lost" (:wat::core::first end))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [exp (:wat::core::macroexpand
           (:wat::core::quote
             (:wat::service::defservice :p32::echo
               :satisfies :p32::Echo :durable [] :ephemeral []
               :impls [(echo [s ctx req]
                         (:wat::service::Outcome.Reply
                           {:state s
                            :reply (:p32::Echo::EchoResponse.Ok {:reply "v"})}))])))]
    (:wat::kernel::println (:p32::lost-arm exp))))
