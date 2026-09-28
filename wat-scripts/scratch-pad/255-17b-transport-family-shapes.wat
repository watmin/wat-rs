;; arc 255 C-b — MEASUREMENT (2026-09-24). Can the language already say "a closed family of
;; phantom transport markers"? A Pure unit-variant enum's VARIANTS used as TYPE arguments:
;; accepted in position, kept apart from each other, assignable to the family.
(:wat::core::defenum :probe::Tr :wat::enum::Pure
  :Sh []
  :Wi [])
(:wat::core::defrecord :probe::Box :- [X] [tags <- (wat.type/Vector :- [X])])
(:wat::core::defn :probe::mk-wi [] -> (:probe::Box :- [:probe::Tr.Wi])
  (:probe::Box :tags (wat.type/Vector :- [:probe::Tr.Wi])))
(:wat::core::defn :probe::takes-wi [b <- (:probe::Box :- [:probe::Tr.Wi])] -> wat.type/i64
  (:wat::core::count (:probe::Box/tags b)))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:probe::takes-wi (:probe::mk-wi))))
