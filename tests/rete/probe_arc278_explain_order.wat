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

;; Inner first-wins: an `:or` ProductionNode has N parents. All arms derive
;; the same Out. F1 itself was blind at two producers; this fixture has
;; eight arms, the size that made the original probe discriminate.
(:wat::core::defrecord :orx::A1 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::A2 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::A3 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::A4 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::A5 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::A6 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::A7 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::A8 [k <- :wat::core::i64])
(:wat::core::defrecord :orx::Out [k <- :wat::core::i64])

(:wat::rete::defrule :orx::either
  :when [(:wat::rete::or
           (:orx::A1 (?k <- :k))
           (:orx::A2 (?k <- :k))
           (:orx::A3 (?k <- :k))
           (:orx::A4 (?k <- :k))
           (:orx::A5 (?k <- :k))
           (:orx::A6 (?k <- :k))
           (:orx::A7 (?k <- :k))
           (:orx::A8 (?k <- :k)))]
  :then [(:orx::Out :k ?k)])

(:wat::core::defn :orx::via-type [ex <- :wat::rete::Explained] -> :wat::core::String
  (:wat::core::type
    (:wat::rete::DerivationNode/fact
      (:wat::rete::DerivationStep/supporting
        (:wat::core::Option/expect
          (:wat::core::get (:wat::rete::DerivationNode/via
                             (:wat::rete::explain ex (:orx::Out :k 1)))
                           0)
          "orx: via[0]")))))

(:wat::core::defn :orx::compiled [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :orx))
    ((:wat::rete::CompileOutcome::Compiled __s) __s)
    ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
      (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :orx::fire-native [s <- :wat::rete::Session] -> :wat::rete::Explained
  (:wat::core::match (:wat::rete::fire-rules-explain s)
    ((:wat::rete::FireOutcome::Fired __e) __e)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
      (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
      (:wat::kernel::assertion-failed! "roundcap" :wat::core::None :wat::core::None))))

(:wat::core::defn :orx::fire-oracle [s <- :wat::rete::Session] -> :wat::rete::Explained
  (:wat::core::match (:wat::rete::fire-rules-explain$oracle s)
    ((:wat::rete::FireOutcome::Fired __e) __e)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
      (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
      (:wat::kernel::assertion-failed! "roundcap" :wat::core::None :wat::core::None))))

(:wat::core::defn :orx::explain-all [] -> :wat::rete::Explained
  (:wat::core::match
    (:wat::rete::insert-all (:orx::compiled)
      (:wat::core::PersistentVector :- [:wat::core::Record]
        (:orx::A1 :k 1) (:orx::A2 :k 1) (:orx::A3 :k 1) (:orx::A4 :k 1)
        (:orx::A5 :k 1) (:orx::A6 :k 1) (:orx::A7 :k 1) (:orx::A8 :k 1)))
    ((:wat::rete::InsertOutcome::Inserted staged) (:orx::fire-native staged))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
      (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :orx::explain-all-oracle [] -> :wat::rete::Explained
  (:wat::core::match
    (:wat::rete::insert-all (:orx::compiled)
      (:wat::core::PersistentVector :- [:wat::core::Record]
        (:orx::A1 :k 1) (:orx::A2 :k 1) (:orx::A3 :k 1) (:orx::A4 :k 1)
        (:orx::A5 :k 1) (:orx::A6 :k 1) (:orx::A7 :k 1) (:orx::A8 :k 1)))
    ((:wat::rete::InsertOutcome::Inserted staged) (:orx::fire-oracle staged))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
      (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :orx::explain-a1-only [] -> :wat::rete::Explained
  (:wat::core::match
    (:wat::rete::insert (:orx::compiled) (:orx::A1 :k 1))
    ((:wat::rete::InsertOutcome::Inserted staged) (:orx::fire-native staged))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
      (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :orx::explain-a1-only-oracle [] -> :wat::rete::Explained
  (:wat::core::match
    (:wat::rete::insert (:orx::compiled) (:orx::A1 :k 1))
    ((:wat::rete::InsertOutcome::Inserted staged) (:orx::fire-oracle staged))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
      (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None))))

;; [native 8-arm via[0] type, oracle same, native A1-only, oracle A1-only]
(:wat::core::defn :user::or-attribution [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::mapv
    (:wat::core::fn [s <- :wat::core::String] -> :wat::core::String s)
    (:wat::core::PersistentVector
      (:orx::via-type (:orx::explain-all))
      (:orx::via-type (:orx::explain-all-oracle))
      (:orx::via-type (:orx::explain-a1-only))
      (:orx::via-type (:orx::explain-a1-only-oracle)))))
