;; wat-scripts/fixes/rename-wat-tests-std-to-wat-tests.wat — the last stranded `std`
;; segment in the test corpus, run over real wat source files IN WAT, through the wat
;; CLI. The migration tool, self-hosted: no Rust harness, no hand-edit of wat source
;; (use-the-tool, not hand-fix).
;;
;;   :wat-tests::std::test:: -> :wat-tests::test::
;;
;; WHY. Arc 109 killed the `wat/std/` directory and moved `wat/std/test.wat` ->
;; `wat/test.wat`. The harness followed (`:wat::test::`), but the CORPUS file that
;; tests it kept the stranded segment: `wat-tests/test.wat` still declares its fns
;; under `:wat-tests::std::test::` even though there is no `wat-tests/std/`
;; directory and never was after the move. Every other corpus namespace mirrors its
;; folder exactly — `:wat-tests::core::`, `:wat-tests::cache::`, `:wat-tests::edn::`,
;; `:wat-tests::bracket::`, … — so this is the single outlier in the whole corpus
;; (measured 2026-07-28: one `std` segment, 24 sibling namespaces without one).
;;
;; A FULL-NAME prefix, not a blanket `std::` rename: `:wat-tests::std::test` is the
;; complete stranded path, and using it as the prefix still catches every fn suffix
;; that shares it (`::test-assert-eq-on`, `::test-run-ast-via-program`, …). Nothing
;; else in the corpus carries `::std::`, so the rewrite cannot over-reach.
;;
;; NOT to be confused with `:wat::std::` — that namespace is ALIVE (`list::zip`,
;; `math::ln`, `stat::mean`, … ~17 verbs registered in src/collection/ and consumed by
;; wat/holon/*.wat). This codemod does not touch it: the prefix here begins
;; `:wat-tests::`, which is disjoint from `:wat::`.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat-tests/test.wat" "wat-tests/run-thread.wat" "wat-scripts/fixes/kill-make-deftest.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-wat-tests-std-to-wat-tests.wat
;;
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes — the
;; old prefix is gone), so it is safe to run over a clean tree.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat-tests::std::test" ":wat-tests::test"
    src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
