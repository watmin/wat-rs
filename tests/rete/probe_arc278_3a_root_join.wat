;; tests/rete/probe_arc278_3a_root_join.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :user::Temp record used by the root-join tests.

(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])

;; P11: beta is ephemeral by design; a fired Session no longer retains beta-memory — provenance
;; regenerates on re-fire. Join-correctness coverage relocated to:
;;   src/rete/kernel.rs #[cfg(test)]::root_join_seeds_one_token_per_element
;; These entries stay only so the (permanently #[ignore]d) sibling probes have somewhere to point.

(:wat::core::defn :user::beta-populated-count [] -> :wat::core::i64
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))
     fired (:wat::rete::fire-rules sess1)
     bmem  (:wat::rete::Session/beta-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys bmem))))

(:wat::core::defn :user::seeded-token-count [] -> :wat::core::i64
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))
     fired (:wat::rete::fire-rules sess1)
     bmem  (:wat::rete::Session/beta-memory fired)
     rjid (:wat::core::Option/expect -> :wat::core::i64 (:wat::core::get (:wat::core::PersistentMap/keys bmem) 0) "rjid")
     toks (:wat::core::Option/expect -> :wat::core::PersistentVector (:wat::core::PersistentMap/get bmem rjid) "toks")]
    (:wat::core::length toks)))

(:wat::core::defn :user::seeded-token-t-binding [] -> :wat::core::Option<wat::core::i64>
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))
     fired (:wat::rete::fire-rules sess1)
     bmem  (:wat::rete::Session/beta-memory fired)
     rjid (:wat::core::Option/expect -> :wat::core::i64 (:wat::core::get (:wat::core::PersistentMap/keys bmem) 0) "rjid")
     toks (:wat::core::Option/expect -> :wat::core::PersistentVector (:wat::core::PersistentMap/get bmem rjid) "toks")
     tok  (:wat::core::Option/expect -> :wat::rete::Token (:wat::core::get toks 0) "tok")
     binds (:wat::rete::Token/bindings tok)]
    (:wat::core::PersistentMap/get binds "?t")))

(:wat::core::defn :user::seeded-token-support-length [] -> :wat::core::i64
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))
     fired (:wat::rete::fire-rules sess1)
     bmem  (:wat::rete::Session/beta-memory fired)
     rjid (:wat::core::Option/expect -> :wat::core::i64 (:wat::core::get (:wat::core::PersistentMap/keys bmem) 0) "rjid")
     toks (:wat::core::Option/expect -> :wat::core::PersistentVector (:wat::core::PersistentMap/get bmem rjid) "toks")
     tok  (:wat::core::Option/expect -> :wat::rete::Token (:wat::core::get toks 0) "tok")]
    (:wat::core::length (:wat::rete::Token/matches tok))))
