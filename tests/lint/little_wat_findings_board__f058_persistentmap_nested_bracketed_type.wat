;; Board specimen — the-little-wat F-058: a `PersistentMap` constructor refuses a NESTED
;; bracketed type argument.
;;
;; SHAPE: STRICT — the checker REFUSES, so the binary never evaluates.
;; ⛔ Do NOT "fix" this file. The finding is the refusal; a file edited to check clean
;; stops measuring it.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::length
    (:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])]))))
