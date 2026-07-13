;; Arc 170 Strike C1 — THE GATE. A full clean-surface run: a kwargs work-fn
;; ([item & [echo <- Peer'<...>]]) dialed via bracket/map's OWN `:name val` tail
;; (arc 170 gap J — provisioning rides map/each directly; `process/uses` retired),
;; run through bracket/map with the CLEAN base name (never $impl).
;; EXPECT (green): ["echo:a" "echo:b" "echo:c"]
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable []  :ephemeral []
  :impls [(echo [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse :reply
                (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]]
  -> :wat::core::String
  (:probe::Echo::EchoResponse/reply
    (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh    (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     out   (:wat::bracket::map (:wat::spawn::process) ["a" "b" "c"] :probe::work :echo eh)]
    (:wat::kernel::println out)))
