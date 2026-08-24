;; wat-scripts/perf/grid/asym-join.wat — GRID AXIS A2: asymmetric-arrival joins, IN WAT.
;;
;; The axis that HID R18 (now P6-fixed): a derived⋈input join where the RIGHT side of the join
;; arrives BEFORE the left, at scale. Grounded in tests/rete/probe_arc278_P6_delta_asymmetric_join
;; and CLARA-TRANSLATIONS.md §A2. The minimal repro is the "chain":
;;   R1: A(?k)          -> B(?k)          [derive B from every input A]
;;   R2: B(?k) ⋈ A(?k)  -> C(?k)          [join derived B with input A on ?k -> C]
;; Insert A(0..items). Under wat's round-based semi-naive delta kernel, A (the RIGHT side of R2's
;; hash join) is present from round 0, while B (the LEFT/token side) is only DERIVED in round 1 —
;; i.e. the right side is keyed before the left exists. Before the P6 catch-up (rebuild both join
;; indices from ALL cumulative memory on first keying) this dropped C entirely (C=0 instead of
;; C=items). `:wat::rete::fire-rules` (native) is now asserted == `fire-rules-spec` (oracle) on
;; exactly this shape, so this axis is an ACCURACY(match) + RAW-SPEED benchmark.
;;
;; CAVEAT (CLARA-TRANSLATIONS.md §A2, load-bearing): Clara has NO arrival-order hazard by
;; construction — its HashJoinNode left/right-activate always read the OTHER side's complete
;; persistent memory, so any insertion order is trivially correct. Clara is used here purely as the
;; GROUND-TRUTH ORACLE for the now-fixed wat ordering bug; do NOT frame the speed comparison as
;; "Clara handles arrival better/worse" — that axis does not exist for Clara.
;;
;; :derived is the FULL SORTED derived-fact set (both B and C), canonicalized as a single i64 per
;; fact (type-tag * 1,000,000 + k; B=tag 0, C=tag 1) so it compares byte-for-byte against Clara's
;; rendering of the identical workload (gen-asym-join.sh) — a mismatch (missing/extra fact on
;; either side) surfaces loudly.
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[2000]' | cargo wat ./wat-scripts/perf/grid/asym-join.wat
;;   => #grid/Result {:axis "asym-join" :size [2000] :derived [...] :native-ns N}

(:wat::core::defrecord :asym::A [k <- :wat::core::i64])   ;; input
(:wat::core::defrecord :asym::B [k <- :wat::core::i64])   ;; derived: A -> B
(:wat::core::defrecord :asym::C [k <- :wat::core::i64])   ;; derived: B ⋈ A -> C

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :asym::q-B
  :params []
  :when [(?fact <- :asym::B)])


(:wat::rete::defquery :asym::q-C
  :params []
  :when [(?fact <- :asym::C)])


;; encode tag k — canonical single-i64 witness for one derived fact (B=tag 0, C=tag 1).
;; items is always far below 1,000,000 at grid scale, so the encoding is injective here.
(:wat::core::defn :asym::encode [tag <- :wat::core::i64  k <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+ (:wat::core::i64::* tag 1000000) k))

;; build-rules — the fixed 2-rule chain: R1 A->B, R2 B⋈A->C. Mirrors chain_expr in
;; probe_arc278_P6_delta_asymmetric_join.rs exactly (conditions as (:type (?k <- :k)) patterns;
;; a two-pattern LHS on r2 is the join, both binding ?k -> equi-join on k).
(:wat::core::defn :asym::build-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::rete::Rule :name "r1"
      :lhs (:wat::core::PersistentVector (:wat::core::quote (:asym::A (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quote (:asym::B ?k))))
    (:wat::rete::Rule :name "r2"
      :lhs (:wat::core::PersistentVector
        (:wat::core::quote (:asym::B (?k <- :k)))
        (:wat::core::quote (:asym::A (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quote (:asym::C ?k))))))

;; seed-items session items — stage A(i) for i in [0, items), threading the staging session.
;; Every A arrives (round 0) before ANY B/C is derived — the asymmetric-arrival condition.
;; Staged with the BATCH verb: build the fact vector, then ONE `insert-all` (which delegates to
;; the native `insert-all'` — one rebuild, not N). `insert` x N is what a user should never write,
;; so the benchmark must not write it either.
(:wat::core::defn :asym::seed-items [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:asym::A i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; b-codes / c-codes — every derived fact of each type, canonically encoded.
(:wat::core::defn :asym::b-codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::Vector :wat::core::i64)
    (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:asym::encode 0 (:asym::B/k f))))
      (:wat::rete::query fired (:asym::q-B)))))

(:wat::core::defn :asym::c-codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::Vector :wat::core::i64)
    (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:asym::encode 1 (:asym::C/k f))))
      (:wat::rete::query fired (:asym::q-C)))))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :asym::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every derived fact (B then C), canonically encoded and sorted ascending.
;; THE accuracy witness: the full set, not a count.
(:wat::core::defn :asym::derived-vector [fired <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [all (:wat::core::into (:asym::b-codes fired) (:asym::c-codes fired))]
    (:asym::vec->pvec (:wat::core::sort all))))

;; ns-between t0 t1 — nanoseconds between two Instants (cf. strat-neg.wat).
(:wat::core::defn :asym::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    rules   (:asym::build-rules)
                    staged  (:asym::seed-items (:wat::rete::compile-all rules (:wat::core::PersistentVector (:asym::q-B) (:asym::q-C))) items)
                    ;; time the NATIVE production verb only (compile + seed are un-timed setup)
                    n0      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    n1      (:wat::time::now)
                    derived (:asym::derived-vector fired)
                    nat-ns  (:asym::ns-between n0 n1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::rete::fire-rules$oracle staged)
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "asym-join" :size (:wat::core::PersistentVector items) :derived derived :native-ns nat-ns :oracle-derived (:asym::derived-vector ofired) :oracle-ns (:asym::ns-between o0 o1)))))
