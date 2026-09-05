;; VIGILIA experiri probe — `harvest-support` (wat/rete/oracle/explain.wat:10-49) folds over
;; `(:wat::core::PersistentMap/keys network)` — HAMT order — with NO sort, while its sibling
;; `fire-once$oracle` (wat/rete/oracle/fire.wat:151-160) sorts and says in writing
;; "PersistentMap/keys is HAMT order — not that. … Native sorts; the spec must too."
;; harvest-support is FIRST-PRODUCER-WINS, so traversal order decides which rule is
;; attributed when two rules derive the SAME fact.
;;
;; `Out` is derived by BOTH :vex::aaa and :vex::zzz. `Solo` has one producer — the control:
;; its attribution cannot depend on order, so a disagreement there would be a driver defect,
;; not this finding.

(:wat::core::defrecord :vex::In   [k <- :wat::core::i64])
(:wat::core::defrecord :vex::Out  [k <- :wat::core::i64])
(:wat::core::defrecord :vex::Solo [k <- :wat::core::i64])

(:wat::rete::defrule :vex::aaa  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::bbb  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::ccc  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::ddd  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::eee  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::fff  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::ggg  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::zzz  :when [(:vex::In (?k <- :k))] :then [(:vex::Out :k ?k)])
(:wat::rete::defrule :vex::solo :when [(:vex::In (?k <- :k))] :then [(:vex::Solo :k ?k)])

(:wat::core::defn :vex::session [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
    (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :vex))
      ((:wat::rete::CompileOutcome::Compiled __s) __s)
      ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
        (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
    (:vex::In :k 1))
    ((:wat::rete::InsertOutcome::Inserted __x) __x)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
      (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :vex::nat [] -> :wat::rete::Explained
  (:wat::core::match (:wat::rete::fire-rules-explain (:vex::session))
    ((:wat::rete::FireOutcome::Fired __e) __e)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
      (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
      (:wat::kernel::assertion-failed! "roundcap" :wat::core::None :wat::core::None))))

(:wat::core::defn :vex::ora [] -> :wat::rete::Explained
  (:wat::core::match (:wat::rete::fire-rules-explain$oracle (:vex::session))
    ((:wat::rete::FireOutcome::Fired __e) __e)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
      (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
      (:wat::kernel::assertion-failed! "roundcap" :wat::core::None :wat::core::None))))

(:wat::core::defn :vex::rule-of [ex <- :wat::rete::Explained  f <- :wat::core::Record] -> :wat::core::String
  (:wat::core::Option/expect
    (:wat::rete::DerivationNode/rule (:wat::rete::explain ex f))
    "no producing rule recorded for this fact"))

;; [native Out-rule, oracle Out-rule, native Solo-rule (control), oracle Solo-rule (control)]
(:wat::core::defn :user::attribution [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::mapv
    (:wat::core::fn [s <- :wat::core::String] -> :wat::core::String s)
    (:wat::core::PersistentVector
      (:vex::rule-of (:vex::nat) (:vex::Out :k 1))
      (:vex::rule-of (:vex::ora) (:vex::Out :k 1))
      (:vex::rule-of (:vex::nat) (:vex::Solo :k 1))
      (:vex::rule-of (:vex::ora) (:vex::Solo :k 1)))))
