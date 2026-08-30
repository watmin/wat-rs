;; probe-brief-first-nth-to-string.wat — RUN proof for
;; docs/arc/2026/06/278-rules-engine/BRIEF-one-naming-rule-then-first-nth-to-string.md, PHASE 2.
;;
;; `nth`'s own Fallback row is NOT minted (see the BRIEF report / vocabulary.rs's absence of a
;; `:wat::rete::core::nth` row): `nth` is a wat-level `defn` (`wat/core.wat:1349`) whose
;; out-of-range failure is `Option/expect`'s `std::panic::panic_any` — a genuine Rust panic, not a
;; `Result::Err` — which `dispatch_rete_op`'s `Fallback` arm cannot intercept (it only pattern-
;; matches on `Err(EvalBreak::Diagnostic(_))`). This file therefore covers `first` (all three
;; containers) and `to-string` (all three scalars) only.
;;
;; A vacuous probe (no `:user::main`) proves nothing — this file has a real `:user::main`,
;; printing one line per assertion so the transcript is the proof.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── to-string, all three scalars ──────────────────────────────────────────────
    (:wat::kernel::println (:wat::rete::i64::to-string 42))        ;; expect "42"
    (:wat::kernel::println (:wat::rete::f64::to-string 1.5))       ;; expect "1.5"
    (:wat::kernel::println (:wat::rete::core::bool::to-string true))     ;; expect "true"

    ;; ── first, happy path, all three containers (fallback NOT taken) ────────────────
    (:wat::kernel::println
      (:wat::rete::core::PersistentVector/first
        (:wat::core::PersistentVector 7 8 9) :undefined -1))             ;; expect 7
    (:wat::kernel::println
      (:wat::rete::core::Vector/first
        (:wat::core::Vector :- [:wat::core::i64] 7 8 9) :undefined -1))       ;; expect 7
    (:wat::kernel::println
      (:wat::rete::core::List/first
        (:wat::core::List 7 8 9) :undefined -1))                      ;; expect 7

    ;; ── first, fallback FIRES on empty, all three containers ────────────────────────
    (:wat::kernel::println
      (:wat::rete::core::PersistentVector/first
        (:wat::core::PersistentVector) :undefined -1))                  ;; expect -1
    (:wat::kernel::println
      (:wat::rete::core::Vector/first
        (:wat::core::Vector :- [:wat::core::i64]) :undefined -1))            ;; expect -1
    (:wat::kernel::println
      (:wat::rete::core::List/first
        (:wat::core::List) :undefined -1))                           ;; expect -1

    ;; ── NON-VACUITY — the row that matters most: the SAME empty-container expression,
    ;; run twice, with DIFFERENT `:undefined` fallback values. A `first` fallback arm that
    ;; merely returned a constant would pass every row above; only this pair proves the
    ;; caller's actual fallback VALUE is what comes back, not a hardcoded stand-in.
    (:wat::kernel::println
      (:wat::rete::core::PersistentVector/first
        (:wat::core::PersistentVector) :undefined 0))                   ;; expect 0
    (:wat::kernel::println
      (:wat::rete::core::PersistentVector/first
        (:wat::core::PersistentVector) :undefined 99))                  ;; expect 99
    nil))
