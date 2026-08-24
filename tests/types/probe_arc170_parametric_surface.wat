;; Arc 170 C2 — parametric defsurface: a `defsurface :Name :- [T]` method whose return type `:T`
;; resolves to the CONCRETE type bound by the satisfier's `extend-type :Satisfier (:Name :- [Concrete])`.
;; Minimal model: (Holds :- [T]) with (get [self] -> :T); IntBox binds T = i64.
;; POSITIVE: `:probe::resolve` returns `(Holds/get b)` directly — proves the call-site's resolved
;; return type is genuinely i64 (not bare/any), since a mismatched declared return would be a
;; located ReturnTypeMismatch at startup, and the runtime value itself is asserted to be i64(42).
;; Driven via `invoke_user_main` (not a `parse_one!`-string) so this test inlines no wat.

(:wat::core::defsurface :probe::Holds :- [T] :nature :wat::core::Struct
  :features
  [(get [self <- (:probe::Holds :- [T])] -> :T)])

(:wat::core::defrecord :probe::IntBox [n <- :wat::core::i64])
(:wat::core::extend-type :probe::IntBox (:probe::Holds :- [:wat::core::i64])
  (get [self] (:probe::IntBox/n self)))

(:wat::core::defn :probe::resolve [] -> :wat::core::i64
  (:wat::core::let [b (:probe::IntBox :n 42)]
    (:probe::Holds/get b)))
