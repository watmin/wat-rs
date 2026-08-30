;; The compiled program lives on disk as datamancer.rete.edn.
;; Types stay in the world (ABI + facts + the ask). No defrule. No compile-all.

(:wat::core::defrecord :dm::Beat      [t <- :wat::core::i64  kind <- :wat::core::String])
(:wat::core::defrecord :dm::Artifact  [kind <- :wat::core::String  name <- :wat::core::String])
(:wat::core::defrecord :dm::Gap       [t <- :wat::core::i64])
(:wat::core::defrecord :dm::ReadAfter [t <- :wat::core::i64])
(:wat::core::defrecord :dm::Hollow    [t <- :wat::core::i64])
(:wat::core::defrecord :dm::Primer    [name <- :wat::core::String])
(:wat::core::defrecord :dm::Four      [n <- :wat::core::i64])
(:wat::core::defrecord :dm::Datamancer [n <- :wat::core::i64  sigil <- :wat::core::String])

(:wat::rete::defquery :dm::q-who    :params [] :when [(?who    <- :dm::Datamancer)])
(:wat::rete::defquery :dm::q-hollow :params [] :when [(?hollow <- :dm::Hollow)])

(:wat::core::defn :dm::seed-practice [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert s
    (:dm::Artifact :kind "log"   :name "datamancer.rete.edn")
    (:dm::Artifact :kind "log"   :name "CURRENT-STATE")
    (:dm::Artifact :kind "cache" :name "summary")
    (:dm::Beat :t 0 :kind "gap")
    (:dm::Beat :t 1 :kind "cache")
    (:dm::Beat :t 2 :kind "read-log")
    (:dm::Beat :t 3 :kind "fetch-primer")
    (:dm::Beat :t 4 :kind "tend-record")
    (:dm::Beat :t 5 :kind "weigh-disk")
    (:dm::Beat :t 6 :kind "root-failure")))

;; Same gap. A cache. Even a written file. Never read the log.
(:wat::core::defn :dm::seed-impostor [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert s
    (:dm::Artifact :kind "cache" :name "summary")
    (:dm::Beat :t 0 :kind "gap")
    (:dm::Beat :t 1 :kind "cache")
    (:dm::Beat :t 4 :kind "tend-record")))

(:wat::core::defn :dm::counts
  [txt  <- :wat::core::String
   seed <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [exp   (:wat::edn::read txt)
                    s0    (:wat::rete::import exp)
                    fired (:wat::core::match (:wat::rete::fire-rules (seed s0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::query fired (:dm::q-who)))
      (:wat::core::length (:wat::rete::query fired (:dm::q-hollow))))))

(:wat::core::defn :user::practice [txt <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:dm::counts txt :dm::seed-practice))

(:wat::core::defn :user::impostor [txt <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:dm::counts txt :dm::seed-impostor))

(:wat::core::defn :user::disk-reexport-identical [txt <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [e1 (:wat::edn::read txt)
                    e2 (:wat::rete::export (:wat::rete::import e1))]
    (:wat::core::= (:wat::edn::write e1) (:wat::edn::write e2))))

(:wat::core::defn :user::sigil [txt <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [exp   (:wat::edn::read txt)
                    s0    (:wat::rete::import exp)
                    fired (:wat::core::match (:wat::rete::fire-rules (:dm::seed-practice s0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    who   (:wat::rete::query fired (:dm::q-who))
                    fact  (:wat::core::Option/expect
                            (:wat::core::PersistentMap/get
                              (:wat::core::first who)
                              "?who")
                            "sigil: no Datamancer")]
    (:dm::Datamancer/sigil fact)))
