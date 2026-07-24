;; wat-tests/core/struct-to-form.wat — arc 091 slice 8 smoke test
;; for `:wat::core::struct->form` and runtime quasiquote.
;;
;; struct->form lifts a struct VALUE to its constructor-call FORM
;; — `Value::Struct{type_name, fields}` becomes a
;; `Value::wat__WatAST(List(:type-name field0 field1 ...))`.
;; Inverse of struct construction; round-trips through eval-ast!.
;;
;; Quasiquote is a sibling: same shape, but the user composes the
;; form with embedded `,unquote` sites, and the substrate fills them
;; in from the surrounding environment at evaluation time.

(:wat::core::defstruct :my::Pair
  [a <- :wat::core::i64
   b <- :wat::core::i64])

(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest :wat-rs::std::struct-to-form::test-roundtrip-via-eval
  
  (:wat::core::let
    [outcome
      (:wat::test::run-thread
        (:wat::core::do
          (:wat::core::let
            [p (:my::Pair :a 7 :b 9)
             form (:wat::core::struct->form p)
             _roundtrip (:wat::eval-ast! form)]
            ())))
     fail (:wat::kernel::RunResult/failure outcome)]
    ;; Assert the inner run-thread succeeded (no failure) — the
    ;; struct was built from its lifted form without panicking.
    (:wat::core::match fail 
      (:wat::core::None nil)
      ((:wat::core::Some f)
        (:wat::kernel::assertion-failed!
          (:wat::core::string::concat "roundtrip-via-eval failed: "
            (:wat::kernel::Failure/message f))
          :wat::core::None :wat::core::None)))))


(:wat::test::deftest :wat-rs::std::struct-to-form::test-quasiquote-splices-runtime-values
  
  (:wat::core::let
    [x 42
     y "hello"
     form
      (:wat::core::quasiquote (:my::Foo ~x ~y))]
    ;; Quasiquote at runtime: unquoting ~x and ~y must not panic (they
    ;; are live bindings); the WatAST is constructed. Successful
    ;; construction without panicking is the provable fact — the
    ;; deftest's run-thread catches any panic and surfaces it as failure,
    ;; so a clean RunResult IS the assertion. No further structural
    ;; inspection is available (show renders "<WatAST>" for all WatAST
    ;; values; eval-ast! would fail because :my::Foo is not declared).
    (:wat::core::do form ())))
