;; wat-scripts/perf/grid/negation.wat — GRID AXIS A3: plain :not negation, IN WAT.
;;
;; The SINGLE-STRATUM negation axis of the Clara grid
;; (docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md): a plain `(:wat::rete::not ...)` in a
;; rule LHS at scale — distinct from the FOUNDATION strat-neg.wat axis (an N-deep chain of
;; negations-over-DERIVED facts). Here the negated type is an INPUT fact (no producer rule), so
;; there is exactly ONE rule and ONE non-trivial stratum — the truest "plain :not" workload.
;;
;; Shape (mirrors wat-scripts/fixes/rete-truth-maintenance-probes/neg.clj, with Bad seeded as an
;; input fact rather than derived, so the negation is single-stratum):
;;   Item(k)  — M seed facts, k in [0, items).
;;   Bad(k)   — seeded for every EVEN k in [0, items).  [input, no producer rule]
;;   Ok(k)    :- Item(k) AND NOT Bad(k)                 [the ONE negation rule]
;; ⇒ Ok fires for exactly the ODD k in [0, items). `items` (M) is the single scale dial.
;;
;; The wat NegationNode (wat/rete.wat:109-112) passes a token iff ZERO elements in the negated
;; alpha-memory (Bad) are compatible with its bindings. The stratifier (wat/rete.wat:1707+) places
;; the Ok rule one stratum above the base Bad facts; `:wat::rete::fire-rules` (native, the
;; differential-tested fast path) resolves it in a single batch fixpoint.
;;
;; CAVEAT (from CLARA-TRANSLATIONS.md A3): Clara resolves the SAME final set via INCREMENTAL truth
;; maintenance (a Bad insert live-retracts any already-fired Ok token for that ?k), NOT a stratified
;; batch fixpoint. Same answer, different mechanism — the speed ratio reflects that, not raw
;; superiority. Accuracy MUST match.
;;
;; :derived is the FULL SORTED set of Ok keys (single fact type ⇒ the canonical witness is just k),
;; comparable byte-for-byte against Clara's rendering of the identical workload (gen-negation.sh).
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[1000]' | cargo wat ./wat-scripts/perf/grid/negation.wat
;;   => #grid/Result {:axis "negation" :size [1000] :derived [1 3 5 ...] :native-ns N}

(:wat::core::defrecord :neg::Item [k <- :wat::core::i64])
(:wat::core::defrecord :neg::Bad  [k <- :wat::core::i64])
(:wat::core::defrecord :neg::Ok   [k <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- :wat::core::PersistentVector<wat::core::i64>
   derived   <- :wat::core::PersistentVector<wat::core::i64>
   native-ns <- :wat::core::i64])

;; build-rules — the single-rule set: Ok(k) :- Item(k) AND NOT Bad(k).
;; Two LHS conditions: bind ?k off Item, then negate Bad on the same ?k. One RHS insert.
(:wat::core::defn :neg::build-rules [] -> :wat::core::PersistentVector<wat::rete::Rule>
  (:wat::core::PersistentVector
    (:wat::rete::Rule "ok"
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (:neg::Item (?k <- :k)))
        (:wat::core::quasiquote (:wat::rete::not (:neg::Bad (?k <- :k)))))
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (:wat::rete::insert (:neg::Ok ?k)))))))

;; seed session items — stage Item(i) for every i in [0, items), plus Bad(i) for every EVEN i,
;; threading the staging session (mirrors strat-neg.wat's seed-items, with the extra Bad insert).
(:wat::core::defn :neg::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::core::let [s2 (:wat::rete::insert s (:neg::Item i))]
        (:wat::core::if (:wat::core::= i (:wat::core::i64::* (:wat::core::i64::/ i 2) 2))
          (:wat::rete::insert s2 (:neg::Bad i))
          s2)))
    session
    (:wat::core::range 0 items)))

;; vec->pvec v — materialize a Vector<i64> into a PersistentVector<i64> (same honest conj-fold
;; bridge strat-neg.wat uses; `into` has no (PersistentVector<T>, Vector<T>) clause).
(:wat::core::defn :neg::vec->pvec [v <- :wat::core::Vector<wat::core::i64>] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::i64>  x <- :wat::core::i64]
      -> :wat::core::PersistentVector<wat::core::i64>
      (:wat::core::PersistentVector/conj acc x))
    (:wat::core::PersistentVector)
    v))

;; derived-vector fired — every derived Ok fact's key, sorted ascending. This IS the accuracy
;; witness: a missing/extra Ok anywhere shows up in the byte-for-byte compare.
(:wat::core::defn :neg::derived-vector [fired <- :wat::rete::Session] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::let [codes (:wat::core::into (:wat::core::Vector :wat::core::i64)
                            (:wat::core::map
                              (:wat::core::fn [f <- :neg::Ok] -> :wat::core::i64 (:neg::Ok/k f))
                              (:wat::rete::query-by-type-string fired "neg::Ok")))]
    (:neg::vec->pvec (:wat::core::sort codes))))

;; ns-between t0 t1 — nanoseconds between two Instants (mirrors strat-neg.wat's ns-between).
(:wat::core::defn :neg::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::kernel::readln -> :wat::core::Vector<wat::core::i64>)
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    rules   (:neg::build-rules)
                    staged  (:neg::seed (:wat::rete::compile rules) items)
                    ;; time the NATIVE production verb only (compile + seed are un-timed setup)
                    n0      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    n1      (:wat::time::now)
                    derived (:neg::derived-vector fired)
                    nat-ns  (:neg::ns-between n0 n1)]
    (:wat::kernel::println
      (:grid::Result "negation" (:wat::core::PersistentVector items) derived nat-ns))))
