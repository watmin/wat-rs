;; 293.4e-pre.iii own-probe — a GENERIC `extend-type` impl on a surface: its bare body uses the surface
;; method's TYPE-PARAM (T) — both in an arg (x <- :T) and the return (:t::Box<T>). This is the shape
;; `:wat::spawn::Locus`'s generic `launch<S,R,St,Sh,Lu>` impls need (the body references S,R,St,Sh,Lu).
;;
;; GREEN on current HEAD: the bare generic extend-impl inherits the surface member's sig (self → extending
;; type, x → :T, ret → :t::Box<T>) so `T` resolves in the body. The capability landed via 293.4e-pre.ii
;; (generic surface-method call-site instantiation, `c62a817c`) + the Clause→ArgSpec heresy fix (`7d983012`).
;;
;; (:t::Maker/make (:t::Id 7) 42) → (:t::Box 42) → (:t::Box/v …) = 42. The body wraps `x` (42), not the tag.

(:wat::core::defsurface :t::Maker
  :nature :wat::core::Struct
  :features [(make<T> [self <- :t::Maker  x <- :T] -> (:t::Box :- [T]))])

(:wat::core::defrecord :t::Box :- [T] [v <- :T])
(:wat::core::defrecord :t::Id [tag <- :wat::core::i64])

;; bare GENERIC impl — body wraps x (typed :T from the surface) in a :t::Box<T>.
(:wat::core::extend-type :t::Id :t::Maker
  (make [self x] (:t::Box :v x)))

(:wat::core::defn :t::probe [] -> :wat::core::i64 (:t::Box/v (:t::Maker/make (:t::Id :tag 7) 42)))
