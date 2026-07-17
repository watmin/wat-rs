;; Co-located fixture for probe_arc278_eprintln_terminal.rs.
;;
;; Arc 278 no-hidden-failures — SUB-STRIKE "eprintln is terminal" (closes
;; feedback_eprintln_is_terminal). eprintln (and its pretty twin epprintln) is
;; a DYING DECLARATION: it emits the value's EDN to stderr, then TERMINATES
;; non-zero. Any form that follows it MUST NOT run. Each compute fn runs a body
;; under run-hermetic (forked child) and returns the RunResult the Rust driver
;; inspects: a terminal eprintln makes the child crash (failure = Some), the
;; value lands on stderr, and the `println "AFTER"` never reaches stdout.

;; eprintln terminates BEFORE the following println — stdout must stay empty.
(:wat::core::defn :probe::compute-eprintln-terminates [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::core::do
      (:wat::kernel::eprintln "dying words")
      (:wat::kernel::println "AFTER"))))

;; epprintln (pretty twin) is likewise terminal — same shape, pretty EDN writer.
(:wat::core::defn :probe::compute-epprintln-terminates [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::core::do
      (:wat::kernel::epprintln "pretty dying words")
      (:wat::kernel::println "AFTER"))))
