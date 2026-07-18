;; tests/rete/probe_arc278_1b_compile.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). just-eval entry points: `compile` a rule-set into a
;; connected, node-shared DAG and render it (the two scenarios the sibling .rs asserts on).

;; Two rules; FIRST condition identical (c1), second divergent (c2a vs c2b). Proves node SHARING
;; (a shared prefix collapses to one alpha + one root-join) and wired edges (the shared root-join
;; fans out to both divergent hash-joins).
(:wat::core::defn :user::compile-shared-prefix [] -> :wat::core::String
  (:wat::core::let
    [c1  (:wat::core::quote (:Temperature (= ?t :value)))
     c2a (:wat::core::quote (:Humidity    (= ?h :value)))
     c2b (:wat::core::quote (:Pressure    (= ?p :value)))
     rA  (:wat::rete::Rule :name "rA" :lhs (:wat::core::PersistentVector c1 c2a) :rhs (:wat::core::PersistentVector))
     rB  (:wat::rete::Rule :name "rB" :lhs (:wat::core::PersistentVector c1 c2b) :rhs (:wat::core::PersistentVector))
     sess (:wat::rete::compile (:wat::core::PersistentVector rA rB))]
    (:wat::rete::render-dag sess)))

;; One single-condition rule → alpha → root-join → production, fully connected.
(:wat::core::defn :user::compile-single-rule [] -> :wat::core::String
  (:wat::core::let
    [c1 (:wat::core::quote (:Temperature (= ?t :value)))
     rC (:wat::rete::Rule :name "rC" :lhs (:wat::core::PersistentVector c1) :rhs (:wat::core::PersistentVector))
     sess (:wat::rete::compile (:wat::core::PersistentVector rC))]
    (:wat::rete::render-dag sess)))
