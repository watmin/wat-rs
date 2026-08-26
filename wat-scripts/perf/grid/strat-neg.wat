;; wat-scripts/perf/grid/strat-neg.wat — GRID AXIS A4: stratified negation, IN WAT.
;;
;; The FOUNDATION axis of the Clara grid (docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md):
;; non-monotonic negation over DERIVED facts, at scale — the R18 capability
;; (wat/rete.wat:1513+ StratifyAcc/fire-stratified), stressed N strata deep x M items wide.
;;
;; WHY N DISTINCT record types (S0..S9), NOT deep-cascade.wat's single-type-with-a-level-field
;; trick: `stratify` (wat/rete.wat:1570 rule-negates + :1598 stratify-sweep) reasons about
;; dependencies at the TYPE-FQDN level — "rule R negates type T" ⇒ "T's producers must be a
;; STRICTLY LOWER stratum than R". A single "State{stratum,k}" type would be BOTH produced AND
;; negated by the SAME type across every level (rule i produces State, rule i+1 negates State)
;; → the stratifier sees an unresolvable self-cycle ("negation cycle detected — rule set is not
;; stratifiable"), confirmed empirically before this file settled on 10 distinct types. This is
;; a real, load-bearing constraint of the stratifier, not a workaround — n3 (the proven 3-strata
;; fixture at tests/rete/probe_arc278_7strat_native_differential.wat) already uses 3 distinct
;; types (Bad/Warn/Safe) for exactly this reason; S0..S9 is that same shape generalized to 10.
;;
;; Shape (mirrors n3's Bad/Warn/Safe chain, generalized):
;;   Item(k)  — M seed facts, k in [0, items).
;;   S0(k)    :- Item(k) AND (k mod 2 == 0)     [stratum 0: the base marking rule]
;;   Si(k)    :- Item(k) AND NOT S(i-1)(k)      [stratum i>0: negation over the PRIOR
;;                                                stratum's derived facts — only correct if
;;                                                S(i-1)'s closure is known first]
;; MAX_STRATA = 10 (S0..S9) is a static ceiling from the pre-declared types; `items` (M) is the
;; free scale dial (the derived-set-size driver), `strata` (N) proves genuine chain depth up to
;; the ceiling — matching the axis note "N strata x M items so the derived set is large".
;;
;; Fires the NATIVE production verb `:wat::rete::fire-rules` (native stratification,
;; arc278 stone 7-strat-native — the differential-tested fast path; NOT
;; `fire-rules$oracle`).
;;
;; :derived is the FULL SORTED derived-fact set, canonicalized as a single i64 per fact
;; (stratum * 1,000,000 + k) so it can be compared byte-for-byte against Clara's rendering of the
;; identical workload (gen-strat-neg.sh) — no record/keyword shape to reconcile, just a sorted
;; vector of integers on both sides.
;;
;; Usage (stdin = an i64 vector [strata items]; stdout = one #grid/Result EDN line):
;;   echo '[6 2000]' | cargo wat ./wat-scripts/perf/grid/strat-neg.wat
;;   => #grid/Result {:axis "strat-neg" :size [6 2000] :derived [...] :native-ns N}

(:wat::core::defrecord :strat::Item [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S0 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S3 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S4 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S5 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S6 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S7 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S8 [k <- :wat::core::i64])
(:wat::core::defrecord :strat::S9 [k <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :strat::q-S0
  :params []
  :when [(?fact <- :strat::S0)])


(:wat::rete::defquery :strat::q-S1
  :params []
  :when [(?fact <- :strat::S1)])


(:wat::rete::defquery :strat::q-S2
  :params []
  :when [(?fact <- :strat::S2)])


(:wat::rete::defquery :strat::q-S3
  :params []
  :when [(?fact <- :strat::S3)])


(:wat::rete::defquery :strat::q-S4
  :params []
  :when [(?fact <- :strat::S4)])


(:wat::rete::defquery :strat::q-S5
  :params []
  :when [(?fact <- :strat::S5)])


(:wat::rete::defquery :strat::q-S6
  :params []
  :when [(?fact <- :strat::S6)])


(:wat::rete::defquery :strat::q-S7
  :params []
  :when [(?fact <- :strat::S7)])


(:wat::rete::defquery :strat::q-S8
  :params []
  :when [(?fact <- :strat::S8)])


(:wat::rete::defquery :strat::q-S9
  :params []
  :when [(?fact <- :strat::S9)])


;; encode stratum k — canonical single-i64 witness for one derived S<n> fact.
;; items is always far below 1,000,000 in every size this axis is run at (grid scale, not
;; production scale), so the encoding is injective for the sizes this ward ever sees.
(:wat::core::defn :strat::encode [stratum <- :wat::core::i64  k <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ (:wat::i64::* stratum 1000000) k))

;; insert-form lvl — the full (:wat::rete::insert (:strat::S<lvl> ?k)) action form for stratum
;; lvl. Each branch is a LITERAL nested quasiquote (no cross-boundary AST splicing of a computed
;; sub-form into a type-name position — that path is unproven/risky); this dispatch is the one
;; place the MAX_STRATA=10 ceiling is enforced (the :else branch raises).
(:wat::core::defn :strat::insert-form [lvl <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::cond
    ((:wat::core::= lvl 0) (:wat::core::quasiquote (:strat::S0 ?k)))
    ((:wat::core::= lvl 1) (:wat::core::quasiquote (:strat::S1 ?k)))
    ((:wat::core::= lvl 2) (:wat::core::quasiquote (:strat::S2 ?k)))
    ((:wat::core::= lvl 3) (:wat::core::quasiquote (:strat::S3 ?k)))
    ((:wat::core::= lvl 4) (:wat::core::quasiquote (:strat::S4 ?k)))
    ((:wat::core::= lvl 5) (:wat::core::quasiquote (:strat::S5 ?k)))
    ((:wat::core::= lvl 6) (:wat::core::quasiquote (:strat::S6 ?k)))
    ((:wat::core::= lvl 7) (:wat::core::quasiquote (:strat::S7 ?k)))
    ((:wat::core::= lvl 8) (:wat::core::quasiquote (:strat::S8 ?k)))
    ((:wat::core::= lvl 9) (:wat::core::quasiquote (:strat::S9 ?k)))
    (:else (:wat::core::Option/expect  :wat::core::None
             (:wat::string::interpolate
               "strat-neg: strata exceeds MAX_STRATA=10 (S0..S9); requested level {lvl-s}"
               :lvl-s (:wat::i64::to-string lvl))))))

;; not-pattern prev — the full (:wat::rete::not (:strat::S<prev> (?k <- :k))) condition form,
;; negating stratum `prev`'s derived facts. Same literal-dispatch shape as insert-form.
(:wat::core::defn :strat::not-pattern [prev <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::cond
    ((:wat::core::= prev 0) (:wat::core::quasiquote (:wat::rete::not (:strat::S0 (?k <- :k)))))
    ((:wat::core::= prev 1) (:wat::core::quasiquote (:wat::rete::not (:strat::S1 (?k <- :k)))))
    ((:wat::core::= prev 2) (:wat::core::quasiquote (:wat::rete::not (:strat::S2 (?k <- :k)))))
    ((:wat::core::= prev 3) (:wat::core::quasiquote (:wat::rete::not (:strat::S3 (?k <- :k)))))
    ((:wat::core::= prev 4) (:wat::core::quasiquote (:wat::rete::not (:strat::S4 (?k <- :k)))))
    ((:wat::core::= prev 5) (:wat::core::quasiquote (:wat::rete::not (:strat::S5 (?k <- :k)))))
    ((:wat::core::= prev 6) (:wat::core::quasiquote (:wat::rete::not (:strat::S6 (?k <- :k)))))
    ((:wat::core::= prev 7) (:wat::core::quasiquote (:wat::rete::not (:strat::S7 (?k <- :k)))))
    ((:wat::core::= prev 8) (:wat::core::quasiquote (:wat::rete::not (:strat::S8 (?k <- :k)))))
    ((:wat::core::= prev 9) (:wat::core::quasiquote (:wat::rete::not (:strat::S9 (?k <- :k)))))
    (:else (:wat::core::Option/expect  :wat::core::None
             (:wat::string::interpolate
               "strat-neg: strata exceeds MAX_STRATA=10 (S0..S9); requested level {prev-s}"
               :prev-s (:wat::i64::to-string prev))))))

;; build-rule lvl — the lvl-th stratum's rule.
;;   lvl == 0: S0(k) :- Item(k) AND (k mod 2 == 0)     [2 conditions: bind, then a :where test]
;;   lvl >  0: Slvl(k) :- Item(k) AND NOT S(lvl-1)(k)  [2 conditions: bind + negate]
;; WHY the mod-2 test is a SEPARATE (:wat::rete::where <expr>) condition, NOT embedded as an
;; extra child of the Item fact pattern (as deep-cascade.wat embeds its level-equality test):
;; alpha-level tests (matcher.rs `eval_clause`/`resolve_binary_operands`) resolve operands ONLY
;; from {bindings, fact-field, bare-literal} — NEVER by evaluating a compound expression. Deep-
;; cascade's embedded `(= ?l (unquote prev))` works because `prev` splices to a bare i64
;; LITERAL; `(i64::* (i64::/ ?k 2) 2)` is a compound expression, which resolve_operand cannot
;; resolve — embedding it silently made the alpha test unsatisfiable for every fact (found
;; empirically: S0 came back EMPTY). `:wat::rete::where` compiles to a beta-level TestNode
;; (compile-condition's where-branch, wat/rete.wat:547+) whose `eval-test` does a genuine
;; `eval_inner` with the bound vars in a child Environment — full expressions are fine there.
;; NOTE: not-pattern is only ever CALLED inside the else-branch of the `if` below (`if` branches
;; are lazily evaluated — the untaken branch never runs), NOT hoisted into a `let` binding —
;; a `let` binding evaluates eagerly regardless of which branch of `conds` gets picked, which
;; would call `(not-pattern -1)` for lvl=0 and panic on the MAX_STRATA guard.
(:wat::core::defn :strat::build-rule [lvl <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [item-c  (:wat::core::quasiquote (:strat::Item (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::rete::core::i64::= ?k
                                  (:wat::rete::core::i64::* (:wat::rete::core::i64::/ ?k 2 :undefined -1) 2 :undefined -1))))
                    ins     (:strat::insert-form lvl)
                    conds   (:wat::core::if (:wat::core::= lvl 0)
                              (:wat::core::PersistentVector item-c where-c)
                              (:wat::core::PersistentVector item-c (:strat::not-pattern (:wat::i64::- lvl 1))))]
    (:wat::rete::Rule :name (:wat::i64::to-string lvl) :lhs conds :rhs (:wat::core::PersistentVector ins))))

;; build-rules strata — the rule set [rule0 .. rule(strata-1)], folding build-rule over
;; (range 1 strata) atop a seeded rule0 (mirrors deep-cascade.wat's build-rules exactly).
(:wat::core::defn :strat::build-rules [strata <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  lvl <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:strat::build-rule lvl)))
    (:wat::core::PersistentVector (:strat::build-rule 0))
    (:wat::core::range 1 strata)))

;; seed-items session items — stage Item(i) for i in [0, items), threading the staging session.
;; Staged with the BATCH verb — one `insert-all` (native, one rebuild) rather than `insert` x N.
(:wat::core::defn :strat::seed-items [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:strat::Item i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; codes-for-level fired lvl — every derived fact of stratum lvl's type, canonically encoded.
;; Same literal-dispatch shape as insert-form/not-pattern (the type-qualified accessor
;; `:strat::S<n>/k` needs a literal symbol, same reason the other two dispatches are literal);
;; mirrors deep-cascade.wat's count-at-level (typed lambda directly over query-by-type-string).
(:wat::core::defn :strat::codes-for-level
  [fired <- :wat::rete::Session  lvl <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::cond
    ((:wat::core::= lvl 0)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 0 (:strat::S0/k f))))
         (:wat::rete::query fired (:strat::q-S0)))))
    ((:wat::core::= lvl 1)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 1 (:strat::S1/k f))))
         (:wat::rete::query fired (:strat::q-S1)))))
    ((:wat::core::= lvl 2)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 2 (:strat::S2/k f))))
         (:wat::rete::query fired (:strat::q-S2)))))
    ((:wat::core::= lvl 3)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 3 (:strat::S3/k f))))
         (:wat::rete::query fired (:strat::q-S3)))))
    ((:wat::core::= lvl 4)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 4 (:strat::S4/k f))))
         (:wat::rete::query fired (:strat::q-S4)))))
    ((:wat::core::= lvl 5)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 5 (:strat::S5/k f))))
         (:wat::rete::query fired (:strat::q-S5)))))
    ((:wat::core::= lvl 6)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 6 (:strat::S6/k f))))
         (:wat::rete::query fired (:strat::q-S6)))))
    ((:wat::core::= lvl 7)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 7 (:strat::S7/k f))))
         (:wat::rete::query fired (:strat::q-S7)))))
    ((:wat::core::= lvl 8)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 8 (:strat::S8/k f))))
         (:wat::rete::query fired (:strat::q-S8)))))
    ((:wat::core::= lvl 9)
     (:wat::core::into (:wat::core::Vector :wat::core::i64)
       (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:strat::encode 9 (:strat::S9/k f))))
         (:wat::rete::query fired (:strat::q-S9)))))
    (:else (:wat::core::Option/expect  :wat::core::None
             (:wat::string::interpolate
               "strat-neg: strata exceeds MAX_STRATA=10 (S0..S9); requested level {lvl-s}"
               :lvl-s (:wat::i64::to-string lvl))))))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :strat::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired strata — every derived fact across all `strata` levels, canonically
;; encoded and sorted ascending. This IS the accuracy witness: the full set, not a count — a
;; mismatch anywhere (missing/extra fact at any stratum) shows up.
(:wat::core::defn :strat::derived-vector
  [fired <- :wat::rete::Session  strata <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [all (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::i64])  lvl <- :wat::core::i64]
                            -> (:wat::core::Vector :- [:wat::core::i64])
                            (:wat::core::into acc (:strat::codes-for-level fired lvl)))
                          (:wat::core::Vector :wat::core::i64)
                          (:wat::core::range 0 strata))]
    (:strat::vec->pvec (:wat::core::sort all))))

;; ns-between t0 t1 — nanoseconds between two Instants (mirrors deep-cascade.wat's ns-between).
(:wat::core::defn :strat::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    strata  (:wat::core::Option/expect  (:wat::core::get params 0) "stdin: [strata items]")
                    items   (:wat::core::Option/expect  (:wat::core::get params 1) "stdin: [strata items]")
                    rules   (:strat::build-rules strata)
                    staged  (:strat::seed-items (:wat::rete::compile-all rules (:wat::core::PersistentVector (:strat::q-S0) (:strat::q-S1) (:strat::q-S2) (:strat::q-S3) (:strat::q-S4) (:strat::q-S5) (:strat::q-S6) (:strat::q-S7) (:strat::q-S8) (:strat::q-S9))) items)
                    ;; time the NATIVE production verb only (compile + seed are un-timed setup)
                    n0      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    n1      (:wat::time::now)
                    derived (:strat::derived-vector fired strata)
                    nat-ns  (:strat::ns-between n0 n1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::rete::fire-rules$oracle staged)
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "strat-neg" :size (:wat::core::PersistentVector strata items) :derived derived :native-ns nat-ns :oracle-derived (:strat::derived-vector ofired strata) :oracle-ns (:strat::ns-between o0 o1)))))
