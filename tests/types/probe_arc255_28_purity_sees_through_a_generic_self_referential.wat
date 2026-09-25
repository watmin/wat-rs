;; Stone 255.28 — the recursion guard: a self-referential generic (`:Link` holds another
;; `(Chain :- [T])`) instantiated over `Transport.Wire` terminates and is accepted.
(:wat::core::defenum :probe::Chain :- [T] :wat::enum::Pure
  :Link [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])
         next <- (:probe::Chain :- [T])]
  :End [])
(:wat::core::defrecord :probe::HoldsChain
  [chain <- (:probe::Chain :- [:wat::kernel::Transport.Wire])])
