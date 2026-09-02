;; Arc 255 Stone 1a-α — the orchestrator's INDEPENDENT weigh of the rider's kill.
;;
;; Reads `signature-of-defn` for the five forms the stone's acceptance rows name:
;; three that MUST move to their declared `@syntax`, and two that MUST NOT move —
;; `if` (carries `@arg`, no `@syntax`) and `quasiquote` (not registered at all).
;;
;; The two negatives are the load-bearing rows: they are what distinguishes
;; "the new arm is placed correctly" from "the new arm swallowed everything".
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::let))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::fn))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::match))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::if))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::quasiquote))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::vec::length))))
