;; rune:lint(red-by-design) — this tree's resolver checks EVERY defn body's call heads regardless of whether anything calls the function; the invented head is refused here, unlike the gap grok's own tree was calibrating for
;; VIGILIA experiri — a freshly invented head under :wat::core::, in a call position
;; inside a `defn` body NOTHING forces.
(:wat::core::defn :vph::dead [] -> :wat::core::i64
  (:wat::core::VIGILIA-NEVER-EXISTED-ANYWHERE 1 2))
(:wat::core::defn :user::go [] -> :wat::core::i64 7)
