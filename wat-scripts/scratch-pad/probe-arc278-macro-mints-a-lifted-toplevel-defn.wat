;; PROBE — STOP-3: can a macro MINT A TOP-LEVEL DEFN with a COMPUTED NAME from
;; inside its own expansion, alongside the consumer that references it?
;;
;; The whole `defrule` lift rests on this: defrule must emit TWO top-level forms
;; (the lifted `<rule>$where0` body and the rule defn) where today it emits one.
;; If a macro cannot do that, the lifted body is not ordinary top-level code and
;; the stone collapses like the two before it.
;;
;; PRECEDENT (not proof): wat/core.wat:1147 — the kwargs `defn` macro emits
;; `(do ~record-def (def ~impl-name-node …) (defmacro ~name-base-node …))`, with
;; the computed name minted by `keyword-node`. Shipping stdlib machinery. This
;; probe RUNS it in the defrule shape rather than trusting the resemblance.
;;
;; ⚠ NON-VACUITY — the consumer calls the LIFTED fn by its COMPUTED name. If the
;; mint silently produced a different name, the call fails to resolve and this
;; file does not load. Loading and printing `true` is the proof, and `$where0`
;; being usable as a LET BINDER is tested here too (a naming-cast input).

(:wat::core::defmacro :probe::lift-a-where
  [name <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [raw         (:wat::core::ast-name name)
     lifted-str  (:wat::string::concat raw "$where0")
     lifted-node (:wat::core::keyword-node lifted-str)]
    `(:wat::core::do
       ;; the LIFTED body — ordinary top-level code, computed name
       (:wat::core::defn ~lifted-node [?c <- :wat::core::i64] -> :wat::core::bool
         (:wat::core::i64::> ?c 100))
       ;; the consumer — mentions the lifted fn, then calls it
       (:wat::core::defn ~name [] -> :wat::core::bool
         (:wat::core::let [~(:wat::core::symbol-node "$where0") ~lifted-node]
           (~lifted-node 150))))))

(:probe::lift-a-where :probe::r1)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat "STOP-3 macro-minted top-level lifted defn = "
      (:wat::core::if (:probe::r1) "true" "false"))))
