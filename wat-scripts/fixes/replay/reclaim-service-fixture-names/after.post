;; Co-located fixture for probe_arc278_eprintln_terminal.rs.
;;
;; Arc 278 no-hidden-failures — SUB-STRIKE "eprintln is terminal" (closes
;; feedback_eprintln_is_terminal). eprintln (and its pretty twin epprintln) is
;; a DYING DECLARATION: it emits the value's EDN to stderr, then TERMINATES
;; non-zero. Any form that follows it MUST NOT run.
;;
;; IPC de-prime (arc 278): migrated off the non-prime `run-hermetic` (forks +
;; scrapes OS pipes → RunResult.stdout/stderr) onto the primed `run-hermetic'`
;; (peer wire: the child signals a pass-marker over recv'; a crash → Lost[cause]
;; → RunResult.failure = Some[cause]; RunResult.stdout/stderr are EMPTY — the
;; wire model captures NO stderr). What of "eprintln is terminal" survives the
;; wire model, and how:
;;   - TERMINATION is observable: run-hermetic' appends a `(println 0)` pass-marker
;;     AFTER the body. A terminal eprintln crashes the child BEFORE that marker →
;;     recv' returns Lost[cause] → RunResult.failure = Some. (Were eprintln benign,
;;     the following forms — incl. the pass-marker — would run → Message → failure
;;     = None.) So failure=Some IS the "following forms never ran" signal.
;;   - The emitted VALUE also survives: eprintln_terminate panics with the value's
;;     EDN as the crash reason (src/services/verbs.rs `eprintln_terminate`), which
;;     rides the Lost cause's `:wat::kernel::Failure/message` — NOT the OS-stderr
;;     capture the wire model drops. The Rust driver reads it out of the failure.
;; Each compute fn runs a body under run-hermetic' and returns the RunResult.

;; eprintln terminates BEFORE the following println AND the pass-marker — the
;; child crashes → RunResult.failure = Some, carrying the value's EDN.
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
