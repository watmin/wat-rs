;; tests/wat_lang/wat_arc143_manipulation.wat — co-located fixture.
;; Arc 143 slice 3 — HolonAST manipulation primitives:
;;   :wat::runtime::rename-callable-name
;;   :wat::runtime::extract-arg-names
;; Named functions return String or bool so eval_in_frozen can assert values.
;; Runtime-error cases (:t::test3-* and :t::test7-*) intentionally err at eval.

;; Helper functions used by the tests (unique names to avoid conflicts).
(:wat::core::defn :t::my-double [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::* x 2))
(:wat::core::defn :t::my-neg [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::- 0 n))
(:wat::core::defn :t::my-add [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::+ x y))
(:wat::core::defn :t::constant [] -> :wat::core::i64 42)

;; ─── :wat::runtime::rename-callable-name ────────────────────────────────────

;; test1: rename :wat::core::foldl → :wat::list::reduce — verify "reduce" + type params.
(:wat::core::defn :t::test1-rename-foldl-to-reduce [] -> :wat::core::String
  (:wat::core::let
    [sig     (:wat::core::Option/expect
               (:wat::runtime::signature-of-defn :wat::core::foldl)
               "expected Some")
     renamed (:wat::runtime::rename-callable-name
               sig :wat::core::foldl :wat::list::reduce)
     rendered (:wat::edn::write renamed)]
    rendered))

;; test2: rename user function with no type params; verify new name appears.
(:wat::core::defn :t::test2-rename-no-type-params [] -> :wat::core::String
  (:wat::core::let
    [sig     (:wat::core::Option/expect
               (:wat::runtime::signature-of-defn :t::my-double)
               "expected Some")
     renamed (:wat::runtime::rename-callable-name
               sig :t::my-double :t::my-triple)
     rendered (:wat::edn::write renamed)]
    rendered))

;; test3: runtime error — from-name mismatch (intentionally errors at eval).
(:wat::core::defn :t::test3-rename-mismatch [] -> :wat::core::String
  (:wat::core::let
    [sig     (:wat::core::Option/expect
               (:wat::runtime::signature-of-defn :t::my-neg)
               "expected Some")
     renamed (:wat::runtime::rename-callable-name
               sig :t::wrong-name :t::alias)]
    (:wat::edn::write renamed)))

;; ─── :wat::runtime::extract-arg-names ───────────────────────────────────────

;; test4: foldl has 3 synthetic args :_a0, :_a1, :_a2.
(:wat::core::defn :t::test4-extract-foldl-names [] -> :wat::core::String
  (:wat::core::let
    [sig   (:wat::core::Option/expect
             (:wat::runtime::signature-of-defn :wat::core::foldl)
             "expected Some")
     names (:wat::runtime::extract-arg-names sig)]
    (:wat::edn::write names)))

;; test5: zero-arg function → empty Vec; edn::write of length.
(:wat::core::defn :t::test5-extract-zero-args [] -> :wat::core::String
  (:wat::core::let
    [sig   (:wat::core::Option/expect
             (:wat::runtime::signature-of-defn :t::constant)
             "expected Some")
     names (:wat::runtime::extract-arg-names sig)
     len   (:wat::core::length names)]
    (:wat::edn::write len)))

;; test6: two-arg function → exactly 2 names, stops before return type.
(:wat::core::defn :t::test6-extract-stops-before-return [] -> :wat::core::String
  (:wat::core::let
    [sig      (:wat::core::Option/expect
                (:wat::runtime::signature-of-defn :t::my-add)
                "expected Some")
     names    (:wat::runtime::extract-arg-names sig)
     len      (:wat::core::length names)
     rendered (:wat::edn::write names)]
    (:wat::string::concat
      (:wat::edn::write len)
      " "
      rendered)))

;; test7: runtime error — non-Bundle input to extract-arg-names (intentionally errors).
(:wat::core::defn :t::test7-extract-non-bundle-err [] -> :wat::core::String
  (:wat::core::let
    [leaf  (:wat::holon::to-holon :user::foo)
     names (:wat::runtime::extract-arg-names leaf)]
    (:wat::edn::write names)))

;; ─── Composition test ────────────────────────────────────────────────────────

;; test8: rename-callable-name ∘ signature-of-defn; extract-arg-names still gives 2.
(:wat::core::defn :t::test8-rename-then-extract [] -> :wat::core::String
  (:wat::core::let
    [sig      (:wat::core::Option/expect
                (:wat::runtime::signature-of-defn :t::my-add)
                "expected Some")
     renamed  (:wat::runtime::rename-callable-name
                sig :t::my-add :t::my-sum)
     names    (:wat::runtime::extract-arg-names renamed)
     len      (:wat::core::length names)
     rendered (:wat::edn::write names)]
    (:wat::string::concat
      (:wat::edn::write len)
      " "
      rendered)))
