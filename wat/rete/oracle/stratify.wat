;; wat/rete/oracle/stratify.wat — type-stratum numbering (ordering layer).
;;
;; StratifyAcc, rule-produces / rule-negates / rule-consumes,
;; stratify-sweep / stratify-fix / rule-stratum / stratify.
;; Fire-stratified drive stays in fire.wat (FireStratAcc + fire-stratified*).
;; Loads before fire.wat. Dual of src/rete/kernel/stratify.rs.
;;
;; Namespace: :wat::rete::

;; ─── stratified negation (arc 300 interstitial) ─────────────────────────────
;;
;; STRATIFICATION: partition rules so every rule negating type T fires only
;; AFTER all rules producing T have run to fixpoint. This fixes non-monotonic
;; negation: a rule consuming NOT(T) cannot fire before T is fully derived and
;; thereby leak a spurious derived fact that is never retracted.
;;
;; Standard stratified-datalog algorithm:
;;   1. Assign each produced-type a stratum number (init 0).
;;   2. Iterate: if rule R negates type N, all types R produces must be at
;;      stratum ≥ stratum[N]+1. Repeat until fixpoint or cycle detected.
;;   3. Group rules by stratum ascending → fire each group to fixpoint before
;;      advancing to the next, threading the accumulated facts forward so
;;      higher-stratum rules see the complete lower-stratum derivation.
;;
;; WHY this file: numbering is one reason-to-change (the produced/negated/consumed
;; type graph). Fire-stratified drive stays in fire.wat.
;; WHY fire-fixpoint unchanged: it is correct within a stratum (monotone,
;; finite, no negation-ordering hazard). Stratification is the ordering layer.

;; StratifyAcc — sweep accumulator: current type-strata map + change flag.
;; type-strata: (HashMap :- [String i64]) mapping produced-type FQDN → stratum number.
;; changed: true iff this sweep raised any stratum value.
(:wat::core::defrecord :wat::rete::StratifyAcc
  [type-strata <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
   changed     <- :wat::core::bool])

;; rule-produces — extract produced type-FQDNs (colon-free) from a Rule's RHS.
;; Arc 278 Stone A: each RHS entry IS the fact-form directly (:ProducedType …) — the
;; `:wat::rete::insert` wrapper is gone, so the type head is the first child of `form`
;; itself (no more unwrapping a second child).
(:wat::core::defn :wat::rete::rule-produces
  [rule <- :wat::rete::Rule]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [rhs (:wat::rete::Rule/rhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::core::String])
                       form <- :wat::WatAST]
        -> (:wat::core::PersistentVector :- [:wat::core::String])
        (:wat::core::let [fact-ch   (:wat::core::ast->children form)
                          type-hd   (:wat::core::first fact-ch)
                          raw-nm    (:wat::core::ast-name type-hd)
                          ;; strip leading colon → bare FQDN matching (:wat::core::type fact)
                          type-nm   (:wat::core::if (:wat::core::= (:wat::string::subs raw-nm 0 1) ":")
                                      (:wat::string::subs raw-nm 1 (:wat::string::length raw-nm))
                                      raw-nm)]
          (:wat::core::PersistentVector/conj acc type-nm)))
      (:wat::core::PersistentVector)
      rhs)))

;; type-name-of — colon-stripped fact-type head, or None for engine forms / ?var.
(:wat::core::defn :wat::rete::type-name-of
  [form <- :wat::WatAST] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::empty? ch)
      :wat::core::None
      (:wat::core::let [raw (:wat::core::ast-name (:wat::core::first ch))
                        n   (:wat::string::length raw)
                        q?  (:wat::core::if (:wat::core::i64::>= n 1)
                              (:wat::core::= (:wat::string::subs raw 0 1) "?")
                              false)
                        rete? (:wat::core::if (:wat::core::i64::>= n 12)
                                (:wat::core::= (:wat::string::subs raw 0 12) ":wat::rete::")
                                false)]
        (:wat::core::if (:wat::core::if q? true rete?)
          :wat::core::None
          (:wat::core::Some
            (:wat::core::if (:wat::core::= (:wat::string::subs raw 0 1) ":")
              (:wat::string::subs raw 1 n)
              raw)))))))

;; negated-types-under — leaves under :not, including :and/:or combinators.
(:wat::core::defn :wat::rete::negated-types-under
  [form <- :wat::WatAST] -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children form)
                    hd (:wat::core::if (:wat::core::empty? ch)
                         ""
                         (:wat::core::ast-name (:wat::core::first ch)))]
    (:wat::core::if (:wat::core::if (:wat::core::= hd ":wat::rete::and")
                      true
                      (:wat::core::= hd ":wat::rete::or"))
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                         kid <- :wat::WatAST]
          -> (:wat::core::PersistentVector :- [:wat::core::String])
          (:wat::core::foldl
            (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::core::String])
                             t <- :wat::core::String]
              -> (:wat::core::PersistentVector :- [:wat::core::String])
              (:wat::core::PersistentVector/conj a t))
            acc
            (:wat::rete::negated-types-under kid)))
        (:wat::core::PersistentVector)
        (:wat::core::rest ch))
      (:wat::core::if (:wat::core::= hd ":wat::rete::not")
        (:wat::rete::negated-types-under (:wat::core::second ch))
        (:wat::core::match (:wat::rete::type-name-of form)
          ((:wat::core::Some n) (:wat::core::PersistentVector n))
          (:wat::core::None (:wat::core::PersistentVector)))))))

;; rule-negates — :not of a fact AND :not of :and/:or. Leaves, not "wat::rete::and".
(:wat::core::defn :wat::rete::rule-negates
  [rule <- :wat::rete::Rule]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [lhs (:wat::rete::Rule/lhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::core::String])
                       form <- :wat::WatAST]
        -> (:wat::core::PersistentVector :- [:wat::core::String])
        (:wat::core::let [ch (:wat::core::ast->children form)
                          hd (:wat::core::if (:wat::core::empty? ch)
                               ""
                               (:wat::core::ast-name (:wat::core::first ch)))]
          (:wat::core::if (:wat::core::= hd ":wat::rete::not")
            (:wat::core::foldl
              (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::core::String])
                               t <- :wat::core::String]
                -> (:wat::core::PersistentVector :- [:wat::core::String])
                (:wat::core::PersistentVector/conj a t))
              acc
              (:wat::rete::negated-types-under (:wat::core::second ch)))
            acc)))
      (:wat::core::PersistentVector)
      lhs)))

;; stratify-sweep — one pass over all rules updating type-strata.
;; For each rule: required = max(stratum[n]+1 for n in negated, default 0).
;; For each produced type p: stratum[p] = max(stratum[p], required).
;; Returns StratifyAcc{updated type-strata, changed flag (true if any stratum rose)}.
;; rule-consumes — the fact types a rule reads POSITIVELY (task #94).
;;
;; The stratifier needs this and did not have it. Correct stratification requires BOTH
;;   stratum(r) >= stratum(p)  for every p used POSITIVELY
;;   stratum(r) >  stratum(p)  for every p NEGATED
;; Only the second was implemented, so a rule consuming a fact produced in a HIGHER stratum
;; was left in a LOWER one, fired to fixpoint before its input existed, and never re-fired.
;;
;; Engine forms :not / :where are NOT positive reads. :exists inner and
;; accumulate :from ARE — lockstep with native `rule_consumes`. A `?n`
;; accumulate head is not a type.
(:wat::core::defn :wat::rete::rule-consumes
  [rule <- :wat::rete::Rule]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [lhs (:wat::rete::Rule/lhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::core::String])
                       form <- :wat::WatAST]
        -> (:wat::core::PersistentVector :- [:wat::core::String])
        (:wat::core::let [ch (:wat::core::ast->children form)
                          hd (:wat::core::if (:wat::core::empty? ch)
                               ""
                               (:wat::core::ast-name (:wat::core::first ch)))
                          n  (:wat::string::length hd)
                          q? (:wat::core::if (:wat::core::i64::>= n 1)
                               (:wat::core::= (:wat::string::subs hd 0 1) "?")
                               false)]
          (:wat::core::if (:wat::core::= hd ":wat::rete::exists")
            (:wat::core::match (:wat::rete::type-name-of (:wat::core::second ch))
              ((:wat::core::Some t) (:wat::core::PersistentVector/conj acc t))
              (:wat::core::None acc))
            (:wat::core::if (:wat::core::if q?
                              (:wat::core::if (:wat::core::i64::>= (:wat::core::length ch) 5)
                                (:wat::core::= (:wat::core::ast-name
                                                 (:wat::core::Option/expect
                                                   (:wat::core::get ch 3)
                                                   "rule-consumes: acc :from"))
                                  ":from")
                                false)
                              false)
              (:wat::core::match (:wat::rete::type-name-of
                                   (:wat::core::Option/expect
                                     (:wat::core::get ch 4)
                                     "rule-consumes: acc :from inner"))
                ((:wat::core::Some t) (:wat::core::PersistentVector/conj acc t))
                (:wat::core::None acc))
              (:wat::core::if (:wat::core::if (:wat::core::i64::>= n 12)
                                (:wat::core::= (:wat::string::subs hd 0 12) ":wat::rete::")
                                false)
                acc
                (:wat::core::match (:wat::rete::type-name-of form)
                  ((:wat::core::Some t) (:wat::core::PersistentVector/conj acc t))
                  (:wat::core::None acc)))))))
      (:wat::core::PersistentVector)
      lhs)))

(:wat::core::defn :wat::rete::stratify-sweep
  [rules       <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   type-strata <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])]
  -> :wat::rete::StratifyAcc
  (:wat::core::foldl
    (:wat::core::fn [acc  <- :wat::rete::StratifyAcc
                     rule <- :wat::rete::Rule]
      -> :wat::rete::StratifyAcc
      (:wat::core::let [ts       (:wat::rete::StratifyAcc/type-strata acc)
                        changed  (:wat::rete::StratifyAcc/changed acc)
                        produced (:wat::rete::rule-produces rule)
                        negated  (:wat::rete::rule-negates rule)
                        consumed (:wat::rete::rule-consumes rule)
                        ;; req-neg = max(stratum[n]+1 for n in negated, default 0)
                        req-neg  (:wat::core::foldl
                                   (:wat::core::fn [mx  <- :wat::core::i64
                                                    neg <- :wat::core::String]
                                     -> :wat::core::i64
                                     (:wat::core::let [ns (:wat::core::match
                                                             (:wat::core::HashMap/get ts neg)
                                                             
                                                           ((:wat::core::Some v) v)
                                                           (:wat::core::None 0))
                                                       v  (:wat::core::i64::+ ns 1)]
                                       (:wat::core::if (:wat::core::i64::> v mx) v mx)))
                                   0
                                   negated)
                        ;; req-pos = max(stratum[c] for c in consumed, default 0) — task #94.
                        ;; NOT +1: a positive consumer may sit in the SAME stratum as its input
                        ;; (that is ordinary forward chaining); it merely may not sit BELOW it.
                        req-pos  (:wat::core::foldl
                                   (:wat::core::fn [mx  <- :wat::core::i64
                                                    con <- :wat::core::String]
                                     -> :wat::core::i64
                                     (:wat::core::let [cs (:wat::core::match
                                                             (:wat::core::HashMap/get ts con)
                                                           ((:wat::core::Some v) v)
                                                           (:wat::core::None 0))]
                                       (:wat::core::if (:wat::core::i64::> cs mx) cs mx)))
                                   0
                                   consumed)
                        required (:wat::core::if (:wat::core::i64::> req-neg req-pos) req-neg req-pos)
                        ;; for each produced type: raise stratum to required if higher
                        new-acc  (:wat::core::foldl
                                   (:wat::core::fn [inner <- :wat::rete::StratifyAcc
                                                    p     <- :wat::core::String]
                                     -> :wat::rete::StratifyAcc
                                     (:wat::core::let [its (:wat::rete::StratifyAcc/type-strata inner)
                                                       ich (:wat::rete::StratifyAcc/changed inner)
                                                       cur (:wat::core::match
                                                              (:wat::core::HashMap/get its p)
                                                              
                                                            ((:wat::core::Some v) v)
                                                            (:wat::core::None 0))]
                                       (:wat::core::if (:wat::core::i64::> required cur)
                                         (:wat::rete::StratifyAcc
                                           :type-strata (:wat::core::HashMap/assoc its p required)
                                           :changed true)
                                         inner)))
                                   (:wat::rete::StratifyAcc :type-strata ts :changed changed)
                                   produced)]
        new-acc))
    (:wat::rete::StratifyAcc :type-strata type-strata :changed false)
    rules))

;; stratify-fix — recursive fixpoint for stratification.
;; Sweeps until no stratum changes (converged) or remaining iterations run out.
;; Raises on negation cycle: rule set is not stratifiable (non-terminating strata).
(:wat::core::defn :wat::rete::stratify-fix
  [rules       <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   type-strata <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
   remaining   <- :wat::core::i64]
  -> (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
  (:wat::core::let [result  (:wat::rete::stratify-sweep rules type-strata)
                    changed (:wat::rete::StratifyAcc/changed result)
                    new-ts  (:wat::rete::StratifyAcc/type-strata result)]
    (:wat::core::if (:wat::core::not changed)
      new-ts
      ;; still changing — check for cycle before recursing
      (:wat::core::let [_cycle (:wat::core::Option/expect
                                  (:wat::core::if (:wat::core::i64::> remaining 0)
                                    (:wat::core::Some nil)
                                    :wat::core::None)
                                  "stratify: negation cycle detected — rule set is not stratifiable")]
        (:wat::rete::stratify-fix rules new-ts (:wat::core::i64::- remaining 1))))))

;; rule-stratum — compute the stratum of one rule given the final type-strata.
;; = max(max strata[p] for produced p, max strata[n]+1 for negated n).
(:wat::core::defn :wat::rete::rule-stratum
  [rule        <- :wat::rete::Rule
   type-strata <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [produced (:wat::rete::rule-produces rule)
                    negated  (:wat::rete::rule-negates rule)
                    from-p   (:wat::core::foldl
                               (:wat::core::fn [mx <- :wat::core::i64
                                                p  <- :wat::core::String]
                                 -> :wat::core::i64
                                 (:wat::core::let [ps (:wat::core::match
                                                         (:wat::core::HashMap/get type-strata p)
                                                         
                                                       ((:wat::core::Some v) v)
                                                       (:wat::core::None 0))]
                                   (:wat::core::if (:wat::core::i64::> ps mx) ps mx)))
                               0
                               produced)
                    from-n   (:wat::core::foldl
                               (:wat::core::fn [mx <- :wat::core::i64
                                                n  <- :wat::core::String]
                                 -> :wat::core::i64
                                 (:wat::core::let [ns (:wat::core::match
                                                         (:wat::core::HashMap/get type-strata n)
                                                         
                                                       ((:wat::core::Some v) v)
                                                       (:wat::core::None 0))
                                                   v  (:wat::core::i64::+ ns 1)]
                                   (:wat::core::if (:wat::core::i64::> v mx) v mx)))
                               0
                               negated)]
    (:wat::core::if (:wat::core::i64::> from-n from-p) from-n from-p)))

;; stratify — compute the type→stratum HashMap for a rule set.
;; Returns (HashMap :- [String i64]) mapping each produced-type FQDN to its stratum number.
;; Raises "negation cycle" if the rule set is not stratifiable (cyclic negation dependency).
(:wat::core::defn :wat::rete::stratify
  [rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
  (:wat::core::let [init-ts (:wat::core::HashMap :wat::core::String :wat::core::i64)
                    ;; length(rules)+1 sweeps is always enough for a stratifiable set
                    bound   (:wat::core::i64::+ (:wat::core::length rules) 1)]
    (:wat::rete::stratify-fix rules init-ts bound)))
