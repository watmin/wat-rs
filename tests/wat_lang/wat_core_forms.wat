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

;; test4: primed-peer roundtrip via println (arc 278 IPC de-prime — migrated off
;; run-hermetic onto spawn-program' (process) + recv'). The child println's a String;
;; the parent drains that single value off the peer as a RecvOutcome::Message. The
;; value crosses the wire DECODED (a String, not the EDN-quoted stdout line the old
;; RunResult/stdout captured), so it is "hello-from-inside" without the outer quotes.
(:wat::core::defn :t::test4-run-sandboxed [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "hello-from-inside"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "test4: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "test4: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; test5: :wat::test::program macro expands to forms (length 3)
(:wat::core::defn :t::test5-program-macro [] -> :wat::core::bool
  (:wat::core::let
    [captured (:wat::test::program (a 1) (b 2) (c 3))
     n        (:wat::core::length captured)]
    (:wat::core::= n 3)))

;; test6: primed-peer roundtrip (arc 278 IPC de-prime — migrated off run-hermetic onto
;; spawn-program' (process) + recv'). Same wire as test4; the child println's "hi" and
;; the parent recv's it as a decoded String Message ("hi", no EDN quotes).
(:wat::core::defn :t::test6-run-ast-hello [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "hi"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "test6: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "test6: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
