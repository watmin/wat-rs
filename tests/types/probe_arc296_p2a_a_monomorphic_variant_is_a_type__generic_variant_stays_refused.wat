;; ⛔ THE SCOPE FENCE. This stone is MONOMORPHIC only. A generic enum's variant
;; needs argument correspondence across the edge (Option::Some<T> -> Option<T>),
;; which is the one-side-normalized string class. It must STAY refused here, so
;; the stepping stone's boundary is a measured fact and not a promise.
(:wat::core::defn :user::f [s <- :wat::core::Option::Some] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
