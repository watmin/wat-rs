;; Domain repro for BRIEF-native-where-vsa-ops. Twin of
;; tests/rete/probe_arc278_vsa_where_native_differential.wat.
;; Four Catalog holons (bool→bool tables), one Observation from applying a
;; wat [bool :-> bool], cosine `where` > 0.9. Oracle names the mystery;
;; native must too.

(:wat::core::defrecord :j2::Catalog     [name <- wat.type/String  obs <- :wat::holon::HolonAST])
(:wat::core::defrecord :j2::Observation [obs  <- :wat::holon::HolonAST])
(:wat::core::defrecord :j2::Guess       [name <- wat.type/String])

(:wat::core::defn :j2::table-of
  [f <- [wat.type/bool :-> wat.type/bool]]
  -> :wat::holon::HolonAST
  (:wat::holon::to-holon
    (wat.type/Vector :- [wat.type/bool] (f true) (f false))))

(:wat::rete::defrule :j2::classify
  :when
  [(:j2::Catalog (?name :- :name) (?cobs :- :obs))
   (:j2::Observation (?obs :- :obs))
   (:wat::rete::where
     (:wat::rete::f64::>
       (:wat::rete::holon::cosine ?obs ?cobs :undefined 0.0)
       0.9))]
  :then
  [(:j2::Guess :name ?name)])

(:wat::rete::defquery :j2::q-Guess
  :params []
  :when [(:j2::Guess (?name :- :name))])

(:wat::core::defn :j2::catalog [] -> (wat.type/PersistentVector :- [:j2::Catalog])
  (wat.type/PersistentVector :- [:j2::Catalog]
    (:j2::Catalog :name "identity"    :obs (:j2::table-of (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool b)))
    (:j2::Catalog :name "not"         :obs (:j2::table-of (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool (:wat::core::if b false true))))
    (:j2::Catalog :name "const-true"  :obs (:j2::table-of (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool true)))
    (:j2::Catalog :name "const-false" :obs (:j2::table-of (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool false)))))

(:wat::core::defn :j2::run
  [fire    <- [:wat::rete::Session :-> (:wat::rete::FireOutcome :- [:wat::rete::Session])]
   mystery <- [wat.type/bool :-> wat.type/bool]]
  -> wat.type/String
  (:wat::core::let
    [s0    (:wat::core::match (:wat::rete::compile-all
             (wat.type/PersistentVector :- [:wat::rete::Rule] (:j2::classify))
             (wat.type/PersistentVector :- [:wat::rete::Query] (:j2::q-Guess))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
     s1    (:wat::core::match (:wat::rete::insert-all s0 (:j2::catalog)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     s2    (:wat::core::match (:wat::rete::insert s1 (:j2::Observation :obs (:j2::table-of mystery))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired (:wat::core::match (fire s2) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "fire: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "fire: fixpoint round cap exceeded")])
     hits  (:wat::rete::query fired (:j2::q-Guess))
     n     (:wat::core::length hits)]
    (:wat::core::if (:wat::i64::= n 1)
      (:wat::core::Option/expect
        (:wat::core::get (:wat::core::first hits) "?name")
        "q-Guess: ?name")
      (:wat::string::concat "count=" (:wat::i64::to-string n)))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::do
    (:wat::kernel::println (wat.type/PersistentMap :- [wat.type/keyword wat.type/String] :identity (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool b))))
    (:wat::kernel::println (wat.type/PersistentMap :- [wat.type/keyword wat.type/String] :not (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool (:wat::core::if b false true)))))
    (:wat::kernel::println (wat.type/PersistentMap :- [wat.type/keyword wat.type/String] :const-true (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool true))))
    (:wat::kernel::println (wat.type/PersistentMap :- [wat.type/keyword wat.type/String] :const-false (:j2::run :wat::rete::fire-rules$oracle (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool false))))
    (:wat::kernel::println (wat.type/PersistentMap :- [wat.type/keyword wat.type/String] :native-id (:j2::run :wat::rete::fire-rules (:wat::core::fn [b <- wat.type/bool] -> wat.type/bool b))))))
