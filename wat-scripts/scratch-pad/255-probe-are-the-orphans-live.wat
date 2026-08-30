;; Arc 255 — do the intrinsic_meta hand-list names with NO literal dispatch arm still RESOLVE?
;; Recorded for WORKLIST-the-44-unhomed.md. Determines homing-work vs dead-entry.
;; MEASURED 2026-08-30: `+` `-` `*` `/` resolve (live, dispatched off the literal-arm path);
;; `when` does NOT — `unknown function: :wat::core::when`, a hand-list row for a verb that
;; does not exist. `reduce` measured separately below.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::+ 1 2))
    (:wat::kernel::println (:wat::core::- 9 4))
    (:wat::kernel::println (:wat::core::* 3 3))
    (:wat::kernel::println (:wat::core::/ 8 2))))
