;; Scratch probe — arc 255. TWO QUESTIONS ABOUT `sort$native` (measured while it was still named `sort'`), BOTH ANSWERED BY RUNNING IT.
;;
;; Builder, 2026-08-30: "can we impose the sort fn is pure, deterministic, total?"
;;                      "users can't make sort effectful with their callable?"
;;
;; ── MEASURED 1: YES, a user CAN make sort effectful today. Nothing stops them. ──
;; `:user::effectful` below passes a comparator that println's. Result:
;;     "SIDE-EFFECT-FROM-COMPARATOR"   x4      then  [1 2 3]      exit 0
;; FOUR effects for a THREE-element vector — the count is an artifact of the
;; two-sided less? call (transform.rs:294), i.e. an implementation detail leaking
;; into observable user output. `--check` exits 0; no purity gate exists at this door.
;; ⇒ `sort$native`'s HONEST @Purity today is Effectful, which is why homing it hits the
;;   W7 `effectful_by_prefix` blocker (NOTE-the-prefix-guess-does-not-scale...).
;;
;; ── MEASURED 2: a PURE but INCONSISTENT comparator does NOT panic. ──
;; `:user::inconsistent` returns `true` always — not a strict weak ordering.
;; Result: [0 6 4 8 2 7 1 9 3 5]  exit 0. Scrambled, well-formed, no panic.
;; ⇒ @Totality is `Total` on its own merits: no reachable failure path, even
;;   pathologically. Imposing purity is NOT what buys totality — totality already
;;   holds. And imposing purity does NOT buy a correct ORDER; nobody should claim it.
;;
;; ★ THE RULING THIS SUPPORTS: impose pure ∧ deterministic ∧ total on the
;;   comparator (precedent + machinery: `src/freeze.rs:803` does exactly this for
;;   sigma fns via `find_axis_violation` in `src/rete/purity.rs`). Then `sort$native`
;;   is Pure ∧ Deterministic ∧ Total and homes with NO prefix widening — the W7
;;   blocker dissolves because its premise ("runs code it did not write") is made false.
;;
;; ⚠ AFTER THE RENAME this file must say `sort$native`; the `every_wat_scripts_file_loads`
;;   gate will force it. That is the ratchet working, not a chore.
;; Not a permanent fixture — delete when the ruling ships.

(:wat::core::defn :user::effectful [] -> :wat::core::nil
  (:wat::core::let
    [cmp (:wat::core::fn [a <- :wat::core::i64
                          b <- :wat::core::i64] -> :wat::core::bool
           (:wat::core::do
             (:wat::kernel::println "SIDE-EFFECT-FROM-COMPARATOR")
             (:wat::core::< a b)))]
    (:wat::kernel::println
      (:wat::core::sort$native cmp (:wat::core::Vector :- [:wat::core::i64] 3 1 2)))))

(:wat::core::defn :user::inconsistent [] -> :wat::core::nil
  (:wat::core::let
    [always-less (:wat::core::fn [a <- :wat::core::i64
                                  b <- :wat::core::i64] -> :wat::core::bool
                   true)]
    (:wat::kernel::println
      (:wat::core::sort$native always-less
        (:wat::core::Vector :- [:wat::core::i64] 5 3 9 1 7 2 8 4 6 0)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::effectful)
    (:user::inconsistent)))
