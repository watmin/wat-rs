;; tests/rete/probe_arc278_1a_data_model.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). just-eval entry points — hand-build a 2-node Session
;; (RootJoinNode id0 -> ProductionNode id1) and expose the two probe assertions the sibling .rs
;; makes on it. (No records needed; the stone-0 Session/Node data model is on rete.wat's own types.)

(:wat::core::defn :user::network-length [] -> :wat::core::i64
  (:wat::core::let
    [n0 (:wat::rete::RootJoinNode :id 0 :children (:wat::core::PersistentVector 1) :binding-keys (:wat::core::PersistentVector))
     n1 (:wat::rete::ProductionNode :id 1 :rule-name "rule-1")
     net (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap [:wat::core::i64 :wat::core::Record]) 0 n0) 1 n1)
     em  (:wat::core::PersistentMap)
     ev  (:wat::core::PersistentVector)
     s   (:wat::rete::Session :network net :rules ev :alpha-memory em :beta-memory em :production-memory em :facts ev :next-id 2 :query-memory em)]
    (:wat::core::PersistentMap/length (:wat::rete::Session/network s))))

(:wat::core::defn :user::render-dag-of-session [] -> :wat::core::String
  (:wat::core::let
    [n0 (:wat::rete::RootJoinNode :id 0 :children (:wat::core::PersistentVector 1) :binding-keys (:wat::core::PersistentVector))
     n1 (:wat::rete::ProductionNode :id 1 :rule-name "rule-1")
     net (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap [:wat::core::i64 :wat::core::Record]) 0 n0) 1 n1)
     em  (:wat::core::PersistentMap)
     ev  (:wat::core::PersistentVector)
     s   (:wat::rete::Session :network net :rules ev :alpha-memory em :beta-memory em :production-memory em :facts ev :next-id 2 :query-memory em)]
    (:wat::rete::render-dag s)))
