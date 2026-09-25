;; Stone 255.28, inverted by 255.29. The recursion guard still walks the
;; self-referential generic. Over `Transport.Shared` that walk used to find an
;; impure address and refuse. Pre-255.29 this file was `.wat.bad`. 255.29: the
;; address is data, so the chain is pure and the walk terminates accepted.
(:wat::core::defenum :probe::Chain :- [T] :wat::enum::Pure
  :Link [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])
         next <- (:probe::Chain :- [T])]
  :End [])
(:wat::core::defrecord :probe::HoldsChain
  [chain <- (:probe::Chain :- [:wat::kernel::Transport.Shared])])
