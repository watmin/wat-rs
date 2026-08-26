;; PROBE — TRANSITIVITY. The mention collects the LIFTED fn. Does the extractor
;; then walk INTO it and collect what the lifted fn itself calls?
;;
;; If it does NOT, the lift ships an empty shell: the child gets `$where0` and dies
;; naming `:usr::big?`, and the macro would have to mention EVERY helper rather than
;; one name — which means free-symbol analysis inside the macro.
;;
;; ⚠ NON-VACUITY, four arms over ONE helper chain (`:usr::deep?` -> `:usr::big?`):
;;   PC   — both called in ordinary position          → the ceiling; instrument is live
;;   BASE — rule, no mention, helper only in the quote → the floor (proven 5 earlier)
;;   MENTION-1 — mention the LIFTED fn only           → does `:usr::big?` come too?
;;   MENTION-DEEP — lifted fn calls a fn that calls a fn → TWO levels of transitivity
;; MENTION-1 == PC means transitive collection works and ONE mention suffices.

(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])

;; the leaf helper
(:wat::rete::core::defn :usr::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::i64::> n 100))

;; a MIDDLE fn that calls the leaf — this is what a lifted where-body looks like
(:wat::rete::core::defn :usr::ok-rule$where0 [?c <- :wat::core::i64] -> :wat::core::bool
  (:usr::big? ?c))

;; ── PC — everything in ordinary call position: the CEILING ──────────────────
(:wat::core::defn :usr::pc [] -> :wat::core::bool
  (:usr::ok-rule$where0 150))

;; ── BASE — the shape defrule emits today: names live only inside the quote ──
(:wat::core::defn :usr::rule-base [] -> :wat::rete::Rule
  (:wat::rete::make-rule "usr::rule-base"
    (:wat::core::quote [(:usr::Temp (?c <- :c))
                        (:wat::rete::where (:usr::ok-rule$where0 ?c))])
    (:wat::core::quote [(:usr::Hot :c ?c)])))

;; ── MENTION-1 — identical, plus ONE mention of the LIFTED fn ────────────────
(:wat::core::defn :usr::rule-mentioned [] -> :wat::rete::Rule
  (:wat::core::let [$where0 :usr::ok-rule$where0]
    (:wat::rete::make-rule "usr::rule-mentioned"
      (:wat::core::quote [(:usr::Temp (?c <- :c))
                          (:wat::rete::where (:usr::ok-rule$where0 ?c))])
      (:wat::core::quote [(:usr::Hot :c ?c)]))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pc (:wat::kernel::fn-forms :usr::pc
          (:wat::keyword::from-string "user::root-pc"))
     _p (:wat::kernel::println
          (:wat::string::concat "PC        (both in call position) forms="
            (:wat::i64::to-string (:wat::core::length pc))))
     bs (:wat::kernel::fn-forms :usr::rule-base
          (:wat::keyword::from-string "user::root-bs"))
     _b (:wat::kernel::println
          (:wat::string::concat "BASE      (quote only)            forms="
            (:wat::i64::to-string (:wat::core::length bs))))
     mn (:wat::kernel::fn-forms :usr::rule-mentioned
          (:wat::keyword::from-string "user::root-mn"))
     _m (:wat::kernel::println
          (:wat::string::concat "MENTION-1 (one mention of $where0) forms="
            (:wat::i64::to-string (:wat::core::length mn))))]
    (:wat::kernel::println
      "MENTION-1 == PC => transitive; ONE mention ships the whole chain")))
