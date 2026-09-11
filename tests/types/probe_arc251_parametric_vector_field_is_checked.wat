;; ★★★ RATCHET FIRED. This file's PREMISE was that `--check` EXITS 0 on it. It now exits 1.
;;
;; It was `wat-scripts/scratch-pad/probe-parametric-vector-field-is-unchecked.wat`, committed at
;; `02dc901a2` as FINDING(check): a PARAMETRIC vector field of a variant map-ctor is NEVER
;; INFERRED. `(:probe::Box :NOPE 1)` names a field that does not exist, inside a VECTOR-literal
;; value for a variant field whose element type is PARAMETRIC — and nothing inferred it. The
;; discriminating rows, measured then:
;;
;;   :state <- S                            a bogus value IS caught            exit 1
;;   :reply <- R                            a bogus value IS caught            exit 1
;;   items  <- (Vector :- [i64])            NON-parametric element, CAUGHT     exit 1
;;   items  <- (Vector :- [(Box :- [T])])   PARAMETRIC element, bogus field    exit 0  ⛔
;;
;; ⛔ WHY IT MATTERED BEYOND ITS OWN SHAPE: `:wat::service::Alarm` lives in exactly this position
;; (`Outcome.ReplyAndArm`'s `arms <- (Vector :- [(Alarm :- [O])])`), so `PublicOpInAlarm` could
;; never fire on a map-form construction. It had been firing ONLY via the positional variant form,
;; which takes a different inference path — and positional construction is independently retired.
;; The wall's one live caller was a retired spelling: the wall was silently DEAD.
;;
;; ★ CURED 2026-09-10 by two fixes, neither of which was aimed here. `expand_form` never walked
;; MAP or SET, so a macro call inside a map literal was never expanded — the unexpanded head
;; carried no scheme, `infer` returned a fresh var, and `assignable` passed trivially. And
;; `{:keys}`/field instantiation stopped dropping a Parametric's type arguments. `:NOPE` is now
;; refused, named, and located.
;;
;; ⛔ IT CANNOT LIVE UNDER `wat-scripts/` ANY MORE, and that is not incidental. That tree's gate
;; (`tests/lint/wat_scripts_fixes_load.rs`) requires every `.wat` beneath it to load CLEANLY; a
;; fixture whose whole job is to BE REFUSED reads there as rot. Same move, same reason, as arc 255
;; Stone 4, which relocated the blanket's two dependents into `tests/` when their containment
;; premise died. The refusal is asserted by the probe beside this file.
(:wat::core::defrecord :probe::Box :- [T] [v <- :T])
(:wat::core::defenum :probe::Holder :- [T] :wat::enum::Pure
  :Many [items <- (:wat::core::Vector :- [(:probe::Box :- [:T])])]
  :One  [item <- :T])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::Holder.Many {:items [(:probe::Box :NOPE 1)]})))
