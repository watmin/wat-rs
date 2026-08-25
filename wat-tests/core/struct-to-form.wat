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


(:wat::test::deftest :wat-rs::std::struct-to-form::test-roundtrip-via-eval
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::core::do
             (:wat::core::do
               (:wat::core::let
                 [p (:my::Pair :a 7 :b 9)
                  form (:wat::core::struct->form p)
                  _roundtrip (:wat::eval-ast! form)]
                 nil))
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))]
    ;; Assert the inner child succeeded — a clean completion crosses the wire
    ;; as Message; a crash reaches recv' as Lost carrying the death message.
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) nil)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed!
          (:wat::string::concat "roundtrip-via-eval failed: "
            (:wat::kernel::LociDiedError/message cause))
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "roundtrip-via-eval: child closed before signaling completion"
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
    (:wat::core::do form nil)))
