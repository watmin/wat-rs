;; Excursus 003 step 4 (D4) — G1, C-114 is retired.
;;
;; the-little-wat FINDINGS.md § C-114 (restating F-006/F-008): an i64 overflow inside
;; `(:wat::core::+ ...)` "locates at wat/core.wat:66 rather than at the line that
;; overflowed." `:user::grow` calls the public alias `:wat::core::+` in TAIL position,
;; so its own frame is replaced by the tail call's (the elision itself is out of scope
;; for this strike) — the call site still survives as frame 0, which is exactly what
;; the derivation reads.
(:wat::core::defn :user::grow [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::+ n 9223372036854775807))

(:wat::core::defn :my::test::probe-overflow [] -> :wat::core::i64
  (:user::grow 41))
