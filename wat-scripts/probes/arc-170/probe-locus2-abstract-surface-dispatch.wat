;; probe-locus2-abstract-surface-dispatch.wat — disconfirming probe for the Locus→surface flip, risk 2:
;; can an AGGREGATE (:nature :Struct) surface's METHOD be called through an ABSTRACT surface-typed
;; fn param, dispatching to the concrete extend-type impl? This is EXACTLY the shape Locus needs:
;;   defservice's `start [locus <- :Locus] ... (:Locus/launch locus ...)` and S2's runner-count.
;;
;; :probe::drive holds `r <- :probe::Runner` (the abstract surface) and calls (:probe::Runner/run r n).
;; At the call site a concrete :probe::Doubler is passed (structural satisfaction); inside drive the
;; abstract-typed r must dispatch to Doubler's extend-type impl (the check.rs:6104 open-surface path).
;;
;; GREEN target: prints "42".

(:wat::core::defsurface :probe::Runner :nature :wat::core::Struct
  :features [(run [self <- :probe::Runner  n <- :wat::core::i64] -> :wat::core::i64)])

(:wat::core::defstruct :probe::Doubler [])
(:wat::core::extend-type :probe::Doubler :probe::Runner
  (run [self n] (:wat::i64::* n 2)))

;; hold at the ABSTRACT surface type; dispatch on the concrete satisfier at runtime
(:wat::core::defn :probe::drive [r <- :probe::Runner  n <- :wat::core::i64] -> :wat::core::i64
  (:probe::Runner/run r n))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::i64::to-string (:probe::drive (:probe::Doubler) 21))))  ;; expect 42
