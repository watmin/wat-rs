;; tests/diagnostics/probe_excursus003_g3_one_shape_per_variant.wat — excursus 003 step 3b,
;; gate G3: one drivable producer per `LociDiedError` failure variant, each proving its
;; `Failure.error` is a REAL RECORD, never a string.
;;
;; Two of the four LociDiedError-channel producers below are spawned peers, defined in THIS
;; file; StartupError and MainSignature are each a direct `wat` binary invocation instead (see
;; the .rs file's header for why — measured, not assumed: both landed as a runtime Panic through
;; `:wat::test::spawn-peer`'s process locus instead of their own variant).
;;   Panic         — a spawned child's own `assertion-failed!` (an AssertionPayload panic).
;;   RuntimeError  — a spawned child's integer division by zero (mirrors gate G2's producer).
;;   StartupError  — driven from the .rs side; see there.
;;   MainSignature — driven from the .rs side; see there.
;;
;; The other two — EntryFormFailure, BadReturn — have NO drivable producer measured anywhere in
;; this tree (see the .rs file's header for the full account); they are named, not faked.

(:wat::core::defrecord :g3::Result
  [cause   <- :wat::kernel::LociDiedError
   message <- :wat::core::String])

;; Shared "recv the Lost cause, or say which wrong thing happened" match — one per producer fn,
;; not a shared helper: `:wat::kernel::LociDiedError` case arms differ by which variant EACH
;; producer expects, so each fn matches its OWN expected shape and sentinels everything else.

(:wat::core::defn :g3::panic-report [] -> :g3::Result
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::kernel::assertion-failed! :message "g3-panic-sentinel"))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg _m}
        (:g3::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:Message")]
      [:wat::kernel::RecvOutcome.Lost {:cause cause}
        (:wat::core::match cause
          [:wat::kernel::LociDiedError.Panic {:failure _f}
            (:g3::Result :cause cause :message (:wat::kernel::LociDiedError/message cause))]
          [_ (:g3::Result :cause cause :message "WRONG:not-Panic")])]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:g3::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:Stopped")]
      [:wat::kernel::RecvOutcome.Closed {}
        (:g3::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:Closed")])))

(:wat::core::defn :g3::runtime-error-report [] -> :g3::Result
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let [_ (:wat::i64::/ 1 0)] nil))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg _m}
        (:g3::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:Message")]
      [:wat::kernel::RecvOutcome.Lost {:cause cause}
        (:wat::core::match cause
          [:wat::kernel::LociDiedError.RuntimeError {:failure _f}
            (:g3::Result :cause cause :message (:wat::kernel::LociDiedError/message cause))]
          [_ (:g3::Result :cause cause :message "WRONG:not-RuntimeError")])]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:g3::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:Stopped")]
      [:wat::kernel::RecvOutcome.Closed {}
        (:g3::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:Closed")])))

;; StartupError, like MainSignature, is driven from the .rs side instead of a peer-death channel
;; here: measured (not assumed) that a spawned PEER's own freeze failure — even the exact
;; retired-colon-spelling trigger `probe_arc255_..._colon_spelling_is_refused.wat` uses — landed
;; as a runtime Panic through `:wat::test::spawn-peer`'s process locus, not `StartupError`. The
;; real, known-working producer is `probe_arc255_the_blanket_hides_a_phantom_head.rs`'s own:
;; the `wat` binary invoked DIRECTLY (exec, not this file's internal fork-based spawn) on a
;; program using the retired colon spelling.

;; MainSignature is driven from the .rs side instead of a peer-death channel here: a spawned
;; PEER's missing `:user::main` measured as a runtime `UnboundSymbol` (a Panic), not
;; `validate_user_main_signature`'s refusal — that wall runs at the TOP-LEVEL CLI entry
;; (`src/host/entry.rs` / `finish_in_process`), not inside `:wat::test::spawn-peer`'s process
;; locus. The real producer is `tests/cli/wat_cli.rs::missing_user_main_rejected`'s own
;; scenario: invoke the `wat` binary directly on a program with no `:user::main` at all.
