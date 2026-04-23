;; examples/with-loader/wat-tests/helpers.wat — test-local library
;; loaded by sibling test files via the arc 017 `loader:` option on
;; `wat::test_suite!`.
;;
;; Library files in the test directory don't commit startup config;
;; test_runner treats any .wat with no top-level `(:wat::config::set-*!)`
;; as a library and skips freezing it standalone. The entry test file
;; (test_loader.wat) commits config + `(:wat::core::load-file!)`s this file.

(:wat::core::define (:user::with_loader::test_helpers::magic -> :i64)
  42)
