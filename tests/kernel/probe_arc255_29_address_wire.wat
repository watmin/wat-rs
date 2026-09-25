;; Stone 255.29 — address-wire? stays false for a thread address.
;; The process twin stays true: portable-for-a-process is the socket tier.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [tb (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     ta (:wat::spawn::Bound/address tb)
     pb (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
     pa (:wat::spawn::Bound/address pb)]
    (:wat::core::do
      (:wat::kernel::println (:wat::kernel::address-wire? ta))
      (:wat::kernel::println (:wat::kernel::address-wire? pa)))))
