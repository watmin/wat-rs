;; Scratch probe — arc 255 Stone P6-c-W6, acceptance row 5.
;;
;; One direct call per verb on a NON-trivial collection (3+ elements, not the empty case),
;; run through `wat --check` first (must stay clean) then executed. Output compared byte-for-
;; byte against a real HEAD clone (before homing) — mirrors `255-p6c-w5c-direct-calls.wat`.
;; `:wat::edn::write` requires an explicit `(Head :- [T])` param-spec on a literal Vector (an
;; edn::write-specific requirement, unrelated to any of the seven verbs — confirmed by an
;; isolated repro of `(:wat::edn::write (:wat::core::Vector 1 2 3))` alone failing the same way).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::edn::write (:wat::core::length (:wat::core::Vector :- [:wat::core::i64] 10 20 30 40))))
    (:wat::kernel::println (:wat::edn::write (:wat::core::empty? (:wat::core::Vector :- [:wat::core::i64] 10 20 30 40))))
    (:wat::kernel::println (:wat::edn::write (:wat::core::nth (:wat::core::Vector :- [:wat::core::i64] 10 20 30 40) 2)))
    (:wat::kernel::println (:wat::edn::write (:wat::core::last (:wat::core::Vector :- [:wat::core::i64] 10 20 30 40))))
    (:wat::kernel::println (:wat::edn::write (:wat::core::rest (:wat::core::Vector :- [:wat::core::i64] 10 20 30 40))))
    (:wat::kernel::println (:wat::edn::write (:wat::core::reverse (:wat::core::Vector :- [:wat::core::i64] 10 20 30 40))))
    (:wat::kernel::println (:wat::edn::write (:wat::core::range 0 5)))
    ;; A second receiver kind per gated verb, to exercise more than Vector alone.
    (:wat::kernel::println (:wat::edn::write (:wat::core::length (:wat::core::List 1 2 3))))
    (:wat::kernel::println (:wat::edn::write (:wat::core::nth (:wat::core::PersistentVector 5 6 7) 1)))
    (:wat::kernel::println (:wat::edn::write (:wat::core::rest (:wat::core::List 1 2 3))))
    (:wat::kernel::println (:wat::edn::write (:wat::core::reverse (:wat::core::PersistentVector 1 2 3))))))
