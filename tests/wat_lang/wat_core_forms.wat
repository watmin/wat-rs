;; tests/wat_lang/wat_core_forms.wat — co-located fixture.
;; :wat::core::forms (variadic-quote) and :wat::test::program defmacro.

;; test1: forms captures each arg as WatAST (length 3)
(:wat::core::defn :t::test1-forms-3 [] -> :wat::core::bool
  (:wat::core::let
    [captured (:wat::core::forms (foo 1) (bar 2) (baz 3))
     n        (:wat::core::length captured)]
    (:wat::core::= n 3)))

;; test2: forms() → empty vec (length 0)
(:wat::core::defn :t::test2-forms-empty [] -> :wat::core::bool
  (:wat::core::let
    [captured (:wat::core::forms)
     n        (:wat::core::length captured)]
    (:wat::core::= n 0)))

;; test3: forms args are not evaluated (unevaluated form captured as data)
(:wat::core::defn :t::test3-forms-unevaluated [] -> :wat::core::bool
  (:wat::core::let
    [captured (:wat::core::forms (:this::is::not::a::real::function 1 2 3))
     n        (:wat::core::length captured)]
    (:wat::core::= n 1)))

;; test4: run-hermetic roundtrip via println
(:wat::core::defn :t::test4-run-sandboxed [] -> :wat::core::String
  (:wat::core::let
    [r        (:wat::test::run-hermetic
                (:wat::kernel::println "hello-from-inside"))
     captured (:wat::kernel::RunResult/stdout r)
     line     (:wat::core::first captured)]
    line))

;; test5: :wat::test::program macro expands to forms (length 3)
(:wat::core::defn :t::test5-program-macro [] -> :wat::core::bool
  (:wat::core::let
    [captured (:wat::test::program (a 1) (b 2) (c 3))
     n        (:wat::core::length captured)]
    (:wat::core::= n 3)))

;; test6: run-hermetic roundtrip via test::program
(:wat::core::defn :t::test6-run-ast-hello [] -> :wat::core::String
  (:wat::core::let
    [r        (:wat::test::run-hermetic
                (:wat::kernel::println "hi"))
     captured (:wat::kernel::RunResult/stdout r)
     line     (:wat::core::first captured)]
    line))
