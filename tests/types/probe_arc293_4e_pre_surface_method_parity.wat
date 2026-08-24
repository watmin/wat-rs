;; 293.4e-pre own-probe — surface-method dispatch must handle methods with args BEYOND self
;; (and, the generic case, type-params) — the shape `:wat::spawn::Locus`'s `launch` needs before
;; `defprotocol` can be annihilated (293.4e migrates Locus → defsurface).
;;
;; RED at HEAD: 293.4b/c/d only ever exercised `[self]`-only method members. A method with a
;; second arg fails the surface-method arity check — `:t::Maker/make: expected 3 argument(s); got 2`
;; (self is double-counted). The generic form `make :- [T]` is worse: `unknown callee :t::Maker/make`.
;;
;; GREEN at 293.4e-pre: a surface method `(make [self … extra-args …] -> ret)` dispatches with the
;; right arity (and the generic `make :- [T]` resolves), at parity with arc-267 generic protocol methods.

(:wat::core::defsurface :t::Maker
  :nature :wat::core::Struct
  :features [(make [self <- :t::Maker  x <- :wat::core::i64] -> :wat::core::i64)])

(:wat::core::defrecord :t::Id [tag <- :wat::core::i64])

(:wat::core::extend-type :t::Id :t::Maker
  (make [self x] x))

(:wat::core::defn :t::use [m <- :t::Maker] -> :wat::core::i64 (:t::Maker/make m 42))

(:wat::core::defn :t::probe [] -> :wat::core::i64 (:t::use (:t::Id :tag 1)))
