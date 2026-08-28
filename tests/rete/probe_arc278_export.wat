;; #wat.rete/Export — compiled program from source. Native fire only.

(:wat::core::defrecord :exp::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :exp::Hit [c <- :wat::core::i64])

(:wat::rete::defquery :exp::q-Hit :params [] :when [(?fact <- :exp::Hit)])

(:wat::rete::defrule :exp::cool
  :when [(:exp::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))]
  :then [(:exp::Hit ?c)])

;; ── Op::Eval ACROSS THE WIRE (fix-list F) ─────────────────────────────────────────────────────
;;
;; An INLINE constraint with a COMPUTED operand. Every other rule in this fixture uses a `where`
;; fence over plain operands, so before this rule NOTHING in the tree ever serialized an
;; `Op::Eval` — its `pack_cond_op` / `unpack` / `check_cond_ops` arms existed and were never
;; driven. Presence is not aliveness, and an untested serialization arm is exactly the class this
;; arc keeps finding.
;;
;; `(c + 5) < 20` selects the c=10 fact and rejects c=30, the same two seeds the other rules use.
(:wat::rete::defrule :exp::cool-computed
  :when [(:exp::Temp (?c <- :c)
           (:wat::rete::core::i64::< (:wat::rete::core::i64::+ :c 5 :undefined 0) 20))]
  :then [(:exp::Hit ?c)])

(:wat::core::defn :exp::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:exp::Temp :c 10))
    (:exp::Temp :c 30)))

(:wat::core::defn :user::source-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s0)) (:exp::q-Hit)))))

(:wat::core::defn :user::empty-pv [] -> :wat::core::PersistentVector
  (:wat::core::PersistentVector))

(:wat::core::defn :user::cool-export [] -> :wat::rete::Export
  (:wat::rete::export
    (:wat::rete::compile-all
      (:wat::core::PersistentVector (:exp::cool))
      (:wat::core::PersistentVector (:exp::q-Hit)))))

;; Op::Eval across the wire, NATIVE: compile the computed-inline rule, export it, import it, fire.
;; `(c + 5) < 20` admits c=10 and rejects c=30, so the answer is 1 — and it is 1 only if the
;; `Op::Eval` survived pack -> unpack -> bounds-check -> exec.
(:wat::core::defn :user::computed-roundtrip-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool-computed))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    s1 (:wat::rete::import (:wat::rete::export s0))]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s1)) (:exp::q-Hit)))))

;; The SOURCE answer for the same rule — no export/import at all. The round-trip is only
;; meaningful against what the un-serialized program says, and pinning both catches a fixture
;; whose rule stopped discriminating (which would make the round-trip agree at the wrong number).
(:wat::core::defn :user::computed-source-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool-computed))
                         (:wat::core::PersistentVector (:exp::q-Hit)))]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s0)) (:exp::q-Hit)))))

(:wat::core::defn :user::import-one [e <- :wat::rete::Export] -> :wat::rete::Session
  (:wat::rete::import e))

(:wat::core::defn :user::spec-on-import [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    s1 (:wat::rete::import (:wat::rete::export s0))]
    (:wat::core::length
      (:wat::rete::query
        (:wat::rete::fire-rules$oracle (:exp::seed s1))
        (:exp::q-Hit)))))

(:wat::core::defn :user::spec-once-on-import [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    s1 (:wat::rete::import (:wat::rete::export s0))]
    (:wat::core::length
      (:wat::rete::query
        (:wat::rete::fire-once$oracle (:exp::seed s1))
        (:exp::q-Hit)))))

(:wat::core::defn :user::import-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    exp (:wat::rete::export s0)
                    s1 (:wat::rete::import exp)]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s1)) (:exp::q-Hit)))))

(:wat::core::defn :user::export-sizes [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    exp (:wat::rete::export s0)
                    sl (:wat::core::string::length (:wat::edn::write s0))
                    el (:wat::core::string::length (:wat::edn::write exp))
                    nc (:wat::core::length (:wat::rete::Export/classes exp))
                    nn (:wat::core::length (:wat::rete::Export/nodes exp))
                    ncond (:wat::core::length (:wat::rete::Export/conds exp))]
    (:wat::core::PersistentVector sl el nc nn ncond)))

;; Negation over a DERIVED fact. Source fire is stratified. Import must match.
(:wat::core::defrecord :sn::A   [k <- :wat::core::i64])
(:wat::core::defrecord :sn::Bad [k <- :wat::core::i64])
(:wat::core::defrecord :sn::Ok  [k <- :wat::core::i64])

(:wat::rete::defrule :sn::mark-bad
  :when [(:sn::A (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= ?k 2))]
  :then [(:sn::Bad :k ?k)])

(:wat::rete::defrule :sn::ok
  :when [(:sn::A (?k <- :k))
         (:wat::rete::not (:sn::Bad (?k <- :k)))]
  :then [(:sn::Ok :k ?k)])

(:wat::rete::defquery :sn::q-Bad :params [] :when [(?fact <- :sn::Bad)])
(:wat::rete::defquery :sn::q-Ok  :params [] :when [(?fact <- :sn::Ok)])

(:wat::core::defn :sn::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:sn::A :k 1))
    (:sn::A :k 2)))

(:wat::core::defn :sn::counts [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query fired (:sn::q-Bad)))
    (:wat::core::length (:wat::rete::query fired (:sn::q-Ok)))))

(:wat::core::defn :user::strat-source-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:sn::mark-bad) (:sn::ok))
                         (:wat::core::PersistentVector (:sn::q-Bad) (:sn::q-Ok)))]
    (:sn::counts (:wat::rete::fire-rules (:sn::seed s0)))))

(:wat::core::defn :user::strat-import-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:sn::mark-bad) (:sn::ok))
                         (:wat::core::PersistentVector (:sn::q-Bad) (:sn::q-Ok)))
                    exp (:wat::rete::export s0)
                    s1 (:wat::rete::import exp)]
    (:sn::counts (:wat::rete::fire-rules (:sn::seed s1)))))

(:wat::core::defn :user::reexport-shape [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    e1 (:wat::rete::export s0)
                    e2 (:wat::rete::export (:wat::rete::import e1))]
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::Export/deps e1))
      (:wat::core::length (:wat::rete::Export/deps e2))
      (:wat::core::length (:wat::rete::Export/nodes e1))
      (:wat::core::length (:wat::rete::Export/nodes e2))
      (:wat::core::length (:wat::rete::Export/conds e1))
      (:wat::core::length (:wat::rete::Export/conds e2))
      (:wat::core::length (:wat::rete::Export/rhs e1))
      (:wat::core::length (:wat::rete::Export/rhs e2)))))

(:wat::core::defn :user::reexport-deps-length [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    e1 (:wat::rete::export s0)
                    s1 (:wat::rete::import e1)
                    e2 (:wat::rete::export s1)]
    (:wat::core::length (:wat::rete::Export/deps e2))))

(:wat::core::defn :user::edn-roundtrip-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    exp (:wat::rete::export s0)
                    txt (:wat::edn::write exp)
                    exp2 (:wat::edn::read txt)
                    s1 (:wat::rete::import exp2)]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s1)) (:exp::q-Hit)))))

;; One EDN value: write(e) == write(export(import(e))).
(:wat::core::defn :user::reexport-edn-identical [] -> :wat::core::bool
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    e1 (:wat::rete::export s0)
                    e2 (:wat::rete::export (:wat::rete::import e1))]
    (:wat::core::= (:wat::edn::write e1) (:wat::edn::write e2))))

;; Fire of import(export(import(e))) — the re-export, not the original export.
(:wat::core::defn :user::reexport-import-fires [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    e1 (:wat::rete::export s0)
                    e2 (:wat::rete::export (:wat::rete::import e1))
                    s2 (:wat::rete::import e2)]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s2)) (:exp::q-Hit)))))

;; The compiled program as an EDN string — source of tests/rete/hello.rete.edn.
(:wat::core::defn :user::export-edn [] -> :wat::core::String
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))]
    (:wat::edn::write (:wat::rete::export s0))))
