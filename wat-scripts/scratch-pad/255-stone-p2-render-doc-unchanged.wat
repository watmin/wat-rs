;; Scratch probe — arc 255 Stone P2, acceptance row 3.
;;
;; render-doc IS UNCHANGED. `(:wat::core::render-doc :wat::core::if)` must be
;; byte-identical before and after this stone's fold/reflect.rs changes — it
;; already derives its "Syntax:" line correctly from `entry.args`
;; (reflect.rs:349-356), and this stone touches neither that derivation nor
;; `entry.args` itself.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::render-doc :wat::core::if))
    (:wat::kernel::println (:wat::core::render-doc :wat::core::let))))
