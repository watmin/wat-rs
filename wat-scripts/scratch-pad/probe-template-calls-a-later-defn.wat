;; probe-template-calls-a-later-defn.wat — CAN A MACRO'S TEMPLATE CALL A defn DEFINED AFTER IT?
;;
;; R1 v2 died on STOP-4: four EDN goldens snapshot service.wat LINE NUMBERS (`:line 896`),
;; so inserting the helper before `defservice` shifted them +13 and redded the floor for a
;; reason that has nothing to do with behaviour.
;;
;; ★ THE FIX v3 RESTS ON: append the helper AFTER the macro instead. Then nothing before it
;; moves, the goldens hold, and the floor is a pure extraction proof again. But the macro's
;; TEMPLATE would then call a defn defined LATER IN THE SAME FILE.
;;
;; Reasoning says fine -- the template is quasiquoted data, expanded at the USE site, by
;; which time the whole file is registered. But "reasoning says fine" is what killed v1
;; (Peer :- [Never R]) and v2 (the floor as sufficient proof). So: measure it.
;;
;;   expect: expanded=84;verdict=FORWARD-REFERENCE-OK

(:wat::config::set-redef! true)

(:wat::core::defmacro :fw::twice [x <- :wat::WatAST] -> :wat::WatAST
  ;; the template calls :fw::double, which is defined BELOW this macro
  `(:fw::double ~x))

(:wat::core::defn :fw::double [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::* n 2))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [r (:fw::twice 42)]
    (:wat::kernel::println
      (:wat::core::format "expanded={r};verdict={v}"
        :r r
        :v (:wat::core::if (:wat::i64::= r 84)
             "FORWARD-REFERENCE-OK" "FORWARD-REFERENCE-FAILS")))))
