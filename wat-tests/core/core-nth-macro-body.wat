;; wat-tests/core/core-nth-macro-body.wat — stone 118.B4-0 row 1: a `defmacro` PROGRAM BODY can
;; call `(nth ...)`.
;;
;; Before this stone this was IMPOSSIBLE: `nth` was a wat-defined `defclause`, and a macro's
;; expand-time program body evaluates only through `dispatch_keyword_head` (the Rust intrinsic
;; dispatcher) — which never sees wat-defined names. `(first (drop X n))` worked in that position
;; because both halves are intrinsics; `nth` was not one. B4-ii's codemod rewrote 44 call sites to
;; `(nth X n)`, and one of them (the exact shape below, mirroring `wat/service.wat:468`'s
;; `(first (drop init-fn-ch 4))` idiom — extracting a positional child of an argument form at
;; macro-expand time) landed inside a `defservice` macro body and the stdlib stopped loading.
;; This is the WHOLE REASON the stone exists — it must go from impossible to green.
;;
;; The macro returns a raw `:wat::WatAST` node (the second child of its argument form), so the
;; call site's expansion IS that node, spliced in and then evaluated as ordinary code — the
;; asserted value proves the macro-body `nth` call ran (expand-time) AND selected the right
;; child, not merely that the file loaded.

(:wat::core::defmacro :wat-tests::core::core-nth-macro-body::second-child
  [form <- :wat::WatAST] -> :wat::WatAST
  `~(:wat::core::nth (:wat::core::ast->children form) 1))

(:wat::test::deftest :wat-tests::core::core-nth-macro-body::macro-body-can-call-nth
  (:wat::test::assert-eq
    (:wat-tests::core::core-nth-macro-body::second-child `(:tag 20 30))
    20))
