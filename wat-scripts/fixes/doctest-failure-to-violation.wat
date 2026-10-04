;; wat-scripts/fixes/doctest-failure-to-violation.wat — excursus 003 strike F (AUDIT-the-
;; shape-of-an-error.md F8, RULING 2026-09-27 item 5).
;; SCOPE: wat/doctest.wat tests/reflection/probe_arc255_ivb2b_verify_examples.wat wat-scripts/scratch-pad/255-home4-verify-examples-failures.wat
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; `:wat::doctest::Failure` is a domain OUTCOME, not an error: `verify-examples` folds
;; over every `run=true` doctest and returns `(Vector :- [Failure])` — empty means every
;; doctest passed. It never travels in a `Result`'s `Err`, is never a `Failure.error`, and
;; carries no `location` — the shared name with the floor's `Fault`/`Failure` claims a
;; conformance this record does not have (AUDIT F8). The cure here is the OTHER branch of
;; F8's verdict: RENAME, not conform — `:wat::doctest::Violation`, matching the existing
;; `:wat::deporder::Violation` precedent (`wat/deporder.wat`) for exactly this shape: a
;; per-item record a verifier returns in a Vector, empty meaning clean, never consumed as
;; an error.
;;
;; A plain PREFIX rename: `:wat::doctest::Failure` (the defrecord's own name, every
;; construction call's head, every `/field` accessor) -> `:wat::doctest::Violation`,
;; riding `:wat::fix::rename-keyword-prefix` (boundary-aware whole-token leaf rewrite). No
;; other keyword in the corpus shares this prefix.
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat/doctest.wat" "tests/reflection/probe_arc255_ivb2b_verify_examples.wat" \
;;     "wat-scripts/scratch-pad/255-home4-verify-examples-failures.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/doctest-failure-to-violation.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::doctest::Failure" ":wat::doctest::Violation" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[doctest-failure-to-violation] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
