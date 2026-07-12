;; tests/types/probe_arc294_9a_kwargs_ctor.wat — co-located fixture (arc 294 item 9a)
;;
;; CONSTRUCTION ERGONOMICS FLIP: the bare type name is now the KWARGS macro (order-free
;; `(:ns::T :field val …)`); the raw positional ctor moved to the type-name PRIME `:ns::T'`.
;;
;; RED at HEAD (pre-flip): `:probe294a::Pair'` does not exist (bare `:probe294a::Pair` was
;; the positional ctor) — `wat --check` rejects the prime call, `:probe294a::Pair` accepts
;; positional args cleanly.
;; GREEN after item 9a: bare kwargs (either key order) reorders to the prime positional
;; call; the prime accepts raw positional args; bare positional is a LOCATED error (see
;; probe_arc294_9a_kwargs_ctor_bad.wat.bad for the negative fixture).

(:wat::core::defrecord :probe294a::Pair [a <- :wat::core::i64  b <- :wat::core::i64])

;; bare kwargs, declared field order (:a then :b)
(:wat::core::defn :probe294a::mk-ab [] -> :probe294a::Pair
  (:probe294a::Pair :a 1 :b 2))

;; bare kwargs, REVERSED key order — proves order-free reordering
(:wat::core::defn :probe294a::mk-ba [] -> :probe294a::Pair
  (:probe294a::Pair :b 2 :a 1))

;; the PRIME — raw positional construction, the reserved escape hatch
(:wat::core::defn :probe294a::mk-prime [] -> :probe294a::Pair
  (:probe294a::Pair' 1 2))
