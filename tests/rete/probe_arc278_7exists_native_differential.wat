;; tests/rete/probe_arc278_7exists_native_differential.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :w::watched existential rule for exists tests.

(:wat::core::defrecord :w::Station [location <- :wat::core::String])
(:wat::core::defrecord :w::Reading [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :w::Watched [location <- :wat::core::String])

(:wat::rete::defrule :w::watched
  :when
  [(:w::Station (?loc <- :location))
   (:wat::rete::exists (:w::Reading (?loc <- :location)))]
  :then
  [(:w::Watched :location ?loc)])

(:wat::rete::defquery :w::q-Watched
  :params []
  :when [(?fact <- :w::Watched)])

;; ── THE MULTIPLICITY INSTRUMENT — a query over the RULE's own shape ──────────────────────────
;; `q-Watched` above counts DERIVED FACTS, and that is exactly what cannot see multiplicity:
;; `:w::watched`'s `:then` binds only `?loc`, so three passes of the same token derive the SAME
;; `Watched{location:"Oslo"}` three times, and `production_delta`'s value-dedup collapses them to
;; one. A count of 1 is what a correct engine AND a fully-multiplying engine both report.
;;
;; This query has the rule's `:when` verbatim, so `query` reads BETA directly and each token that
;; reaches the production is one row. Three readings, one distinct `?loc`: correct is ONE row; an
;; engine that multiplied the existential would report three. Same instrument that caught the
;; leading-filter defect (`probe_arc278_leading_filter_multiplicity`) — the dedup is one layer
;; below beta, so a beta-reading query is the only vantage that can see past it.
(:wat::rete::defquery :w::q-watched-tokens
  :params []
  :when
  [(:w::Station (?loc <- :location))
   (:wat::rete::exists (:w::Reading (?loc <- :location)))])

;; ── THE INSTRUMENT'S OWN CONTROL — proves it can count ABOVE one ─────────────────────────────
;; Identical to `q-watched-tokens` in every way but ONE: the `exists` wrapper is gone, leaving a
;; plain join. A join DOES multiply — three matching Readings must give three beta rows. So the
;; pair is a differential on the wrapper alone: 3 here and 1 there is the existential property
;; being observed, not asserted. If this row count ever collapses to 1, the instrument has gone
;; blind and the sibling's green means nothing — which is the failure mode this control exists to
;; make impossible, since a gate that cannot go red is decoration.
(:wat::rete::defquery :w::q-watched-join
  :params []
  :when
  [(:w::Station (?loc <- :location))
   (:w::Reading (?loc <- :location))])


(:wat::core::defn :test::compile-watched [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all
    (:wat::rete::collect-rules :w)
    (:wat::core::PersistentVector (:w::q-Watched) (:w::q-watched-tokens) (:w::q-watched-join))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::seed-oslo-station [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:w::Station :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::seed-reading [s <- :wat::rete::Session loc <- :wat::core::String v <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:w::Reading :location loc :value v)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::fire-native [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::fire-oracle [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules$oracle s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::count-watched [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:w::q-Watched))))

;; Rows out of the rule's own beta — one per token that reached the production.
(:wat::core::defn :test::count-tokens [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:w::q-watched-tokens))))

;; Same query with the `exists` wrapper removed — the multiplying control.
(:wat::core::defn :test::count-join-tokens [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:w::q-watched-join))))

(:wat::core::defn :user::compile-watched-fires-nothing [] -> :wat::core::i64
  (:test::count-watched (:test::fire-native (:test::compile-watched))))

;; Fire via `fire` after the given inserts; count derived Watched facts. Four scenarios x {native, oracle}.

(:wat::core::defn :user::native-one-reading [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-native
      (:test::seed-reading
        (:test::seed-oslo-station (:test::compile-watched))
        "Oslo" 1))))

(:wat::core::defn :user::oracle-one-reading [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-oracle
      (:test::seed-reading
        (:test::seed-oslo-station (:test::compile-watched))
        "Oslo" 1))))

(:wat::core::defn :user::native-station-only [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-native
      (:test::seed-oslo-station (:test::compile-watched)))))

(:wat::core::defn :user::oracle-station-only [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-oracle
      (:test::seed-oslo-station (:test::compile-watched)))))

(:wat::core::defn :user::native-three-readings [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-native
      (:test::seed-reading
        (:test::seed-reading
          (:test::seed-reading
            (:test::seed-oslo-station (:test::compile-watched))
            "Oslo" 1)
          "Oslo" 2)
        "Oslo" 3))))

(:wat::core::defn :user::oracle-three-readings [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-oracle
      (:test::seed-reading
        (:test::seed-reading
          (:test::seed-reading
            (:test::seed-oslo-station (:test::compile-watched))
            "Oslo" 1)
          "Oslo" 2)
        "Oslo" 3))))

(:wat::core::defn :user::native-reading-elsewhere [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-native
      (:test::seed-reading
        (:test::seed-oslo-station (:test::compile-watched))
        "Bergen" 1))))

(:wat::core::defn :user::oracle-reading-elsewhere [] -> :wat::core::i64
  (:test::count-watched
    (:test::fire-oracle
      (:test::seed-reading
        (:test::seed-oslo-station (:test::compile-watched))
        "Bergen" 1))))

;; ── Token-level readings for the multiplicity gate (see :w::q-watched-tokens above) ──────────

(:wat::core::defn :user::native-three-readings-tokens [] -> :wat::core::i64
  (:test::count-tokens
    (:test::fire-native
      (:test::seed-reading
        (:test::seed-reading
          (:test::seed-reading
            (:test::seed-oslo-station (:test::compile-watched))
            "Oslo" 1)
          "Oslo" 2)
        "Oslo" 3))))

(:wat::core::defn :user::oracle-three-readings-tokens [] -> :wat::core::i64
  (:test::count-tokens
    (:test::fire-oracle
      (:test::seed-reading
        (:test::seed-reading
          (:test::seed-reading
            (:test::seed-oslo-station (:test::compile-watched))
            "Oslo" 1)
          "Oslo" 2)
        "Oslo" 3))))

(:wat::core::defn :user::native-one-reading-tokens [] -> :wat::core::i64
  (:test::count-tokens
    (:test::fire-native
      (:test::seed-reading
        (:test::seed-oslo-station (:test::compile-watched))
        "Oslo" 1))))

(:wat::core::defn :user::native-three-readings-join-tokens [] -> :wat::core::i64
  (:test::count-join-tokens
    (:test::fire-native
      (:test::seed-reading
        (:test::seed-reading
          (:test::seed-reading
            (:test::seed-oslo-station (:test::compile-watched))
            "Oslo" 1)
          "Oslo" 2)
        "Oslo" 3))))
