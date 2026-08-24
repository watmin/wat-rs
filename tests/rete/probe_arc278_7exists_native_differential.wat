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


(:wat::core::defn :test::compile-watched [] -> :wat::rete::Session
  (:wat::rete::compile-all
    (:wat::rete::collect-rules :w)
    (:wat::core::PersistentVector (:w::q-Watched))))

(:wat::core::defn :test::seed-oslo-station [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert s (:w::Station :location "Oslo")))

(:wat::core::defn :test::seed-reading [s <- :wat::rete::Session loc <- :wat::core::String v <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert s (:w::Reading :location loc :value v)))

(:wat::core::defn :test::fire-native [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::fire-rules s))

(:wat::core::defn :test::fire-oracle [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::fire-rules$oracle s))

(:wat::core::defn :test::count-watched [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:w::q-Watched))))

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
