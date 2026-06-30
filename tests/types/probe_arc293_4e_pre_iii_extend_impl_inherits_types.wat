;; 293.4e-pre.iii own-probe — a GENERIC `extend-type` impl on a surface: its bare body uses the surface
;; method's TYPE-PARAM (T) — both in an arg (x <- :T) and the return (:t::Box<T>). This is the shape
;; `:wat::spawn::Locus`'s generic `launch<S,R,St,Sh,Lu>` impls need (the body references S,R,St,Sh,Lu).
;;
;; RED at HEAD (post-293.4c): the surface-extend scheme (check.rs:8957) hardcodes `type_params: vec![]`
;; AND uses the bare impl clause's nil types — so the impl body's `T` is unbound + `self`/args mistyped,
;; exactly the Locus failure (self: :(), ReturnTypeMismatch). The monomorphic constant-body case worked,
;; so this generic typed-body case is the real gap.
;;
;; GREEN at 293.4e-pre.iii: the scheme inherits the surface member's sig (self → extending type, args + ret
;; from the member, type_params carried) so `T` resolves in the body.

(:wat::core::defsurface :t::Maker
  :holder :wat::core::Struct
  :features [(make<T> [self <- :t::Maker  x <- :T] -> :t::Box<T>)])

(:wat::core::defrecord :t::Box<T> [v <- :T])
(:wat::core::defrecord :t::Id [tag <- :wat::core::i64])

;; bare GENERIC impl — body wraps x (typed :T from the surface) in a :t::Box<T>.
(:wat::core::extend-type :t::Id :t::Maker
  (make [self x] (:t::Box x)))

(:wat::core::defn :t::probe [] -> :wat::core::i64 (:t::Box/v (:t::Maker/make (:t::Id 7) 42)))
