;; Fixture for probe_arc170_repl_freeze_partition.rs — one REPL session's worth of lines,
;; one per DECLARATION KIND plus one expression. The probe freezes these as a form-set and
;; asks what the freeze left behind in `FrozenWorld.program`.
;;
;; The kinds are chosen to span the three DIFFERENT refusal mechanisms that
;; probe-repl-declaration-refusal.wat measured at eval time:
;;   def       → refused at dispatch     (DeclarationInExpressionPosition)
;;   defenum   → refused by the mutation gate (mutation-form-refused)
;;   defn      → never reaches eval at all    (unknown-function — same as a TYPO)
;;   defrecord → never reaches eval at all    (unknown-function — same as a TYPO)
;; If the freeze partitions all four the same way, it is the single authority the
;; eval-time errors are not.

(:wat::core::defn :usr::f [] -> :wat::core::i64 7)
(:wat::core::defrecord :usr::R [a <- :wat::core::i64])
(:wat::core::defenum :usr::E :wat::enum::Pure :A [])
(:wat::core::def :usr::x 1)
(:wat::core::+ 1 2)
