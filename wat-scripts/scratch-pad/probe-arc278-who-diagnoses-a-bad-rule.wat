;; probe-arc278-who-diagnoses-a-bad-rule.wat — WHO tells the user their rule is wrong?
;;
;; THE QUESTION. `fn-forms`'s walker raises on symbols inside QUOTED data (proven, rete-free, by
;; probe-arc278-fnforms-walks-into-quoted-data.wat). Removing that raise means the walker stops
;; policing the contents of a DSL's forms. The objection to answer BEFORE removing a guard is:
;; "then who catches a genuinely broken rule?"
;;
;; The builder's claim: it is a USER problem, already owned by the DSL — "our rete solution will
;; run compile and it will raise if compile faults and the user is given a detailed message on the
;; mistake." This file MEASURES that instead of assuming it.
;;
;; SHAPES, learned from the checker rather than guessed (it corrected two wrong assumptions in the
;; first draft of this file): `defrule` is a DECLARATION that expands to a ZERO-ARG DEFN returning
;; a Rule (wat/rete.wat:2385) — it does not evaluate to a Rule in place; and `:wat::rete::compile`
;; takes a `(PersistentVector :- [Rule])`, not a `(Vector :- [Rule])`.
;;
;; ⚠ NON-VACUITY. The CONTROL rule is well-formed in exactly the way the BROKEN one is malformed —
;; same records, same shape, one unbound variable apart. If both compile, the DSL diagnoses
;; nothing and the claim is REFUTED. If both fail, the instrument is not isolating the mistake.

(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])

;; ── CONTROL — well-formed: `?c` is bound by the `<-` in :when and consumed in :then.
(:wat::rete::defrule :usr::ok-rule
  :when [(:usr::Temp (?c <- :c) (:wat::rete::i64::> ?c 50))]
  :then [(:usr::Hot :c ?c)])

;; ── SUBJECT — a real user mistake: `?missing` is consumed in :then but NEVER bound in :when.
;; This is precisely the class the closure walker CANNOT diagnose (it does not know what a rule
;; is) and the rules layer CAN (it compiles them).
(:wat::rete::defrule :usr::bad-rule
  :when [(:usr::Temp (?c <- :c) (:wat::rete::i64::> ?c 50))]
  :then [(:usr::Hot :c ?missing)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ★ THE BLOCKER, CLOSED HONESTLY. The child-entry strike died because `fn-forms` raised
     ;; on `?c` while walking a rules body. `defrule` expands to a defn calling `make-rule`
     ;; with QUOTED :when/:then, so extracting a closure rooted at a rule fn is the exact
     ;; shape that failed. If this returns, the specific blocker that reverted the strike is
     ;; gone — measured, not inferred from "the floor is green" (the floor was green with the
     ;; strike reverted, so it could not have shown this either way).
     rule-forms (:wat::kernel::fn-forms :usr::ok-rule
                  (:wat::keyword::from-string "user::root-rule"))
     _r  (:wat::kernel::println
           (:wat::string::concat "fn-forms OVER A RULE FN: closure forms="
             (:wat::i64::to-string (:wat::core::length rule-forms))))
     ctl (:wat::core::PersistentVector (:usr::ok-rule))
     _a  (:wat::kernel::println "CONTROL rule built")
     cs  (:wat::rete::compile ctl)
     _b  (:wat::kernel::println "CONTROL compiled OK — the well-formed rule passes its own gate")
     bad (:wat::core::PersistentVector (:usr::bad-rule))
     _c  (:wat::kernel::println "BROKEN rule built — now compiling it")
     bs  (:wat::rete::compile bad)]
    (:wat::kernel::println
      "BROKEN COMPILED WITHOUT RAISING — the DSL did NOT diagnose the unbound variable; the claim is REFUTED")))
