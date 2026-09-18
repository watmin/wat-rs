;; rune:lint(red-by-design) — the invented head inside :user::go's own body is refused, as expected; paired with p1 to show this tree's resolver does not distinguish a forced call site from an unforced one
;; VIGILIA experiri — the SAME invented head, in the body that IS applied.
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::VIGILIA-NEVER-EXISTED-ANYWHERE 1 2))
