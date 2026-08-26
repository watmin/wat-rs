;; wat-scripts/perf/grid/user-reduce.wat — GRID AXIS A6: user reducers (custom accumulator), IN WAT.
;;
;; The A6 capability (DESIGN-STONE-8-custom.md, wat/rete.wat:665+ accumulate 8-custom fence +
;; :2272 accumulate-pass-for-token): the accumulate slot accepts ANY pure∧det user wat fn
;; `((PersistentVector :- [T])) -> R` as the acc-form head — the dispatcher gathers the bound `?var`
;; values into a (PV :- [T]) and folds the user fn over it. Here the fold is `sum-of-squares`
;; (the repo's own differential exemplar, tests/rete/probe_arc278_8custom_native_differential.rs):
;;   sum-of-squares(xs) = Σ x² .
;;
;; WORKLOAD (mirrors strat-neg.wat's build/seed/derive/emit shape):
;;   Station(loc)          — L seed facts, loc in [0, locs).
;;   Reading(loc, value)   — R readings per station, value = (loc + j) mod 7, j in [0, reads).
;;   Agg(loc, s) :- Station(loc) AND (?s <- sum-of-squares(?v) :from Reading(loc, ?v))
;;                         — one derived Agg per location; s is the population Σ v² of that
;;                           location's gathered readings, computed by the USER fold over the
;;                           whole gathered (PV :- [i64]) (batch fold; re-run each fire).
;;
;; WHY sum-of-squares (not a percentile/top-k): CLARA-TRANSLATIONS.md A6 (c) — wat's custom fold is
;; a BATCH fold over (PV :- [T]); Clara's `accum` is a STREAMING reduce (reduce-fn + optional retract-fn).
;; The two SHAPES differ, so this axis compares FINAL RESULTS only. sum-of-squares is
;; incrementally decomposable (Σx² over insert = O(1) update; Σx² over a batch PV = O(n) re-fold) —
;; so BOTH engines compute the IDENTICAL final population value. That equivalence is the point of
;; this axis; a non-incremental fold (percentile) would make the streaming/batch gap expected and
;; is deliberately NOT used here (would be a STOP-and-surface, not a match).
;;
;; :derived is the FULL SORTED per-location aggregate set, each Agg canonicalized as ONE i64
;; (loc * 1,000,000 + s). s < 1,000,000 for every size this axis runs at (max s = reads * 36, and
;; grid `reads` stays far below 27,000), so the encoding is injective — it compares byte-for-byte
;; against Clara's rendering of the identical workload (gen-user-reduce.sh).
;;
;; Fires the NATIVE production verb :wat::rete::fire-rules (the differential-tested fast path).
;;
;; Usage (stdin = an i64 vector [locs reads]; stdout = one #grid/Result EDN line):
;;   echo '[20 50]' | cargo wat ./wat-scripts/perf/grid/user-reduce.wat
;;   => #grid/Result {:axis "user-reduce" :size [20 50] :derived [...] :native-ns N}

(:wat::core::defrecord :ur::Station [loc <- :wat::core::i64])
(:wat::core::defrecord :ur::Reading [loc <- :wat::core::i64  value <- :wat::core::i64])
(:wat::core::defrecord :ur::Agg [loc <- :wat::core::i64  sos <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

;; the USER custom fold: Σ x² over the whole gathered vector — pure∧det (passes the 8-custom fence).
;; Identical to the repo differential exemplar (probe_arc278_8custom_native_differential.rs:26-30).
(:wat::rete::core::defn :ur::sum-of-squares [xs <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::rete::core::foldl
    (:wat::rete::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
      ;; sum-of-squares over readings is always >= 0 (a square is never negative, and a sum of
      ;; non-negative squares is never negative) — -1 is impossible as a legitimate result at
      ;; either op, so it cannot be confused with a real value if the undefined point is ever hit.
      (:wat::rete::i64::+ acc (:wat::rete::i64::* x x :undefined -1) :undefined -1))
    0 xs))

;; mod7 n — n mod 7 via (n - (n/7)*7); wat has no i64::mod, and strat-neg.wat uses this same
;; div/mul/sub identity for its mod-2 test. Keeps reading values in [0,7) so Σx² stays small.
(:wat::core::defn :ur::mod7 [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::- n (:wat::i64::* (:wat::i64::/ n 7) 7)))

;; encode loc s — canonical single-i64 witness for one derived Agg(loc, s) fact.
(:wat::core::defn :ur::encode [loc <- :wat::core::i64  s <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ (:wat::i64::* loc 1000000) s))

;; the ONE rule: per Station, fold the user sum-of-squares over that location's Readings → Agg.
(:wat::rete::defrule :ur::flag
  :when
  [(:ur::Station (?loc <- :loc))
   (?s <- (:ur::sum-of-squares ?v) :from (:ur::Reading (?loc <- :loc) (?v <- :value)))]
  :then
  [(:ur::Agg ?loc ?s)])

(:wat::rete::defquery :ur::q-Agg
  :params []
  :when [(?fact <- :ur::Agg)])


;; seed-loc session loc reads — stage Station(loc) then Reading(loc, (loc+j) mod 7) for j in [0,reads).
;; loc-facts acc loc reads — Station(loc) then its `reads` Readings, appended to a FACT VECTOR.
;; No longer threads a Session: staging is one BATCH `insert-all` at the end of `seed-all`.
(:wat::core::defn :ur::loc-facts
  [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  loc <- :wat::core::i64  reads <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::core::Record])  j <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:wat::vector::conj a (:ur::Reading :loc loc :value (:ur::mod7 (:wat::i64::+ loc j)))))
    (:wat::vector::conj acc (:ur::Station loc))
    (:wat::core::range 0 reads)))

;; seed-all session locs reads — stage every location's Station + Reading block.
(:wat::core::defn :ur::seed-all
  [session <- :wat::rete::Session  locs <- :wat::core::i64  reads <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  loc <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:ur::loc-facts acc loc reads))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 locs))))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :ur::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every derived Agg fact, canonically encoded and sorted ascending. THE
;; accuracy witness: the full per-location aggregate set (a wrong Σx² anywhere shows up).
(:wat::core::defn :ur::derived-vector
  [fired <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [codes (:wat::core::into (:wat::core::Vector :wat::core::i64)
                           (:wat::core::map
                             (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:ur::encode (:ur::Agg/loc f) (:ur::Agg/sos f))))
                             (:wat::rete::query fired (:ur::q-Agg))))]
    (:ur::vec->pvec (:wat::core::sort codes))))

;; ns-between t0 t1 — nanoseconds between two Instants (mirrors strat-neg.wat's ns-between).
(:wat::core::defn :ur::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    locs    (:wat::core::Option/expect  (:wat::core::get params 0) "stdin: [locs reads]")
                    reads   (:wat::core::Option/expect  (:wat::core::get params 1) "stdin: [locs reads]")
                    rules   (:wat::rete::collect-rules :ur)
                    staged  (:ur::seed-all (:wat::rete::compile-all rules (:wat::core::PersistentVector (:ur::q-Agg))) locs reads)
                    ;; time the NATIVE production verb only (compile + seed are un-timed setup)
                    n0      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    n1      (:wat::time::now)
                    derived (:ur::derived-vector fired)
                    nat-ns  (:ur::ns-between n0 n1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::rete::fire-rules$oracle staged)
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "user-reduce" :size (:wat::core::PersistentVector locs reads) :derived derived :native-ns nat-ns :oracle-derived (:ur::derived-vector ofired) :oracle-ns (:ur::ns-between o0 o1)))))
