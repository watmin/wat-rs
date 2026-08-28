;; wat/rete/oracle/explain.wat — interpreted fire-rules-explain$oracle.
;;
;; harvest-support + fire-rules-explain$oracle. Loads after fire.wat
;; (replays fire-rules$oracle then fire-once$oracle).
;;
;; Namespace: :wat::rete::

;; harvest-support — first-producer-wins index: derived-fact → Support{rule, token}.
;; Replay on a session whose beta is still live (fire-once$oracle of the closure).
(:wat::core::defn :wat::rete::harvest-support
  [network  <- :wat::core::PersistentMap
   beta-mem <- :wat::core::PersistentMap
   rules    <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::core::PersistentMap
  (:wat::core::foldl
    (:wat::core::fn [sup     <- :wat::core::PersistentMap
                     node-id <- :wat::core::i64]
      -> :wat::core::PersistentMap
      (:wat::core::let [node (:wat::core::Option/expect
                                (:wat::map::get network node-id)
                                "harvest-support: node")]
        (:wat::core::if (:wat::core::= (:wat::rete::node-kind-label node) "ProductionNode")
          (:wat::core::let [rname (:wat::rete::ProductionNode/rule-name node)
                            rule  (:wat::rete::rule-by-name rules rname)
                            rhs   (:wat::rete::Rule/rhs rule)
                            toks  (:wat::rete::tokens-from-parents beta-mem
                                    (:wat::rete::node-parents node-id network))]
            (:wat::core::foldl
              (:wat::core::fn [s   <- :wat::core::PersistentMap
                               tok <- :wat::rete::Token]
                -> :wat::core::PersistentMap
                (:wat::core::foldl
                  (:wat::core::fn [s2   <- :wat::core::PersistentMap
                                   form <- :wat::WatAST]
                    -> :wat::core::PersistentMap
                    (:wat::core::let [derived (:wat::rete::eval-insert form
                                                 (:wat::rete::Token/bindings tok))]
                      (:wat::core::match (:wat::map::get s2 derived)
                        ((:wat::core::Some _) s2)
                        (:wat::core::None
                         (:wat::map::assoc s2 derived
                           (:wat::rete::Support :rule rname :token tok))))))
                  s
                  rhs))
              sup
              toks))
          sup)))
    (:wat::core::PersistentMap)
    (:wat::map::keys network)))

;; fire-rules-explain$oracle — wat reference for explain. Same session as
;; fire-rules$oracle; support harvested from a fire-once$oracle replay of the
;; closure so beta is live. First-producer-wins, matching the native index.
(:wat::core::defn :wat::rete::fire-rules-explain$oracle
  [session <- :wat::rete::Session]
  -> :wat::rete::Explained
  (:wat::core::let [input       (:wat::rete::Session/facts session)
                    oracle-sess (:wat::rete::fire-rules$oracle session)
                    derived     (:wat::rete::collect-derived
                                  (:wat::rete::Session/production-memory oracle-sess))
                    closed      (:wat::rete::merge-facts input derived)
                    empty       (:wat::core::PersistentMap)
                    replay      (:wat::rete::fire-once$oracle
                                  (:wat::rete::Session
                                    :network (:wat::rete::Session/network session)
                                    :rules (:wat::rete::Session/rules session)
                                    :alpha-memory empty
                                    :beta-memory empty
                                    :production-memory empty
                                    :facts closed
                                    :next-id (:wat::rete::Session/next-id session)
                                    :query-memory empty))
                    support     (:wat::rete::harvest-support
                                  (:wat::rete::Session/network replay)
                                  (:wat::rete::Session/beta-memory replay)
                                  (:wat::rete::Session/rules session))]
    (:wat::rete::Explained :session oracle-sess :support support)))

