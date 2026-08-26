;; tests/rete/probe_arc278_6b_eval_test.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines :test::big? for the eval-test probe.

(:wat::core::defn :test::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::> n 100))

;; just-eval entry points — `:wat::rete::eval-test` over a token's merged bindings.

;; 1 — a true comparison over bindings → true.
(:wat::core::defn :user::comparison-true [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::core::> ?a ?b))
    (:wat::map::assoc (:wat::map::assoc (:wat::core::PersistentMap) "?a" 5) "?b" 3)))

;; 2 — a false comparison over bindings → false.
(:wat::core::defn :user::comparison-false [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::core::> ?a ?b))
    (:wat::map::assoc (:wat::map::assoc (:wat::core::PersistentMap) "?a" 3) "?b" 5)))

;; 3 — a pure intrinsic predicate (string::starts-with?) over a string binding.
(:wat::core::defn :user::string-predicate-over-binding [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::string::starts-with? ?path "/admin"))
    (:wat::map::assoc (:wat::core::PersistentMap) "?path" "/admin/x")))

;; 4 — a COMPUTED operand `(> (- ?hi ?lo) 10)` → true (the "any pure expr" proof, not just a 2-var cmp).
(:wat::core::defn :user::computed-operand-true [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::core::> (:wat::core::- ?hi ?lo) 10))
    (:wat::map::assoc (:wat::map::assoc (:wat::core::PersistentMap) "?hi" 20) "?lo" 5)))

;; 5 — the same computed operand, false branch.
(:wat::core::defn :user::computed-operand-false [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::core::> (:wat::core::- ?hi ?lo) 10))
    (:wat::map::assoc (:wat::map::assoc (:wat::core::PersistentMap) "?hi" 12) "?lo" 5)))

;; 6 — a USER-defined predicate over a binding (THE load-bearing case: filter with your own fn).
(:wat::core::defn :user::user-fn-predicate [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:test::big? ?x))
    (:wat::map::assoc (:wat::core::PersistentMap) "?x" 150)))

;; 7 — a non-bool result is a TypeMismatch (a `where` must be a predicate). Declared bool per eval-test's
;; contract; the raise happens at runtime when the quoted expr's actual result is non-bool.
(:wat::core::defn :user::non-bool-result-is-error [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::core::+ ?a ?b))
    (:wat::map::assoc (:wat::map::assoc (:wat::core::PersistentMap) "?a" 1) "?b" 2)))

