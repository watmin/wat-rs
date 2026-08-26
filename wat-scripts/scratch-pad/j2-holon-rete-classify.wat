;; Domain repro for BRIEF-native-where-vsa-ops. Twin of
;; tests/rete/probe_arc278_vsa_where_native_differential.wat.
;; Four Catalog holons (bool→bool tables), one Observation from applying a
;; wat [bool :-> bool], cosine `where` > 0.9. Oracle names the mystery;
;; native must too.

(:wat::core::defrecord :j2::Catalog     [name <- :wat::core::String  obs <- :wat::holon::HolonAST])
(:wat::core::defrecord :j2::Observation [obs  <- :wat::holon::HolonAST])
(:wat::core::defrecord :j2::Guess       [name <- :wat::core::String])

(:wat::core::defn :j2::table-of
  [f <- :wat::core::Fn(wat::core::bool)->wat::core::bool]
  -> :wat::holon::HolonAST
  (:wat::holon::to-holon
    (:wat::core::Vector :wat::core::bool (f true) (f false))))

(:wat::rete::defrule :j2::classify
  :when
  [(:j2::Catalog (?name <- :name) (?cobs <- :obs))
   (:j2::Observation (?obs <- :obs))
   (:wat::rete::where
     (:wat::rete::core::f64::>
       (:wat::rete::holon::cosine ?obs ?cobs :undefined 0.0)
       0.9))]
  :then
  [(:j2::Guess :name ?name)])

(:wat::rete::defquery :j2::q-Guess
  :params []
  :when [(:j2::Guess (?name <- :name))])

(:wat::core::defn :j2::catalog [] -> (:wat::core::PersistentVector :- [:j2::Catalog])
  (:wat::core::PersistentVector
    (:j2::Catalog :name "identity"    :obs (:j2::table-of (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool b)))
    (:j2::Catalog :name "not"         :obs (:j2::table-of (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool (:wat::core::if b false true))))
    (:j2::Catalog :name "const-true"  :obs (:j2::table-of (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool true)))
    (:j2::Catalog :name "const-false" :obs (:j2::table-of (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool false)))))

(:wat::core::defn :j2::run
  [fire    <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session
   mystery <- :wat::core::Fn(wat::core::bool)->wat::core::bool]
  -> :wat::core::String
  (:wat::core::let
    [s0    (:wat::rete::compile-all
             (:wat::core::PersistentVector (:j2::classify))
             (:wat::core::PersistentVector (:j2::q-Guess)))
     s1    (:wat::rete::insert-all s0 (:j2::catalog))
     s2    (:wat::rete::insert s1 (:j2::Observation :obs (:j2::table-of mystery)))
     fired (fire s2)
     hits  (:wat::rete::query fired (:j2::q-Guess))
     n     (:wat::core::length hits)]
    (:wat::core::if (:wat::i64::= n 1)
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get (:wat::core::first hits) "?name")
        "q-Guess: ?name")
      (:wat::core::String/concat "count=" (:wat::i64::to-string n)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::PersistentMap :identity (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool b))))
    (:wat::kernel::println (:wat::core::PersistentMap :not (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool (:wat::core::if b false true)))))
    (:wat::kernel::println (:wat::core::PersistentMap :const-true (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool true))))
    (:wat::kernel::println (:wat::core::PersistentMap :const-false (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool false))))
    (:wat::kernel::println (:wat::core::PersistentMap :native-id (:j2::run :wat::rete::fire-rules (:wat::core::fn [b <- :wat::core::bool] -> :wat::core::bool b))))))
