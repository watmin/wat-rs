;; Arc 278 #57 — WHY `:wat::core::unquote` is NOT a migration target, proven by run.
;;
;; The where-census walker (probe-where-census-walker.wat) reports 13 `:wat::core::unquote`
;; head-occurrences inside `where` forms, across 5 rule-builder files. That count is CORRECT for
;; a SOURCE walk and MISLEADING as a worklist, and the difference is worth one measurement rather
;; than one inference — the census walks source text, the fence inspects the form the engine is
;; actually handed.
;;
;; `(:wat::core::unquote x)` only ever appears inside `(:wat::core::quasiquote …)`, where it is
;; TEMPLATE ESCAPE SYNTAX, not a call. Evaluating the quasiquote substitutes x's VALUE, so by the
;; time `compile-condition` sees the `where` there is no `unquote` node left to admit or refuse.
;;
;; A source census cannot see that, because it is measuring a different artifact than the fence is.
;; Hence: 13 occurrences, ZERO migration targets, and no rete `unquote` row is needed — or possible.
;;
;; Shape copied from the real builders (`wat-scripts/perf/grid/min-finding.wat:65-70`,
;; `node-share.wat:65-70`): a fn parameter spliced into a quasiquoted `where`.

(:wat::core::defn :uq::build-where [threshold <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::quasiquote
    (:wat::rete::where (:wat::core::>= ?n (:wat::core::unquote threshold)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [built (:uq::build-where 42)]
    (:wat::kernel::println
      (:wat::core::PersistentMap
        ;; THE MEASUREMENT: the rendered form the fence would receive. If `unquote` survived
        ;; evaluation it would appear here as a head; if it is template syntax, `42` appears in
        ;; its place and the head is gone.
        :built-form built
        ;; Non-vacuity: a DIFFERENT argument must produce a DIFFERENT form. If both printed the
        ;; same thing, the splice never happened and the line above would prove nothing.
        :other-form (:uq::build-where 7)))))
