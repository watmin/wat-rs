;; tests/rete/probe_arc278_alpha_is_fire_scoped.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()).
;;
;; Mirrors probe_arc278_2b_insert_alpha.wat's smallest alpha-populating workload (:user::Temp +
;; `(> ?t 20)`), extended with a non-empty RHS (2b's rule had an empty :rhs, deriving nothing) so a
;; derived-fact differential exists alongside the alpha-key-count differential.

(:wat::core::defrecord :afs::Temp [value <- :wat::core::i64])
(:wat::core::defrecord :afs::Hot  [value <- :wat::core::i64])

(:wat::rete::defquery :afs::q-Hot
  :params []
  :when [(?fact <- :afs::Hot)])


;; One condition, one matching fact (25) and one non-matching fact (15, fails > 20); RHS derives
;; :afs::Hot from the matching fact only.
(:wat::core::defn :afs::built [] -> :wat::rete::Session
  (:wat::core::let
    [cond  (:wat::core::quote (:afs::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rhs1  (:wat::core::quote (:afs::Hot ?t))
     rule  (:wat::rete::Rule :name "afs" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:afs::q-Hot)))
     sess1 (:wat::rete::insert sess0 (:afs::Temp :value 25))
     sess2 (:wat::rete::insert sess1 (:afs::Temp :value 15))]
    sess2))

;; (1) native-alpha-key-count — fired via native fixpoint `fire-rules`. Expect 0: the clear happened.
(:wat::core::defn :user::native-alpha-key-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:wat::rete::fire-rules (:afs::built))
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (2) oracle-alpha-key-count — fired via `fire-rules-spec` (the wat ORACLE, never optimized). Expect
;; 0: `fire-stratified` returns alpha-memory empty (rete.wat:1817-1820) — asserted here, not assumed.
(:wat::core::defn :user::oracle-alpha-key-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:wat::rete::fire-rules-spec (:afs::built))
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (4) single-pass-alpha-key-count — fired via native `fire-once'` (single-pass). Expect > 0: THE
;; ANCHOR — proves this workload really does populate alpha, so (1)/(2)/(3) are not vacuously true
;; over a workload that matches nothing. `fire-once'` is deliberately left untouched by this stone.
(:wat::core::defn :user::single-pass-alpha-key-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:wat::rete::fire-once' (:afs::built))
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (5) native-derived-count / oracle-derived-count — the RESULT (production output), expected equal
;; and > 0: closing the alpha divergence must not move what fire actually derives.
(:wat::core::defn :user::native-derived-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::rete::fire-rules (:afs::built)) (:afs::q-Hot))))

(:wat::core::defn :user::oracle-derived-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::rete::fire-rules-spec (:afs::built)) (:afs::q-Hot))))
