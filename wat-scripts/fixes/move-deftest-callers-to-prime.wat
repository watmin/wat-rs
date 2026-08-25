;; wat-scripts/fixes/move-deftest-callers-to-prime.wat — IPC de-prime batch (core substrate tooling).
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; The harness's "move callers to the prime" step: rewrite the deftest MACRO CALL head to its
;; already-existing prime, BEFORE the non-prime `deftest` macro is deleted and 0z reclaims
;; `deftest'`->`deftest`. deftest/deftest' are call-compatible ([name prelude body]); the only
;; difference is the run-thread (raw-channel) vs run-thread' (peer) driver, behaviour-equivalent
;; for the pass/fail TestResult contract — so this is a pure head rename, no body reshape.
;;
;;   :wat::test::deftest  ->  :wat::test::deftest'
;;
;; IDEMPOTENT BY CONSTRUCTION via `rename-keyword-exact` (whole-token rename): it fires only when
;; the FULL keyword equals `:wat::test::deftest`, so after the rewrite the token is
;; `:wat::test::deftest'` (!= the old name) and a re-run matches nothing. Re-run == 0 changes.
;; (rename-keyword-*prefix* would treat `'` as a valid right-boundary and produce `deftest''` — the
;; exact non-idempotency this exact-match sibling exists to avoid.)
;;
;; WHOLE-TOKEN-SAFE: exact-equality never matches the prefix-sibling `:wat::test::deftest-hermetic`
;; (a different token). deftest-hermetic is migrated separately (its run-hermetic' driver has
;; different prelude-in-child semantics — a per-file, not a blind, move).
;;
;; DO NOT run on wat/test.wat — it DEFINES both `deftest` and `deftest'` macros; renaming the
;; non-prime defmacro head there would collide with the prime definition. Pass only CALLER files.
;;
;; Usage (one EDN vector of caller paths on stdin):
;;   printf '["tests/..." ...]\n' | ./target/release/wat ./wat-scripts/fixes/move-deftest-callers-to-prime.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::test::deftest" ":wat::test::deftest'" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[deftest -> deftest'] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
