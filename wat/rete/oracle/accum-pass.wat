;; wat/rete/oracle/accum-pass.wat — interpreted accumulate-pass.
;;
;; accumulate-pass-for-token / accumulate-pass. Loads after acc.wat (acc::* folds)
;; and pass.wat (tokens-from-parents / extend-token). walk-filter-ids eval-depends
;; on accumulate-pass, so this file loads BEFORE fire.wat.
;;
;; Namespace: :wat::rete::

;; ─── the accumulate dispatch (Stone 8-a) ────────────────────────────────────
;;
;; WHY there is no single `apply-accumulator -> Option<Value>` fn: the wat type system has INVARIANT
;; parametric types — `Option<i64>` is NOT a subtype of `Option<Value>` even though i64 <: Value
;; (STONE-Value's `is_subtype` root rule fires only for `sup == ":wat::core::Value"` Path-to-Path, NOT
;; for `Option<T>` covariance). So the dispatch is inlined per-fold in accumulate-pass-for-token, where
;; each fold's concrete return type is handled directly: bare folds (count/sum/distinct/all/group-by)
;; assoc their result into the token's bindings; Option folds (min/max/mean) match inline (None → drop).

;; accumulate-pass-for-token — apply the acc-form over gathered elements for ONE token.
;; Returns the updated beta-mem: extends the token with result-var → aggregate if the
;; accumulator produces a value, or leaves beta-mem unchanged (drop) if it produces None
;; (empty min/max/mean).
;;
;; DESIGN: Each branch calls a specific acc::* fold and handles that fold's return type
;; directly. Bare folds (count/sum/distinct/all/group-by) always produce a value → assoc
;; into bindings. Option folds (min/max/mean) produce Option<i64> → match on that, then
;; assoc or drop. The PersistentMap/assoc on the BARE bindings PM accepts any value
;; (i64/PV/PM) via STONE-Value UP (i64 <: Value, PV <: Value, PM <: Value).
(:wat::core::defn :wat::rete::accumulate-pass-for-token
  [acc-form   <- :wat::WatAST
   gathered   <- (:wat::core::PersistentVector :- [:wat::rete::Element])
   result-var <- :wat::core::String
   tok        <- :wat::rete::Token
   node-id    <- :wat::core::i64
   bm         <- (:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::rete::Token])])]
  -> (:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::rete::Token])])
  (:wat::core::let [acc-ch (:wat::core::ast->children acc-form)
                    acc-hd (:wat::core::first acc-ch)
                    acc-nm (:wat::core::ast-name acc-hd)
                    ;; helper: extend tok's bindings with result-var → v, append to bm at node-id
                    ;; (inlined below per case to keep each branch's v-type concrete)
                    tok-binds (:wat::rete::Token/bindings tok)
                    tok-matches (:wat::rete::Token/matches tok)]
    (:wat::core::cond
      ;; count — bare i64 result (always); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::count")
       (:wat::core::let [v   (:wat::rete::acc::count gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; sum — bare i64 result (always); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::sum")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: sum missing ?var"))
                         v   (:wat::rete::acc::sum var gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; min — Option<i64>; Some → assoc, None → drop
      ((:wat::core::= acc-nm ":wat::rete::acc::min")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: min missing ?var"))]
         (:wat::core::match (:wat::rete::acc::min var gathered) 
           ((:wat::core::Some v)
            (:wat::rete::append-token bm node-id
              (:wat::rete::Token :matches tok-matches
                :bindings (:wat::core::PersistentMap/assoc tok-binds result-var v))))
           (:wat::core::None bm))))
      ;; max — Option<i64>; Some → assoc, None → drop
      ((:wat::core::= acc-nm ":wat::rete::acc::max")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: max missing ?var"))]
         (:wat::core::match (:wat::rete::acc::max var gathered) 
           ((:wat::core::Some v)
            (:wat::rete::append-token bm node-id
              (:wat::rete::Token :matches tok-matches
                :bindings (:wat::core::PersistentMap/assoc tok-binds result-var v))))
           (:wat::core::None bm))))
      ;; mean — Option<i64>; Some → assoc, None → drop
      ((:wat::core::= acc-nm ":wat::rete::acc::mean")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: mean missing ?var"))]
         (:wat::core::match (:wat::rete::acc::mean var gathered) 
           ((:wat::core::Some v)
            (:wat::rete::append-token bm node-id
              (:wat::rete::Token :matches tok-matches
                :bindings (:wat::core::PersistentMap/assoc tok-binds result-var v))))
           (:wat::core::None bm))))
      ;; distinct — bare PV result (always; empty → []); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::distinct")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: distinct missing ?var"))
                         v   (:wat::rete::acc::distinct var gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; all — bare PV<Record> result (always; empty → []); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::all")
       (:wat::core::let [v   (:wat::rete::acc::all gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; group-by — bare PM result (always; empty → {}); assoc directly
      ((:wat::core::= acc-nm ":wat::rete::acc::group-by")
       (:wat::core::let [var (:wat::core::ast-name
                               (:wat::core::Option/expect  
                                 (:wat::core::get acc-ch 1)
                                 "accumulate-pass-for-token: group-by missing ?var"))
                         v   (:wat::rete::acc::group-by var gathered)
                         nb  (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk)))
      ;; 8-custom — a non-built-in head is a USER fold fn. Gather the ?var values into a
      ;; Vector<i64>, build the call `(user-fn (:wat::core::PersistentVector v0 v1 …))`
      ;; via quasiquote (~acc-hd splices the head; ~@vals splices the literal values into a
      ;; PV constructor), then eval-ast! it. The result (any Value) assocs into the binding.
      ;; The compile fence (compile-condition) has already proven the fn is pure∧det.
      (:else
       (:wat::core::let [var  (:wat::core::ast-name
                                (:wat::core::Option/expect  
                                  (:wat::core::get acc-ch 1)
                                  "accumulate-pass-for-token: custom fold missing ?var"))
                         vals (:wat::rete::acc::gather-vals var gathered)
                         call (:wat::core::quasiquote
                                ((:wat::core::unquote acc-hd)
                                 (:wat::core::PersistentVector
                                   (:wat::core::unquote-splicing vals))))
                         v    (:wat::core::Result/expect  
                                (:wat::eval-ast! call)
                                "accumulate-pass-for-token: custom fold eval failed")
                         nb   (:wat::core::PersistentMap/assoc tok-binds result-var v)
                         ntk  (:wat::rete::Token :matches tok-matches :bindings nb)]
         (:wat::rete::append-token bm node-id ntk))))))

;; acc-operand-keys — `?var` args of the acc-form (`max ?v` → [?v]; count → []).
(:wat::core::defn :wat::rete::acc-operand-keys
  [acc-form <- :wat::WatAST]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children acc-form)
                    n  (:wat::core::length ch)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                       i   <- :wat::core::i64]
        -> (:wat::core::PersistentVector :- [:wat::core::String])
        (:wat::core::let [kid (:wat::core::Option/expect
                                (:wat::core::get ch i)
                                "acc-operand-keys")]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind kid) "symbol")
            (:wat::core::let [nm (:wat::core::ast-name kid)]
              (:wat::core::if (:wat::core::string::starts-with? nm "?")
                (:wat::core::PersistentVector/conj acc nm)
                acc))
            acc)))
      (:wat::core::PersistentVector)
      (:wat::core::range 1 n))))

;; keys-minus — `from` without any name in `drop`.
(:wat::core::defn :wat::rete::keys-minus
  [from <- (:wat::core::PersistentVector :- [:wat::core::String])
   drop <- (:wat::core::PersistentVector :- [:wat::core::String])]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                     k   <- :wat::core::String]
      -> (:wat::core::PersistentVector :- [:wat::core::String])
      (:wat::core::if (:wat::core::PersistentVector/contains? drop k)
        acc
        (:wat::core::PersistentVector/conj acc k)))
    (:wat::core::PersistentVector)
    from))

;; project-group-keys — element's bindings restricted to `keys` (the group key).
(:wat::core::defn :wat::rete::project-group-keys
  [el   <- :wat::rete::Element
   keys <- (:wat::core::PersistentVector :- [:wat::core::String])]
  -> :wat::core::PersistentMap
  (:wat::core::let [eb (:wat::rete::Element/bindings el)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentMap
                       k   <- :wat::core::String]
        -> :wat::core::PersistentMap
        (:wat::core::match (:wat::core::PersistentMap/get eb k)
          ((:wat::core::Some v) (:wat::core::PersistentMap/assoc acc k v))
          (:wat::core::None acc)))
      (:wat::core::PersistentMap)
      keys)))

;; ─── accumulate-pass (Stone 8-a) ────────────────────────────────────────────
;;
;; Fold step: for each AccumulateNode, for each token in beta-memory[parent],
;; gather token-compatible elements from alpha-memory[from-alpha-id], group by
;; `:from` binds that are not already on the token and not acc-form operands
;; (Clara unbound grouping), call accumulate-pass-for-token per group, and
;; append the extended token (or drop it for empty min/max/mean).
;; Empty gather + non-empty group keys: no bag-wide 0 (Clara new-bindings).
;; Runs AFTER hash-join-pass and BEFORE filter-pass (so a :where on ?result sees the binding).
;;
;; STOP-3 resolution: apply-accumulator cannot return Option<Value> because the wat type system
;; has invariant parametric types — Option<i64> is not Option<Value>. The dispatch is inlined
;; in accumulate-pass-for-token where each fold's specific return type is handled directly.
(:wat::core::defn :wat::rete::accumulate-pass
  [network   <- :wat::core::PersistentMap
   alpha-mem <- :wat::core::PersistentMap
   beta-mem  <- :wat::core::PersistentMap
   node-id   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get network node-id)
                             "accumulate-pass: node not found")
                    kind (:wat::rete::node-kind-label node)]
    (:wat::core::if (:wat::core::= kind "AccumulateNode")
      (:wat::core::let [result-var    (:wat::rete::AccumulateNode/result-var    node)
                        acc-form      (:wat::rete::AccumulateNode/acc-form      node)
                        from-alpha-id (:wat::rete::AccumulateNode/from-alpha-id node)
                        tokens        (:wat::rete::tokens-or-empty-seed
                                        network beta-mem node-id)
                        from-els      (:wat::core::match
                                         (:wat::core::PersistentMap/get alpha-mem from-alpha-id)
                                         
                                       ((:wat::core::Some pv) pv)
                                       (:wat::core::None (:wat::core::PersistentVector)))
                        from-alpha    (:wat::core::Option/expect
                                         (:wat::core::PersistentMap/get network from-alpha-id)
                                         "accumulate-pass: from alpha missing")
                        from-cond     (:wat::core::Option/expect
                                         (:wat::core::get (:wat::rete::AlphaNode/tests from-alpha) 0)
                                         "accumulate-pass: from alpha has no cond")
                        from-keys     (:wat::rete::cond-bind-keys from-cond)
                        operand-keys  (:wat::rete::acc-operand-keys acc-form)]
        (:wat::core::foldl
          (:wat::core::fn [bm  <- :wat::core::PersistentMap
                           tok <- :wat::rete::Token]
            -> :wat::core::PersistentMap
            (:wat::core::let [gathered (:wat::core::foldl
                                          (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Element])
                                                           el  <- :wat::rete::Element]
                                            -> (:wat::core::PersistentVector :- [:wat::rete::Element])
                                            (:wat::core::match (:wat::rete::alpha-match-under from-cond
                                                                 (:wat::rete::Element/fact el)
                                                                 (:wat::rete::Token/bindings tok))
                                              ((:wat::core::Some _)
                                               (:wat::core::PersistentVector/conj acc el))
                                              (:wat::core::None acc)))
                                          (:wat::core::PersistentVector)
                                          from-els)
                              tok-keys (:wat::core::foldl
                                          (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                                                           k   <- :wat::core::String]
                                            -> (:wat::core::PersistentVector :- [:wat::core::String])
                                            (:wat::core::PersistentVector/conj acc k))
                                          (:wat::core::PersistentVector)
                                          (:wat::core::PersistentMap/keys
                                            (:wat::rete::Token/bindings tok)))
                              group-keys (:wat::rete::keys-minus
                                           (:wat::rete::keys-minus from-keys tok-keys)
                                           operand-keys)]
              (:wat::core::if (:wat::core::= (:wat::core::length group-keys) 0)
                (:wat::rete::accumulate-pass-for-token
                   acc-form gathered result-var tok node-id bm)
                (:wat::core::if (:wat::core::= (:wat::core::length gathered) 0)
                  bm
                  (:wat::core::let [key-maps
                                    (:wat::rete::distinct-maps
                                      (:wat::core::foldl
                                        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])
                                                         el  <- :wat::rete::Element]
                                          -> (:wat::core::PersistentVector :- [:wat::core::PersistentMap])
                                          (:wat::core::PersistentVector/conj
                                            acc
                                            (:wat::rete::project-group-keys el group-keys)))
                                        (:wat::core::PersistentVector)
                                        gathered))]
                    (:wat::core::foldl
                      (:wat::core::fn [bm2 <- :wat::core::PersistentMap
                                       km  <- :wat::core::PersistentMap]
                        -> :wat::core::PersistentMap
                        (:wat::core::let [group-els
                                          (:wat::core::foldl
                                            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Element])
                                                             el  <- :wat::rete::Element]
                                              -> (:wat::core::PersistentVector :- [:wat::rete::Element])
                                              (:wat::core::if
                                                (:wat::core::PersistentVector/contains?
                                                  (:wat::core::PersistentVector/conj
                                                    (:wat::core::PersistentVector) km)
                                                  (:wat::rete::project-group-keys el group-keys))
                                                (:wat::core::PersistentVector/conj acc el)
                                                acc))
                                            (:wat::core::PersistentVector)
                                            gathered)
                                          km-keys (:wat::core::PersistentMap/keys km)
                                          ext-binds
                                          (:wat::core::foldl
                                            (:wat::core::fn [nb <- :wat::core::PersistentMap
                                                             k  <- :wat::core::String]
                                              -> :wat::core::PersistentMap
                                              (:wat::core::match (:wat::core::PersistentMap/get km k)
                                                ((:wat::core::Some v)
                                                 (:wat::core::PersistentMap/assoc nb k v))
                                                (:wat::core::None nb)))
                                            (:wat::rete::Token/bindings tok)
                                            km-keys)
                                          ext-tok
                                          (:wat::rete::Token
                                            :matches (:wat::rete::Token/matches tok)
                                            :bindings ext-binds)]
                          (:wat::rete::accumulate-pass-for-token
                             acc-form group-els result-var ext-tok node-id bm2)))
                      bm
                      key-maps))))))
          beta-mem
          tokens))
      beta-mem)))

