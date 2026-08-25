;; Run-proof for DESIGN-STONE-per-type-equality-restored.md — the four heads
;; restored beside their ordering twins: `:wat::core::i64::=`, `:i64::not=`,
;; `:f64::=`, `:f64::not=`. A `:user::main` so this actually resolves and runs
;; (a probe with no main fails before resolving anything and proves nothing).
;;
;; Prints each result so the run's stdout is the proof, not a read of the source.

(:wat::core::defrecord :my::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "i64::= 1 1        -> " (:wat::core::bool::to-string (:wat::core::i64::= 1 1))))
    (:wat::kernel::println (:wat::string::concat "i64::not= 1 2     -> " (:wat::core::bool::to-string (:wat::core::i64::not= 1 2))))
    (:wat::kernel::println (:wat::string::concat "f64::= 1.5 1.5    -> " (:wat::core::bool::to-string (:wat::core::f64::= 1.5 1.5))))
    (:wat::kernel::println (:wat::string::concat "f64::= 0.0 1.0    -> " (:wat::core::bool::to-string (:wat::core::f64::= 0.0 1.0))))
    (:wat::kernel::println (:wat::string::concat "generic = 1 1     -> " (:wat::core::bool::to-string (:wat::core::= 1 1))))
    (:wat::kernel::println (:wat::string::concat "generic = record  -> " (:wat::core::bool::to-string (:wat::core::= (:my::Pt :x 0 :y 0) (:my::Pt :x 0 :y 0)))))))
