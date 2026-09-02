;; Arc 255 Stones 1a-β-0 / 1a-β-0b — the orchestrator's INDEPENDENT weigh of two riders' kills.
;;
;; Reads back, through the substrate's own reflection surface, the three claims the
;; two stones make about `:wat::core::defsurface` — the first form registered that is
;; neither checked nor evaluated:
;;
;;   1. the registry answers for it at all           (it is a member)
;;   2. the rete fence REFUSES it as impure          (`@Purity Unevaluated` ⇒ not usable
;;                                                    in a rule body — it is not a runtime
;;                                                    expression at all)
;;   3. `show-source` renders the DECLARE role       (the third regime, naming the
;;                                                    freeze-time fn that processes it)
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println (:wat::rete::pure? '(:wat::core::defsurface)))
    (:wat::kernel::println (:wat::rete::pure? '(:wat::core::if true 1 2)))
    (:wat::kernel::println (:wat::core::show-source :wat::core::defsurface))))
