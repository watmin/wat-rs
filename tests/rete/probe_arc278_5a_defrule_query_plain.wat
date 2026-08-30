;; tests/rete/probe_arc278_5a_defrule_query_plain.wat — records-only fixture (no defrule) for the
;; probe_arc278_5a_defrule_query probe; loaded via startup_from_file for the query-only tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact <- :weather::ColdAndWindy)])


(:wat::rete::defquery :weather::q-WindSpeed
  :params []
  :when [(?fact <- :weather::WindSpeed)])


(:wat::core::defn :test::compile-plain [] -> :wat::rete::Session
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "weather::cold-and-windy" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))]
    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WindSpeed)))))

(:wat::core::defn :test::seed-oslo [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo"))
    (:weather::WindSpeed :kph 45 :location "Oslo")))

(:wat::core::defn :test::fired-oslo [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-plain))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::query-coldandwindy-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:test::fired-oslo) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::query-windspeed-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:test::fired-oslo) (:weather::q-WindSpeed))))
