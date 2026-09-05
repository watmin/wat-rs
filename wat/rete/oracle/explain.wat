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
                                (:wat::core::PersistentMap/get network node-id)
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
                      (:wat::core::match (:wat::core::PersistentMap/get s2 derived)
                        ((:wat::core::Some _) s2)
                        (:wat::core::None
                         (:wat::core::PersistentMap/assoc s2 derived
                           (:wat::rete::Support :rule rname :token tok))))))
                  s
                  rhs))
              sup
              toks))
          sup)))
    (:wat::core::PersistentMap)
    (:wat::rete::topological-node-ids network)))

;; fire-rules-explain$oracle — wat reference for explain. Same session as
;; fire-rules$oracle; support harvested from a fire-once$oracle replay of the
;; closure so beta is live. First-producer-wins over topological-node-ids,
;; matching the native index (`sorted_node_ids`).
(:wat::core::defn :wat::rete::fire-rules-explain$oracle
  [session <- :wat::rete::Session]
  ;; ⛔ SAME TYPE AS THE NATIVE, by the dual-impl contract — `(FireOutcome :- [Explained])`. The
  ;; oracle enforces no ceilings, so it can only ever answer `Fired`; answering a bare `Explained`
  ;; would make the differential harness unwrap one side and not the other, i.e. compare two
  ;; different things.
  -> (:wat::rete::FireOutcome :- [:wat::rete::Explained])
  (:wat::core::let [input       (:wat::rete::Session/facts session)
                    ;; HAND-FACED (arc 278 the fire-outcome wall) — stdlib, per-site semantic.
                    ;; The oracle enforces no ceilings, so only `Fired` is reachable; the other
                    ;; arms say so loudly rather than being swallowed.
                    oracle-sess (:wat::core::match (:wat::rete::fire-rules$oracle session)
                                  ((:wat::rete::FireOutcome::Fired __f) __f)
                                  ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
                                    (:wat::kernel::assertion-failed!
                                      "fire-rules-explain$oracle: memory ceiling — the oracle enforces none"
                                      :wat::core::None :wat::core::None))
                                  ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
                                    (:wat::kernel::assertion-failed!
                                      "fire-rules-explain$oracle: round cap — the oracle enforces none"
                                      :wat::core::None :wat::core::None)))
                    derived     (:wat::rete::collect-derived
                                  (:wat::rete::Session/production-memory oracle-sess))
                    closed      (:wat::rete::merge-facts input derived)
                    empty       (:wat::core::PersistentMap)
                    ;; ⛔ HAND-FACED, not codemod'd — arc 278 the fire-outcome wall. This is a
                    ;; STDLIB site with per-site semantics (the oracle's own replay), and the
                    ;; codemod is a wat program that cannot load while the stdlib is red, so the
                    ;; bootstrap order is: face this by hand, THEN sweep the corpus with the tool.
                    ;; Same precedent as the connect'-wall codemod, whose header records that its
                    ;; stdlib sites were hand-faced for exactly this reason.
                    ;;
                    ;; The `$oracle` enforces no ceilings, so it can only ever answer `Fired` —
                    ;; the standing accepted asymmetry ("the $oracle is the reference an embedder
                    ;; never runs"). The other two arms are therefore UNREACHABLE HERE, and they
                    ;; say so loudly rather than being swallowed: if one ever fires, the oracle has
                    ;; grown a ceiling and this comment is the thing that was wrong.
                    replay      (:wat::core::match
                                  (:wat::rete::fire-once$oracle
                                    (:wat::rete::Session
                                      :network (:wat::rete::Session/network session)
                                      :rules (:wat::rete::Session/rules session)
                                      :alpha-memory empty
                                      :beta-memory empty
                                      :production-memory empty
                                      :facts closed
                                      :next-id (:wat::rete::Session/next-id session)
                                      :query-memory empty))
                                  ((:wat::rete::FireOutcome::Fired __replayed) __replayed)
                                  ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
                                    (:wat::kernel::assertion-failed!
                                      "fire-rules-explain$oracle: the oracle replay hit a memory ceiling — the oracle enforces none"
                                      :wat::core::None :wat::core::None))
                                  ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
                                    (:wat::kernel::assertion-failed!
                                      "fire-rules-explain$oracle: the oracle replay hit a round cap — the oracle enforces none"
                                      :wat::core::None :wat::core::None)))
                    support     (:wat::rete::harvest-support
                                  (:wat::rete::Session/network replay)
                                  (:wat::rete::Session/beta-memory replay)
                                  (:wat::rete::Session/rules session))]
    (:wat::rete::FireOutcome::Fired
      (:wat::rete::Explained :session oracle-sess :support support))))

