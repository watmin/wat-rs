;; 293.4e-pre.ii own-probe — a GENERIC surface method member `(make<T> [self … x <- :T] -> :T)`
;; must parse with the type-params stripped off the name, and dispatch at the call site with the
;; type-params instantiated (parity with arc-267 generic *protocol* methods). This is the last gate
;; before `:wat::spawn::Locus`'s `launch<S,R,St,Sh,Lu>` can migrate `defprotocol` → `defsurface`.
;;
;; RED at HEAD (post-293.4e-pre.i): `parse_method_member_sig` hardcodes `type_params: vec![]` and does
;; NOT split `<T>` off the name → the member is stored as `"make<T>"`, the call `:t::Maker/make` is
;; `"make"`, they don't match → `unknown callee: :t::Maker/make`.
;;
;; GREEN at 293.4e-pre.ii: the surface method name splits to `make` + type_params `[T]` (like
;; `parse_defprotocol_form`), and the call-site check instantiates `T`.

(:wat::core::defsurface :t::Maker
  :nature :wat::core::Struct
  :features [(make :- [T] [self <- :t::Maker  x <- :T] -> :T)])

(:wat::core::defrecord :t::Id [tag <- :wat::core::i64])

;; extend-type impl: bare name (no <T>), bare args — exactly the Locus extend-impl shape.
(:wat::core::extend-type :t::Id :t::Maker
  (make [self x] x))

(:wat::core::defn :t::probe [] -> :wat::core::i64 (:t::Maker/make (:t::Id :tag 1) 42))
