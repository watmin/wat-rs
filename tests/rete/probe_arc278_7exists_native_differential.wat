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


;; Fire via `fire` after the given inserts; count derived Watched facts. Four scenarios x {native, oracle}.

(:wat::core::defn :user::native-one-reading [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 1))
       fired   (:wat::rete::fire-rules session)]
      (:wat::rete::query fired (:w::q-Watched)))))

(:wat::core::defn :user::oracle-one-reading [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 1))
       fired   (:wat::rete::fire-rules$oracle session)]
      (:wat::rete::query fired (:w::q-Watched)))))

(:wat::core::defn :user::native-station-only [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       fired   (:wat::rete::fire-rules session)]
      (:wat::rete::query fired (:w::q-Watched)))))

(:wat::core::defn :user::oracle-station-only [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       fired   (:wat::rete::fire-rules$oracle session)]
      (:wat::rete::query fired (:w::q-Watched)))))

(:wat::core::defn :user::native-three-readings [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 1))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 2))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 3))
       fired   (:wat::rete::fire-rules session)]
      (:wat::rete::query fired (:w::q-Watched)))))

(:wat::core::defn :user::oracle-three-readings [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 1))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 2))
       session (:wat::rete::insert session (:w::Reading :location "Oslo" :value 3))
       fired   (:wat::rete::fire-rules$oracle session)]
      (:wat::rete::query fired (:w::q-Watched)))))

(:wat::core::defn :user::native-reading-elsewhere [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       session (:wat::rete::insert session (:w::Reading :location "Bergen" :value 1))
       fired   (:wat::rete::fire-rules session)]
      (:wat::rete::query fired (:w::q-Watched)))))

(:wat::core::defn :user::oracle-reading-elsewhere [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :w)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w::q-Watched)))
       session (:wat::rete::insert session (:w::Station :location "Oslo"))
       session (:wat::rete::insert session (:w::Reading :location "Bergen" :value 1))
       fired   (:wat::rete::fire-rules$oracle session)]
      (:wat::rete::query fired (:w::q-Watched)))))

