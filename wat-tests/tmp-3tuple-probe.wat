;; Arc 139 verification — generic-T turbofish at user-defined call site.
;;
;; This test was the original probe that surfaced arc 139's
;; asymmetric registration vs lookup: the substrate strips `<T,...>`
;; at define-registration (split_name_and_type_params) but DID NOT
;; strip at call-site lookup. Result: turbofish-suffixed call heads
;; failed UnknownFunction at runtime even though the canonical name
;; was registered.
;;
;; Arc 139 fix: `canonical_callable_name` (in src/runtime.rs) strips
;; the turbofish suffix; called from runtime.rs eval_call user-define
;; dispatch, resolve.rs is_resolvable_call_head, and check.rs's
;; infer_list scheme lookup. Symmetric strip; this test now passes.

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
