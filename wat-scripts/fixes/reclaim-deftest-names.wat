;; wat-scripts/fixes/reclaim-deftest-names.wat — IPC de-prime 0z (name reclaim).
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; The 0z reclaim for the deftest family: after the non-prime deftest /
;; deftest-hermetic macro DEFINITIONS were deleted (their callers already all
;; moved to the primes), the freed plain names are reclaimed by the primes —
;; drop the `'`:
;;
;;   :wat::test::deftest'            ->  :wat::test::deftest
;;   :wat::test::deftest-hermetic'   ->  :wat::test::deftest-hermetic
;;
;; SAFE ON wat/test.wat TOO (unlike the caller-move codemods): the non-prime
;; definitions are already deleted, so renaming the prime defmacro head no
;; longer collides with a non-prime def. Run on EVERY file that references the
;; primes, wat/test.wat included (it renames the prime defmacro heads there).
;;
;; WHOLE-TOKEN-SAFE + IDEMPOTENT via `rename-keyword-exact`: `:…::deftest'`
;; (whole token) never matches `:…::deftest-hermetic'` (a different token), and
;; after the rewrite the tokens are `'`-less (!= the old names) so a re-run is
;; a no-op. The two renames are independent (distinct tokens) — order-free.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/test.wat" "wat-tests/..." …]\n' | ./target/release/wat ./wat-scripts/fixes/reclaim-deftest-names.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::test::deftest-hermetic'" ":wat::test::deftest-hermetic"
    (:wat::fix::rename-keyword-exact ":wat::test::deftest'" ":wat::test::deftest" src)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[reclaim deftest] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
