;; Arc 139 reproduction — generic-T turbofish at user-defined call site.
;;
;; STATUS (2026-05-03): SHOULD-PANIC pending arc 139 fix.
;; The substrate strips `<T>` at define-registration time
;; (split_name_and_type_params in src/runtime.rs) but does NOT strip
;; at call-site lookup. Asymmetric registration vs lookup → runtime
;; UnknownFunction. Same bug class as arc 102's eval-ast! polymorphic
;; return scheme/runtime alignment.
;;
;; When arc 139 ships, remove the should-panic + restore as a passing
;; deftest. The inferred-T form (tmp-3tuple-inferred.wat) already
;; passes — confirms the bug is specifically in turbofish call-site
;; resolution, not generic dispatch in general.

(:wat::test::should-panic "unknown function")
(:wat::test::deftest :wat-tests::tmp::generic-3tuple-roundtrip
  ((:wat::core::define
     (:test::make-3tuple<T> (mid :T) -> :(wat::core::i64,T,wat::core::String))
     (:wat::core::Tuple 42 mid "hello")))
  (:wat::core::let*
    (((triple :(wat::core::i64,wat::core::bool,wat::core::String))
      (:test::make-3tuple<wat::core::bool> true))
     ((a :wat::core::i64) (:wat::core::first triple))
     ((b :wat::core::bool) (:wat::core::second triple))
     ((c :wat::core::String) (:wat::core::third triple))
     ((_ :wat::core::unit) (:wat::test::assert-eq a 42))
     ((_ :wat::core::unit) (:wat::test::assert-eq b true)))
    (:wat::test::assert-eq c "hello")))
