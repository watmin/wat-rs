;; wat-scripts/perf/grid/fanout.wat — GRID AXIS A1: fan-out / low-selectivity join, IN WAT.
;;
;; Adapted from the legacy `wat-scripts/perf/matrix/fanout-join.wat` (R4's original A1 bench) into
;; the Clara-grid contract (`run-axis.sh`, DESIGN-STONE-a0-a1-into-the-grid.md): stdin becomes a
;; single-number i64 vector (not `[keys fanout]`), and `:derived` becomes the FULL SORTED canonical
;; Pair-fact vector (the legacy file only counted pairs). The legacy file + its Clara sibling
;; (`matrix/fanout-clara.clj`) are untouched — R4's record, left in place.
;;
;; **A1 at its top rung (40,000) is the only recorded Clara win in this project's history**
;; (REALIZATIONS.md:201: "40k Clara 1.4x") — measuring it again, with a real accuracy witness this
;; time, is the entire point of this axis landing in the grid.
;;
;; fanout (F) is FIXED at 20 — R4's exact P9/P10/P11 bench configuration (`echo '[100 20]'`, the
;; 40,000-pair cell that produced "134ms vs 96ms"). `items` (the single stdin dial) is the TARGET
;; DERIVED-PAIR COUNT, not keys or fanout directly: keys = items / F^2, so items = keys * F^2
;; exactly at every ladder rung (10000/20000/40000 -> keys 25/50/100). This makes `items` literally
;; equal `:derived`'s length, and reproduces R4's exact 40k cell (keys=100, fanout=20) at the top
;; rung — deliberately, per the DESIGN.
;;
;; Shape (unchanged from the legacy bench): F Lefts x F Rights share a key -> F^2 joined Pairs per
;; key, K keys -> K*F^2 derived Pairs (the classic RETE join-explosion / low-selectivity regime).
;;   Left(k,f), Right(k,f)   — seeded for f in [0,F), for every key k in [0,keys).
;;   Pair(k,l,r) :- Left(k,l) AND Right(k,r)     (joined on the shared key k)
;;
;; Fires the NATIVE production verb `:wat::rete::fire-rules` (the differential-tested fast path,
;; the same verb every other grid axis uses) — the legacy file called `fire-rules'` (P4a) directly;
;; `fire-rules` delegates to it, so the measured mechanism is unchanged.
;;
;; :derived is the full sorted i64 vector of every derived Pair fact, each fact canonicalized as
;; key*1000000 + lid*1000 + rid (single fact type, no `kind` term needed — key<=100, lid/rid<20 at
;; grid scale, injective).
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[10000]' | cargo wat ./wat-scripts/perf/grid/fanout.wat
;;   => #grid/Result {:axis "fanout" :size [10000] :derived [...] :native-ns N}

(:wat::core::defrecord :fan::Left  [key <- :wat::core::i64  lid <- :wat::core::i64])
(:wat::core::defrecord :fan::Right [key <- :wat::core::i64  rid <- :wat::core::i64])
(:wat::core::defrecord :fan::Pair  [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64])

(:wat::core::defrecord :fan::QuerySplit
  [read-ns   <- :wat::core::i64
   encode-ns <- :wat::core::i64
   sort-ns   <- :wat::core::i64
   into-ns   <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64
   insert-ns      <- :wat::core::i64
   fire-ns        <- :wat::core::i64
   query-ns       <- :wat::core::i64
   protocol-ns    <- :wat::core::i64])

(:wat::rete::defquery :fan::q-Pair
  :params []
  :when [(?fact <- :fan::Pair)])


;; facts-key k fanout — the Left(k,f)+Right(k,f) facts for f in [0,fanout), as a FACT VECTOR.
;; It no longer threads a Session: staging is now one BATCH call at the end (below), so the
;; helper's job is to produce facts, not to insert them. Named for what it returns.
(:wat::core::defn :fan::facts-key [k <- :wat::core::i64  fanout <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  f <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:wat::vector::conj
        (:wat::vector::conj acc (:fan::Left :key k :lid f))
        (:fan::Right :key k :rid f)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 fanout)))

;; all-facts keys fanout — every key's F Lefts + F Rights. Construct only.
(:wat::core::defn :fan::all-facts [keys <- :wat::core::i64  fanout <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:wat::vector::concat acc (:fan::facts-key k fanout)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 keys)))

;; seed s keys fanout — every key's F Lefts + F Rights, staged in ONE `insert-all` (which
;; delegates to the native `insert-all'`: one rebuild, not N). Order is preserved exactly —
;; ascending k, and within a key ascending f, Left before Right — so `:derived` is unchanged.
(:wat::core::defn :fan::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all s (:fan::all-facts keys fanout)))

;; enc key lid rid — canonical single-i64 witness for one derived Pair fact.
(:wat::core::defn :fan::enc [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+
    (:wat::i64::+ (:wat::i64::* key 1000000) (:wat::i64::* lid 1000))
    rid))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :fan::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every derived Pair fact, canonically encoded, sorted ascending.
(:wat::core::defn :fan::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:fan::vec->pvec
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:fan::enc (:fan::Pair/key f) (:fan::Pair/lid f) (:fan::Pair/rid f))))
          (:wat::rete::query fired (:fan::q-Pair)))))))

;; ns-between t0 t1 — nanoseconds between two Instants (mirrors accum.wat's ns-between).
(:wat::core::defn :fan::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    fanout  20
                    keys    (:wat::i64::/ items (:wat::i64::* fanout fanout))
                    c1      (:wat::core::quote (:fan::Left  (?k <- :key) (?l <- :lid)))
                    c2      (:wat::core::quote (:fan::Right (?k <- :key) (?r <- :rid)))
                    rhs     (:wat::core::quote (:fan::Pair ?k ?l ?r))
                    rule    (:wat::rete::Rule :name "fan" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs))
                    session (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:fan::q-Pair)))
                    facts   (:fan::all-facts keys fanout)
                    p0      (:wat::time::now)
                    staged  (:wat::rete::insert-all session facts)
                    i1      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    f1      (:wat::time::now)
                    qr0     (:wat::time::now)
                    raw     (:wat::rete::query fired (:fan::q-Pair))
                    qr1     (:wat::time::now)
                    enc0    (:wat::time::now)
                    encoded (:wat::core::mapv
                              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
                                (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")]
                                  (:fan::enc (:fan::Pair/key f) (:fan::Pair/lid f) (:fan::Pair/rid f))))
                              raw)
                    enc1    (:wat::time::now)
                    srt0    (:wat::time::now)
                    sorted  (:wat::core::sort encoded)
                    srt1    (:wat::time::now)
                    pv0     (:wat::time::now)
                    derived (:fan::vec->pvec sorted)
                    pv1     (:wat::time::now)
                    q1      pv1
                    ins-ns  (:fan::ns-between p0 i1)
                    fir-ns  (:fan::ns-between i1 f1)
                    qry-ns  (:fan::ns-between f1 q1)
                    proto-ns (:fan::ns-between p0 q1)
                    _split  (:wat::kernel::println
                              (:fan::QuerySplit
                                :read-ns   (:fan::ns-between qr0 qr1)
                                :encode-ns (:fan::ns-between enc0 enc1)
                                :sort-ns   (:fan::ns-between srt0 srt1)
                                :into-ns   (:fan::ns-between pv0 pv1)))
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::rete::fire-rules$oracle staged)
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "fanout" :size (:wat::core::PersistentVector items) :derived derived :native-ns fir-ns :oracle-derived (:fan::derived-vector ofired) :oracle-ns (:fan::ns-between o0 o1) :insert-ns ins-ns :fire-ns fir-ns :query-ns qry-ns :protocol-ns proto-ns))))
