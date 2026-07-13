;; Dump the forms fn-forms emits for the dial work-fn — hunt the duplicate define.

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

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [wf (:wat::core::fn [c <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>  s <- :wat::core::String]
             -> :wat::core::String
           (:probe::Echo::EchoResponse/reply
             (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg s))))
     forms (:wat::kernel::fn-forms wf (:wat::core::keyword/from-string "user::bracket::work-fn"))]
    (:wat::core::foldl
      (:wat::core::fn [_a <- :wat::core::nil  f <- :wat::WatAST] -> :wat::core::nil
        (:wat::core::let
          [ch (:wat::core::ast->children f)
           hd (:wat::core::ast-name (:wat::core::first ch))
           nm (:wat::core::ast-name (:wat::core::first (:wat::core::drop ch 1)))]
          (:wat::kernel::println (:wat::core::string::concat hd (:wat::core::string::concat "  " nm)))))
      nil
      forms)))
