;; Stone 118.B2c strike 1 — POSITIVE fixture (the bare `.wat` sibling `call_beside_value` resolves).
;; MUST REGISTER AND DISPATCH.
;;
;; ★★ THIS IS THE LOAD-BEARING HALF. A wall that refused EVERY multi-arm `defclause` would satisfy
;; the negative fixture perfectly and take the language with it. These two rows bound what the wall
;; may refuse.
;;
;;   ROW 1 — DISJOINT arms. The ordinary multi-arity/multi-type shape. Must keep working.
;;
;;   ROW 2 — ★ THE FALLBACK SHAPE, and the one this stone nearly outlawed. A concrete arm declared
;;           FIRST, a type-var catch-all SECOND. The catch-all is a WILDCARD at dispatch
;;           (`is_type_var`), so the two arms' domains INTERSECT — but the later arm still fires for
;;           every non-keyword, so it is reachable and legal.
;;
;;           This is not hypothetical: it is `wat/bracket.wat`'s `thread-enter` and
;;           `process-work-forms`, whose own comment (`:314-316`) names first-match-wins, calls the
;;           generic arm a "PERMISSIVE catch-all", and states that ordering is load-bearing. The
;;           first version of this wall refused INTERSECTION and would have outlawed both.
;;           `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`

;; ROW 1 — disjoint
(:wat::core::defclause :my::describe
  ([x <- :wat::core::i64]    -> :wat::core::String "an int")
  ([x <- :wat::core::String] -> :wat::core::String "a string")
  ([x <- :wat::core::bool]   -> :wat::core::String "a bool"))

(:wat::core::defn :my::describe-int    [] -> :wat::core::String (:my::describe 1))
(:wat::core::defn :my::describe-string [] -> :wat::core::String (:my::describe "s"))
(:wat::core::defn :my::describe-bool   [] -> :wat::core::String (:my::describe true))

;; ROW 2 — concrete-then-catch-all (the bracket.wat shape)
(:wat::core::defclause :my::route
  ([x <- :wat::core::keyword] -> :wat::core::String "specific")
  ([x <- :W]                  -> :wat::core::String "generic"))

(:wat::core::defn :my::route-keyword [] -> :wat::core::String (:my::route :a-keyword))
(:wat::core::defn :my::route-other   [] -> :wat::core::String (:my::route 7))
