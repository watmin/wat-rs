;; Board specimen — the-little-wat F-045: `first` of an EMPTY Vector.
;;
;; SHAPE: LENIENT — the checker ACCEPTS, the runtime KILLS. Their own relay files this
;; finding under BOTH "Fix" (refuse at the checker) and "Extend" (make first/rest total,
;; like `last`) — opposite languages, so the board measures it and rules nothing.
;; ⛔ Do NOT "fix" this file; the finding is that wat accepts it.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::first (:wat::core::Vector :- [:wat::core::i64]))))
