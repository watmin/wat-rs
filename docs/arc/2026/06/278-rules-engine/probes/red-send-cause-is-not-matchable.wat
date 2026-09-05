;; red-send-cause-is-not-matchable.wat — the RED probe for DESIGN-STONE-send-carries-its-cause.md (#70).
;;
;; ⛔ RED BY DESIGN, TODAY. Lives under docs/…/probes/ (NOT wat-scripts/) because
;;    `every_wat_scripts_file_loads` walks `wat-scripts` only — a deliberately-failing probe parked
;;    there would break that gate. Same reason as red-owner-signals-child.wat beside it.
;;
;; WHAT IT PROVES, and nothing else: `SendOutcome::Lost` carries a `:wat::kernel::Failure` — a FLAT
;; message record — so a caller CANNOT MATCH the cause. Its recv twin carries a
;; `:wat::kernel::LociDiedError` and can. Everything around the gap works: the spawn tooling, the
;; peer, `send` itself, the outcome match, and the recv-side control below all type-check. Exactly
;; one arm fails, and it fails on the carrier's type.
;;
;; ★ THE CONTROL IS LOAD-BEARING (:probe::recv-side-already-works). It matches the SAME
;;   LociDiedError variants against the RECV outcome and MUST type-check. Without it, a failure in
;;   the send arm is indistinguishable from "this probe doesn't know how to spell LociDiedError" —
;;   the probe would prove nothing. If the control ever goes red, the probe is broken, not the
;;   subject. (feedback_a_grep_that_cannot_reach_is_not_evidence, at the probe layer.)
;;
;; ARBITER: this one IS `--check`. Unlike red-owner-signals-child.wat — whose gap was an UNKNOWN
;; CALLEE and therefore deferred to a runtime UnknownFunction — this gap is a TYPE MISMATCH on a
;; known verb's known field, which is a check-phase failure. Pick the arbiter by the gap's phase.
;; Positive-control it anyway: the recv control's silence is what says --check is actually looking.
;;
;; TURNS GREEN AT #70. When `SendOutcome::Lost` / `TrySendOutcome::Lost` carry `LociDiedError`
;; instead of `Failure`, both arms below match and this file type-checks clean. At that point it
;; should MOVE to wat-scripts/scratch-pad/ (a green, loadable, durable reference) — a probe that has
;; turned green does not belong in the RED probes drawer.
;;
;; ⚠ NOTE WHAT THIS PROBE DELIBERATELY DOES NOT DO. It does not bind the cause as `_c`. Every one of
;;    the six live `SendOutcome::Lost` consumers in the stdlib does exactly that
;;    (wat/test.wat:392, wat/bracket.wat:47/85/139/411/537) — which is WHY nobody ever noticed the
;;    cause was a fabricated constant: the producer collapses it and every reader discards it. This
;;    probe is the first thing in the substrate that tries to READ a send failure's reason.

(:wat::core::defn :probe::send-side-cannot-match [] -> :wat::core::nil
  (:wat::core::let
    [peer (:wat::test::spawn-peer
            (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::kernel::println "child up"))))]
    (:wat::core::match (:wat::kernel::send peer "ping")
      (:wat::kernel::SendOutcome::Sent nil)
      (:wat::kernel::SendOutcome::Closed nil)
      ;; arc 278 #73 — orthogonal to this probe's gap (its subject is the Lost arm's
      ;; carrier type below, not this enum's exhaustiveness). Added only so the corpus
      ;; sweep doesn't overload this file with a SECOND, unrelated red — the deliberate
      ;; failure stays exactly where it was, in the nested cause match beneath ::Lost.
      (:wat::kernel::SendOutcome::Stopped nil)
      ;; ⛔ THE GAP. `cause` is a :wat::kernel::Failure today, so matching it against
      ;;    LociDiedError's variants is a type error. AFTER #70 it is a LociDiedError and
      ;;    `Stopped` vs `Disconnected` — the two states the send path currently conflates into
      ;;    one literal string — become distinguishable at the wat surface.
      ((:wat::kernel::SendOutcome::Lost cause)
        (:wat::core::match cause
          (:wat::kernel::LociDiedError::Stopped
            (:wat::kernel::println "the process is stopping"))
          (:wat::kernel::LociDiedError::Disconnected
            (:wat::kernel::println "the peer is gone"))
          (_ (:wat::kernel::println "some other death")))))))

;; ── POSITIVE CONTROL — the recv side already carries a matchable cause. MUST type-check. ──────
(:wat::core::defn :probe::recv-side-already-works [] -> :wat::core::nil
  (:wat::core::let
    [peer (:wat::test::spawn-peer
            (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::kernel::println "child up"))))]
    (:wat::core::match (:wat::kernel::recv peer)
      ((:wat::kernel::RecvOutcome::Message _m) nil)
      (:wat::kernel::RecvOutcome::Closed nil)
      ;; arc 278 #73 — same orthogonal note as the send-side match above: this control
      ;; MUST stay green (see the header), so it needs this arm to keep type-checking
      ;; now that RecvOutcome gained Stopped; it is not part of what the control proves.
      (:wat::kernel::RecvOutcome::Stopped nil)
      ;; The SAME variants, against the SAME enum, on the outcome that was migrated. Green today.
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          (:wat::kernel::LociDiedError::Stopped
            (:wat::kernel::println "the process is stopping"))
          (:wat::kernel::LociDiedError::Disconnected
            (:wat::kernel::println "the peer is gone"))
          (_ (:wat::kernel::println "some other death")))) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "red-send-cause probe: if you see this, the send arm compiled"))
