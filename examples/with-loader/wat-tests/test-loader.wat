;; examples/with-loader/wat-tests/test-loader.wat — arc 017 slice 2's
;; walkable proof that `wat::test! { ..., loader: "..." }`
;; threads a ScopedLoader into each test file's freeze, so the
;; entry-file `(:wat::load-file! "helpers.wat")`
;; resolves.
;;
;; The actual test body is trivial (1 + 1 == 2). The proof point is
;; upstream of the body: without a loader on the suite, the load
;; below fails with NotFound at freeze time; with one, the helper
;; file's define lands in this file's frozen world and the suite
;; runs. (Deftest bodies run in hermetic sandboxes that don't inherit
;; the outer file's loads — the helper define is not visible inside
;; the sandbox, just registered into the file's world as a freeze-
;; time side effect.)


(:wat::load-file! "helpers.wat")

(:wat::test::deftest :user::with_loader::test::test-loader-wiring
  ()
  (:wat::test::assert-eq (:wat::core::+ 1 1) 2))
