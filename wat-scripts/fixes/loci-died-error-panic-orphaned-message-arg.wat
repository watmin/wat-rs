;; wat-scripts/fixes/loci-died-error-panic-orphaned-message-arg.wat — excursus 003 step 3b,
;; REPAIR pass over the `loci-died-error-carries-failure` codemod's own output.
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; `ldef-shape1-edits` (the codemod this repairs) reuses the pre-existing `:failure` binder's
;; OWN name as the new pattern binding for a Panic arm, but its body-rewrite step
;; (`ldef-wrap-symbol-node-edits`) wraps occurrences of the DROPPED `:message` binder IN
;; PLACE rather than substituting the new binder's name. For the five non-Panic variants the
;; two names are the same binder (repurposed), so this coincidentally produced correct text.
;; For Panic, when the pre-existing `:failure` binder was a DIFFERENT, underscore-prefixed
;; name (`_failure` — genuinely unused under the OLD two-field shape, where `failure` was an
;; ignorable `Option`), the codemod emitted `(:wat::kernel::Failure/message message)` with
;; `message` no longer bound anywhere in the arm. See `wat/fix.wat`'s "EXCURSUS 003 STEP 3B
;; REPAIR" section (`:wat::fix::loci-died-error-repair-orphaned-message-arg` and its helpers)
;; for the full account.
;;
;; Worklist (measured: `git ls-files '*.wat' | xargs grep -n "Failure/message message"`, plus
;; `probe_arc278_loci_died_error_round_trip.wat`'s non-underscored variant of the same
;; orphaned-arg defect — grep by the SYMPTOM, not the exact string, since that file's binder
;; was already named `failure`, just never substituted into the call):
;;   tests/comms/probe_arc278_loci_died_error_round_trip.wat (1)
;;   tests/diagnostics/probe_arc296_raise_gate.wat (1)
;;   tests/diagnostics/probe_no_default_rust_panic_noise_on_stderr.wat (1)
;;   tests/diagnostics/probe_plain_panic_produces_structured_edn.wat (1)
;;   tests/diagnostics/probe_runtime_err_stderr_visibility.wat (1)
;;   tests/kernel/wat_run_sandboxed.wat (5)
;;   wat-tests/core/core-nth-differential.wat (3)
;;   wat-tests/core/core-nth.wat (3)
;;   wat-tests/core/core-seq-walkers.wat (2)
;;   wat-tests/test.wat (1)
;;
;; (`probe_runtime_error_produces_structured_edn.wat`'s `{:failure message} (Failure/message
;; message)]` binds the FAILURE itself to a variable named `message` — a confusing name, but
;; `message` IS the failure there, so the call is correct and this codemod leaves it alone.)
;;
;; Worked examples (quoted verbatim by the replay fixture's ORACLE —
;; `wat-scripts/fixes/replay/loci-died-error-panic-orphaned-message-arg/`):
;;
;;   UNDERSCORE-PREFIXED BINDER (rename needed):
;;     `[:wat::kernel::LociDiedError.Panic {:failure _failure} (:wat::kernel::Failure/message message)]` becomes
;;     `[:wat::kernel::LociDiedError.Panic {:failure failure} (:wat::kernel::Failure/message failure)]`
;;
;;   ALREADY-NAMED BINDER, orphaned arg only (no rename — `failure` already truthfully used):
;;     `[:wat::kernel::LociDiedError.RuntimeError {:failure failure} (:wat::core::Option.Some {:value (:wat::kernel::Failure/message message)})]` becomes
;;     `[:wat::kernel::LociDiedError.RuntimeError {:failure failure} (:wat::core::Option.Some {:value (:wat::kernel::Failure/message failure)})]`
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/loci-died-error-panic-orphaned-message-arg.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::loci-died-error-repair-orphaned-message-arg src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[loci-died-error-panic-orphaned-message-arg] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
