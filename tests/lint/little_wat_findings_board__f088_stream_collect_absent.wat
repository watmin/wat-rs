;; Board specimen — the-little-wat F-088: `:wat::stream::collect` does not exist.
;;
;; SHAPE: STRICT (absent name) — the checker refuses an unresolved head.
;; ⛔ Do NOT "fix" this file by renaming the head to something that resolves. The finding
;; IS the absent name; a resolving head measures nothing. If `:wat::stream::collect` is
;; ever minted, this row flips and the finding is closed.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do (:wat::stream::collect 1) nil))
