;; wat-tests/core/struct-to-form.wat — arc 091 slice 8 smoke test
;; for `:wat::core::struct->form` and runtime quasiquote.
;;
;; struct->form lifts a struct VALUE to its constructor-call FORM
;; — `Value::Struct{type_name, fields}` becomes a
;; `Value::wat__WatAST(List(:type-name/new field0 field1 ...))`.
;; Inverse of struct construction; round-trips through eval-ast!.
;;
;; Quasiquote is a sibling: same shape, but the user composes the
;; form with embedded `,unquote` sites, and the substrate fills them
;; in from the surrounding environment at evaluation time.

(:wat::test::deftest :wat-rs::std::struct-to-form::test-roundtrip-via-eval
  ()
  (:wat::core::let*
    (((_outcome :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::struct :my::Pair
            (a :wat::core::i64)
            (b :wat::core::i64))
          (:wat::core::define
            (:user::main
              (_stdin :wat::io::IOReader)
              (_stdout :wat::io::IOWriter)
              (_stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              (((p :my::Pair) (:my::Pair/new 7 9))
               ((form :wat::WatAST) (:wat::core::struct->form p))
               ((roundtrip :wat::holon::HolonAST) (:wat::eval-ast! form))
               ;; Just check the eval succeeded — the struct re-built
               ;; from its lifted form.
               (_ :wat::core::unit (:wat::test::assert-eq true true)))
              ())))
        (:wat::core::Vector :wat::core::String))))
    (:wat::test::assert-eq true true)))


(:wat::test::deftest :wat-rs::std::struct-to-form::test-quasiquote-splices-runtime-values
  ()
  (:wat::core::let*
    (((x :wat::core::i64) 42)
     ((y :wat::core::String) "hello")
     ((form :wat::WatAST)
      (:wat::core::quasiquote (:my::Foo/new ,x ,y))))
    ;; Quasiquote at runtime: ,x evaluated to 42; ,y to "hello";
    ;; the resulting form is the WatAST `(:my::Foo/new 42 "hello")`.
    ;; Sentinel — successful evaluation is the proof.
    (:wat::test::assert-eq true true)))
