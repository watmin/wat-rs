;; Excursus 003 stone H — measurement (a), non-generic: does the checker refuse a process-tier
;; `after` whose msg type is a handle, before runtime?
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru")
     _p (:wat::kernel::after :wat::program::PeerKind.process (:wat::time::Millisecond 5) h)]
    nil))
