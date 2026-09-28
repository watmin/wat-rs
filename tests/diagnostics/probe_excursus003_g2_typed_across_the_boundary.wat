;; tests/diagnostics/probe_excursus003_g2_typed_across_the_boundary.wat — excursus 003 step 3b,
;; gate G2: a runtime crash crosses the peer boundary TYPED, not as a masked string.
;;
;; A child divides by zero. The parent's `recv'` sees `RecvOutcome::Lost {cause}`; `cause` MUST be
;; a `:wat::kernel::LociDiedError.RuntimeError` whose `failure.error` is the structured
;; `:wat::runtime::DivisionByZero` record — its CLASS, not its rendered text — with a non-empty
;; `frames` vector, and whose derived `(:wat::kernel::LociDiedError/message cause)` equals the
;; error's own structural `.message` field exactly (the "one-line headline" the reshape derives,
;; not a re-serialization of anything).
;;
;; EDN-over-stdio (`docs/CONVENTIONS.md` § Test idioms): the crash is a real dying declaration —
;; only a real spawned process has an exit code, and only ITS wire carries the death report. The
;; `:g2::Result` bundles the raw `cause` enum (so the .rs side can inspect it structurally) with
;; the wat-computed headline, so the .rs test can assert the SAME text both ways without hand-
;; authoring the expected string twice.

(:wat::core::defrecord :g2::Result
  [cause   <- :wat::kernel::LociDiedError
   message <- :wat::core::String])

(:wat::core::defn :g2::divzero-report [] -> :g2::Result
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let [_ (:wat::i64::/ 1 0)] nil))))]
    (:wat::core::match (:wat::kernel::recv svc)
      ;; LociDiedError is the no-hidden-failures enum — every OTHER death is named EXPLICITLY (no
      ;; `_` lump; verbosity is the shield). A distinct WRONG:<variant> sentinel makes a RED name
      ;; exactly which non-RuntimeError shape surfaced instead — including a silent fallback to
      ;; Panic, the exact failure mode item 4 of the brief guards against.
      [:wat::kernel::RecvOutcome.Message {:msg _m}
        (:g2::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:Message")]
      [:wat::kernel::RecvOutcome.Lost {:cause cause}
        (:wat::core::match cause
          [:wat::kernel::LociDiedError.RuntimeError {:failure _f}
            (:g2::Result :cause cause :message (:wat::kernel::LociDiedError/message cause))]
          [:wat::kernel::LociDiedError.Panic {:failure _pf}
            (:g2::Result :cause cause :message "WRONG:Panic")]
          [:wat::kernel::LociDiedError.StartupError {:failure _sf}
            (:g2::Result :cause cause :message "WRONG:StartupError")]
          
          [:wat::kernel::LociDiedError.MainSignature {:failure _mf}
            (:g2::Result :cause cause :message "WRONG:MainSignature")]
          
          [:wat::kernel::LociDiedError.Disconnected {}
            (:g2::Result :cause cause :message "WRONG:Disconnected")]
          [:wat::kernel::LociDiedError.Stopped {}
            (:g2::Result :cause cause :message "WRONG:Stopped")])]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:g2::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:RecvStopped")]
      [:wat::kernel::RecvOutcome.Closed {}
        (:g2::Result :cause (:wat::kernel::LociDiedError.Disconnected {}) :message "WRONG:RecvClosed")])))
