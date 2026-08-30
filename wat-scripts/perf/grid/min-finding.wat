;; wat-scripts/perf/grid/min-finding.wat — GRID AXIS A7: minimum-finding-set ("≥N findings to
;; activate"), IN WAT.
;;
;; The DDoS "N findings to activate" primitive (docs/arc/2026/06/278-rules-engine/
;; DESIGN-clara-grid.md:44): an AccumulateNode(count) composed with a `where`/`:test` threshold
;; predicate that GATES activation — proven at the literal `>= 3` case by the repo's own
;; differential test tests/rete/probe_arc278_8b_accumulate_native_differential.rs:59,88-93
;; (`AccumulateNode(count) → TestNode(?n >= 3) → ProductionNode`), here generalized to scale.
;;
;; Shape (CLARA-TRANSLATIONS.md A7 — a straight composition of the A5 count fold + a boolean gate
;; over a static snapshot; no caveat beyond A5):
;;   Station(loc)                                          — S seed facts, loc in [0, stations).
;;   Reading(loc)                                          — (loc mod (2*threshold)) findings per
;;                                                            station, so counts span [0, 2T) and
;;                                                            EXACTLY the stations with count >= T
;;                                                            activate (a deterministic ~50% mix —
;;                                                            a non-trivial derived set that fails
;;                                                            loudly on any missing/extra activation).
;;   Busy(loc, n)  :- Station(loc) AND
;;                    (?n <- count :from Reading(loc)) AND
;;                    (?n >= threshold)                    — the minimum-finding-set rule: activate
;;                                                            iff at least `threshold` findings.
;;
;; size = [stations threshold]. `stations` (S) is the free scale dial (the activated-set-size
;; driver); `threshold` (T) is the minimum finding count to activate. Station loc is an i64 so the
;; canonical witness needs no string parsing (the probe uses String locations; this axis is free to
;; pick its own workload — an i64 loc is the honest, encoding-trivial choice at grid scale).
;;
;; Fires the NATIVE production verb `:wat::rete::fire-rules` (compile + seed are un-timed setup) —
;; the differential-tested fast path, NOT the wat oracle `fire-rules-spec`.
;;
;; :derived is the FULL SORTED activated set, each Busy fact canonicalized as a single i64
;; (loc * 1,000,000 + n) so it compares byte-for-byte against Clara's rendering of the identical
;; workload (gen-min-finding.sh). Both the activated identity (loc) AND its finding count (n) are
;; in the witness, so a wrong count that still crosses the gate would still show as a mismatch.
;;
;; Usage (stdin = an i64 vector [stations threshold]; stdout = one #grid/Result EDN line):
;;   echo '[2000 3]' | cargo wat ./wat-scripts/perf/grid/min-finding.wat
;;   => #grid/Result {:axis "min-finding" :size [2000 3] :derived [...] :native-ns N}

(:wat::core::defrecord :mf::Station [loc <- :wat::core::i64])
(:wat::core::defrecord :mf::Reading [loc <- :wat::core::i64])
(:wat::core::defrecord :mf::Busy    [loc <- :wat::core::i64  n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :mf::q-Busy
  :params []
  :when [(?fact <- :mf::Busy)])


;; encode loc n — canonical single-i64 witness for one activated Busy fact. `n` is a station's
;; finding count (< 2*threshold, far below 1,000,000) and `loc` is < 1,000,000 at every grid size,
;; so the encoding is injective for the sizes this axis is ever run at.
(:wat::core::defn :mf::encode [loc <- :wat::core::i64  n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ (:wat::i64::* loc 1000000) n))

;; build-rule threshold — the single minimum-finding-set rule:
;;   Busy(loc, n) :- Station(loc) AND (?n <- count :from Reading(loc)) AND (?n >= threshold)
;; The gate is a beta-level (:wat::rete::where ...) TestNode over the accumulate result `?n`
;; (compile-condition's where-branch, wat/rete.wat:557); `threshold` splices into the >= expr as a
;; bare i64 LITERAL via unquote (a computed value into a literal position is proven — deep-cascade
;; embeds `(= ?l (unquote prev))` the same way). The accumulate condition mirrors the probe's
;; COUNT const exactly: (?n <- (:wat::rete::acc::count) :from (:mf::Reading (?loc <- :loc))).
(:wat::core::defn :mf::build-rule [threshold <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [station-c (:wat::core::quasiquote (:mf::Station (?loc <- :loc)))
                    acc-c     (:wat::core::quasiquote
                                (?n <- (:wat::rete::acc::count) :from (:mf::Reading (?loc <- :loc))))
                    where-c   (:wat::core::quasiquote
                                ;; law A (#57): a `where` admits only :wat::rete:: ops. `?n` is bound by
                                ;; `(:wat::rete::acc::count)`, whose declared return is a bare i64, and
                                ;; `threshold` is an i64 — so the per-type twin is unambiguous. `>=` is
                                ;; OpClass::Alias (params [I64, I64]): a pure RENAME, no `:undefined`.
                                ;; A BUCKET C judgement site by the codemod's own table (no bare `>=`
                                ;; row exists — only i64::>= / f64::>=), which is why it is hand-decided.
                                (:wat::rete::where (:wat::rete::i64::>= ?n (:wat::core::unquote threshold))))
                    ins       (:wat::core::quasiquote (:mf::Busy ?loc ?n))]
    (:wat::rete::Rule :name "min-finding"
      :lhs (:wat::core::PersistentVector station-c acc-c where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; i64-mod a b — non-negative modulo via truncating division (no native i64::mod/rem; only
;; + - * / exist — same idiom strat-neg.wat uses for its even test `(* (/ ?k 2) 2)`). a >= 0 and
;; b > 0 for every call here (station indices and 2*threshold), so truncation-toward-zero is exact.
(:wat::core::defn :mf::i64-mod [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::- a (:wat::i64::* (:wat::i64::/ a b) b)))

;; seed-readings session loc count — stage `count` Reading(loc) findings for one station.
;; reading-facts loc count — `count` Reading(loc) facts as a FACT VECTOR. No longer threads a
;; Session: staging is one BATCH `insert-all` at the end of `seed`.
(:wat::core::defn :mf::reading-facts
  [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  loc <- :wat::core::i64  count <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::core::Record])  _r <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:wat::vector::conj a (:mf::Reading loc)))
    acc
    (:wat::core::range 0 count)))

;; seed session stations threshold — for each station i in [0, stations): stage Station(i) plus
;; (i mod (2*threshold)) Reading(i) findings. Counts span [0, 2T) so exactly the stations with
;; (i mod 2T) >= T activate.
(:wat::core::defn :mf::seed
  [session <- :wat::rete::Session  stations <- :wat::core::i64  threshold <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::core::let [span (:wat::i64::* 2 threshold)]
    (:wat::rete::insert-all
      session
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                        -> (:wat::core::PersistentVector :- [:wat::core::Record])
          (:mf::reading-facts
            (:wat::vector::conj acc (:mf::Station i))
            i
            (:mf::i64-mod i span)))
        (:wat::core::PersistentVector)
        (:wat::core::range 0 stations)))))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :mf::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every activated Busy fact, canonically encoded (loc*1M + n) and sorted
;; ascending. THIS is the accuracy witness: the full activated set, not a count — a mismatch
;; anywhere (a station that should/shouldn't have activated, or a wrong finding count) shows up.
(:wat::core::defn :mf::derived-vector
  [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:mf::vec->pvec
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:mf::encode (:mf::Busy/loc f) (:mf::Busy/n f))))
          (:wat::rete::query fired (:mf::q-Busy)))))))

;; ns-between t0 t1 — nanoseconds between two Instants (mirrors strat-neg.wat's ns-between).
(:wat::core::defn :mf::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    stations  (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [stations threshold]")
                    threshold (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [stations threshold]")
                    rules     (:wat::core::PersistentVector (:mf::build-rule threshold))
                    staged    (:mf::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:mf::q-Busy))) stations threshold)
                    ;; time the NATIVE production verb only (compile + seed are un-timed setup)
                    n0        (:wat::time::now)
                    fired     (:wat::rete::fire-rules staged)
                    n1        (:wat::time::now)
                    derived   (:mf::derived-vector fired)
                    nat-ns  (:mf::ns-between n0 n1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::rete::fire-rules$oracle staged)
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "min-finding" :size (:wat::core::PersistentVector stations threshold) :derived derived :native-ns nat-ns :oracle-derived (:mf::derived-vector ofired) :oracle-ns (:mf::ns-between o0 o1)))))
