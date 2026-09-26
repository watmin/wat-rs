;; wat-scripts/fixes/loci-died-error-carries-failure.wat — excursus 003 step 3b.
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Every `:wat::kernel::LociDiedError.<Panic|RuntimeError|StartupError|EntryFormFailure|
;; MainSignature|BadReturn>` match-arm pattern in the corpus destructured the OLD shape
;; (`{:message m}` / `{:error m}`, Panic's `{:message m :failure f}` with `f` an
;; `Option<Failure>`) before the field reshape collapsed every one of the six to a single
;; mandatory `{:failure f}`. See `wat/fix.wat`'s "EXCURSUS 003 STEP 3B" section (the rule
;; logic lives there — `:wat::fix::loci-died-error-carries-failure` and its two shape
;; predicates/edit-builders) for the full account of the two shapes this folds and why a
;; nested map pattern was not the answer (measured: wat's `match` refuses one).
;;
;; Worklist (measured by hand, grepping `LociDiedError\.(variant) {` and reading every
;; hit — not a form-tree census, but every file was opened and its match arms read):
;;   tests/comms/probe_arc209_structured_peer_death.wat (shape 2)
;;   tests/comms/probe_arc278_failure_carries_structured_error.wat (shape 2)
;;   tests/comms/probe_arc278_loci_died_error_round_trip.wat (shape 1)
;;   tests/comms/wat_arc113_cross_fork_cascade.wat (shape 2)
;;   tests/comms/wat_arc113_raise_round_trip.wat (shape 2 + shape 1 x5)
;;   tests/diagnostics/probe_arc296_raise_gate.wat (shape 1)
;;   tests/diagnostics/probe_no_default_rust_panic_noise_on_stderr.wat (shape 1)
;;   tests/diagnostics/probe_plain_panic_produces_structured_edn.wat (shape 1 x6)
;;   tests/diagnostics/probe_runtime_error_produces_structured_edn.wat (shape 1 x6)
;;   tests/diagnostics/probe_runtime_err_stderr_visibility.wat (shape 1)
;;   tests/kernel/wat_run_sandboxed.wat (shape 1 x9)
;;   wat-tests/core/core-nth-differential.wat (shape 1 x3)
;;   wat-tests/core/core-nth.wat (shape 1 x3)
;;   wat-tests/core/core-seq-walkers.wat (shape 1 x2)
;;   wat-tests/test.wat (shape 1)
;;
;; BOOTSTRAP: this ships against the OLD checker (`wat/kernel/diagnostics.wat`'s OLD field
;; shape stashed back in) per `wat/fix.wat`'s own STASH-DANCE note — the codemod verb is
;; NEW, and the excursus 003 step 3b shape change makes the OLD corpus form illegal, so both
;; cannot be live in the same build at once. `git stash` the step 3b shape change (every
;; `wat/*.wat` + `src/*.rs` file that reshape touched), `cargo build --release` (old checker
;; + this new verb), run this codemod on the full worklist above, `git stash pop`, rebuild.
;;
;; Worked examples (quoted verbatim by the replay fixture's ORACLE —
;; `wat-scripts/fixes/replay/loci-died-error-carries-failure/`):
;;
;;   SHAPE 1, non-Panic:
;;     `[:wat::kernel::LociDiedError.RuntimeError {:message m} m]` becomes
;;     `[:wat::kernel::LociDiedError.RuntimeError {:failure m} (:wat::kernel::Failure/message m)]`
;;
;;   SHAPE 2, Panic's Option-drill-down:
;;     `[:wat::kernel::LociDiedError.Panic {:message pm :failure pf} (:wat::core::match pf [:wat::core::Option.Some {:value f} (:wat::kernel::Failure/message f)] [:wat::core::Option.None {} "none"])]` becomes
;;     `[:wat::kernel::LociDiedError.Panic {:failure f} (:wat::kernel::Failure/message f)]`
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/loci-died-error-carries-failure.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::loci-died-error-carries-failure src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[loci-died-error-carries-failure] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
