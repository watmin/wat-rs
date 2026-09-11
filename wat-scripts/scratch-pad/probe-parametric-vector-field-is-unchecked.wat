(:wat::core::defrecord :probe::Box :- [T] [v <- :T])
(:wat::core::defenum :probe::Holder :- [T] :wat::enum::Pure
  :Many [items <- (:wat::core::Vector :- [(:probe::Box :- [:T])])]
  :One  [item <- :T])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::Holder.Many {:items [(:probe::Box :NOPE 1)]})))

;; ⛔ MEASURED 2026-09-10 — `--check` EXITS 0 on this file. `(:probe::Box :NOPE 1)` names a field
;; that does not exist, inside a VECTOR-literal value for a variant field whose element type is
;; PARAMETRIC. Nothing infers it.
;;
;; The discriminating rows, all measured on the same build:
;;   :state  <- S                                    a bogus value IS caught      exit 1
;;   :reply  <- R                                    a bogus value IS caught      exit 1
;;   items   <- (Vector :- [i64])       NON-parametric element, bogus value CAUGHT exit 1
;;   items   <- (Vector :- [(Box :- [T])])  PARAMETRIC element, bogus field       exit 0  ⛔
;;
;; WHY IT MATTERS BEYOND ITS OWN SHAPE: `:wat::service::Alarm` lives in exactly this position —
;; `Outcome.ReplyAndArm`'s `arms <- (Vector :- [(Alarm :- [O])])` — so `alarm_op_internal_check`
;; (CheckErrorKind::PublicOpInAlarm, src/check.rs) can never fire on a map-form construction. It
;; fired before only because its one fixture used the POSITIONAL variant form, which takes a
;; different inference path; positional variant construction is independently retired, so the
;; wall's only live caller was the retired spelling.
