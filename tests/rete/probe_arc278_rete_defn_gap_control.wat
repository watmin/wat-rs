;; Arc 278 #88 — THE NON-VACUITY CONTROL for probe_arc278_rete_defn_gap.wat.bad.
;;
;; Byte-identical to that fixture with form #2 — the `(:wat::rete::core::defn …)` head —
;; DELETED, and nothing else changed. This file MUST load.
;;
;; Why it exists: a negative fixture on its own proves only "something in here is bad".
;; Paired with this control it proves "exactly the rete-defn form is what fails", because
;; the two differ by that form alone. A gap claim without its control is the vacuous-gate
;; class (R59 `NISI FRANGAS, NIHIL PROBAS`) wearing a probe's clothes.
;;
;; If this file ever stops loading, the sibling's RED stops meaning what its name says —
;; fix THIS first, then re-read the gap.

(:wat::core::defn :probe::ordinary [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::i64::> n 100))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::ordinary 42)))
