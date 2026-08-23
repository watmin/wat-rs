;; tests/comms/probe_arc278_loci_died_error_round_trip.wat — the RED gate for the LociDiedError stone
;; (DESIGN-loci-died-error.md). Co-located fixture, slurped via startup_beside(file!()); defns only.
;;
;; THE GATE (disconfirming probe — RED today, GREEN when the stone lands):
;;   A peer dies abnormally (raises a Fault). `recv'` hands the owner `Lost[cause]`.
;;   The ratified UX is that `cause` is a LOCI-AGNOSTIC `:wat::kernel::LociDiedError` —
;;   a matchable VALUE whose `Panic` variant carries the structured death reason — NOT the
;;   flat `:wat::kernel::Failure` it is today. This probe matches `cause` as a LociDiedError
;;   and reads the panic message.
;;
;;   Today this is RED on EXACTLY the gap: `:wat::kernel::LociDiedError` is unregistered and
;;   `RecvOutcome::Lost`'s cause is `Failure` (no `Panic` variant) — a check-time error.
;;   Post-stone it is GREEN: `cause` is a `LociDiedError`, `Panic` matches, the message is read.
;;   Structural round-trip: the death report is a registered record, EDN all the way down.

(:wat::core::defn :my::died-cause-panic-message [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [p
      (:wat::test::spawn-peer (:wat::spawn::process)
        (:wat::core::forms
          (:wat::core::defn :user::main [] -> :wat::core::nil
            (:wat::kernel::raise! (:wat::core::Fault/of "loci-died-panic-data")))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) :wat::core::None)
      ((:wat::kernel::RecvOutcome::Lost cause)
        ;; THE GATE: `cause` is a loci-agnostic LociDiedError — a matchable death report,
        ;; not a flat Failure. Every peer handles every death regardless of its locus.
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic message failure)
           (:wat::core::Some message))
          (_ :wat::core::None)))
      (:wat::kernel::RecvOutcome::Stopped :wat::core::None)
      (:wat::kernel::RecvOutcome::Closed :wat::core::None))))
