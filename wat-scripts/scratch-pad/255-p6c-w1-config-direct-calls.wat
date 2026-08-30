;; Scratch probe — arc 255 Stone P6-c-W1, acceptance row 4.
;;
;; Direct, statically-checked calls to all four `:wat::config::*` nullary readers,
;; at zero arity (the legal call shape). Output must be byte-identical before and
;; after homing them into the `#[wat_intrinsic]` registry — only the DISPATCH
;; mechanism changes, not the returned value.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "dim-count= " (:wat::edn::write (:wat::config::dim-count))))
    (:wat::kernel::println (:wat::string::concat "dim-capacity= " (:wat::edn::write (:wat::config::dim-capacity))))
    (:wat::kernel::println (:wat::string::concat "global-seed= " (:wat::edn::write (:wat::config::global-seed))))
    (:wat::kernel::println (:wat::string::concat "noise-floor= " (:wat::edn::write (:wat::config::noise-floor))))))
