;; wat-scripts/fixes/move-deftest-hermetic-callers-to-prime.wat — IPC de-prime batch.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; The hermetic sibling of move-deftest-callers-to-prime.wat: rewrite the
;; deftest-hermetic MACRO CALL head to its already-existing prime, BEFORE the
;; non-prime `deftest-hermetic` macro is deleted (the ablaze) and 0z reclaims
;; `deftest-hermetic'`->`deftest-hermetic`.
;;
;;   :wat::test::deftest-hermetic  ->  :wat::test::deftest-hermetic'
;;
;; NOW A BLIND, WHOLE-CORPUS SAFE RENAME. The thread codemod's header warned
;; that deftest-hermetic was a "per-file, not a blind, move" because the old
;; non-prime shipped a PRELUDE into the forked child while the prime did not
;; (deftest-hermetic' was an "incomplete prime"). Arc 278 ANNIHILATED the
;; prelude slot: both are now body-only (`deftest-hermetic` -> run-hermetic,
;; `deftest-hermetic'` -> run-hermetic'), differing ONLY in the raw-channel vs
;; peer driver — behaviour-equivalent for the pass/fail TestResult contract.
;; So the incomplete-prime caveat is MOOT; this is a pure head rename, no reshape.
;;
;; IDEMPOTENT BY CONSTRUCTION via `rename-keyword-exact` (whole-token): fires only
;; when the FULL keyword equals `:wat::test::deftest-hermetic`, so after the rewrite
;; the token is `:wat::test::deftest-hermetic'` (!= the old name) and a re-run
;; matches nothing. WHOLE-TOKEN-SAFE: exact-equality never matches the plain
;; `:wat::test::deftest` (a shorter, different token) nor the already-primed
;; `:wat::test::deftest-hermetic'`.
;;
;; DO NOT run on wat/test.wat — it DEFINES both macros; renaming the non-prime
;; defmacro head there would collide with the prime definition. Pass only CALLER files.
;;
;; Usage (one EDN vector of caller paths on stdin):
;;   printf '["wat-tests/..." ...]\n' | ./target/release/wat ./wat-scripts/fixes/move-deftest-hermetic-callers-to-prime.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::test::deftest-hermetic" ":wat::test::deftest-hermetic'" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[deftest-hermetic -> deftest-hermetic'] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
