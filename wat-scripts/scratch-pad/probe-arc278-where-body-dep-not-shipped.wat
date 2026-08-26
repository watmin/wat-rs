;; PROBE — does closure extraction COLLECT a dependency referenced only inside a
;; `(:wat::rete::where …)` body?
;;
;; closure_extract.rs refuses `Boundary::MakeRule`, and its own comment names the
;; cost: "deps inside a (:wat::rete::where …) body go uncollected, and the child
;; names them at startup." That is a documented FALSE NEGATIVE, never measured.
;; If real, it is the mechanism behind "we fail to deliver rules to install-rules":
;; the rule ships, the fn its `where` calls does not.
;;
;; ★ MEASURED 2026-08-12: POSITIVE-CONTROL 6 · BASELINE 5 · SUBJECT 5. The false
;; negative is REAL — the rule ships, the fn its `where` calls does not.
;;
;; ★★ THIS FILE DOCUMENTS A LIVE DEFECT AND ITS VERDICT WILL FLIP. When
;; DESIGN-STONE-defrule-splits-at-expansion-time lands, `defrule` lifts each `where`
;; body into a named top-level defn and emits one code-position mention, so SUBJECT
;; must RISE to meet POSITIVE-CONTROL. At that moment this file's closing line stops
;; being true and MUST be rewritten to say what it then proves. A verdict written for
;; a RED state and kept after it goes green reads as a failure to the next person and
;; as a pass to the tooling — that exact slip is on the record for this arc.
;;
;; ⚠ NON-VACUITY, three arms sharing ONE helper `:usr::big?`:
;;   POSITIVE CONTROL — helper called in ORDINARY code position. The collector MUST
;;     find it; if this count does not exceed the bare baseline, the instrument is
;;     not measuring dep-collection at all and every other number is meaningless.
;;   BASELINE       — a rule whose `where` uses ONLY rete-core ops. No user dep.
;;   SUBJECT        — the SAME rule shape, but its `where` calls `:usr::big?`.
;; If SUBJECT == BASELINE, the user fn was NOT collected → the false negative is real.
;; If SUBJECT == BASELINE + 1, it WAS collected → the documented cost is wrong.

(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])

;; the ONE helper, shared by the positive control and the subject
(:wat::rete::core::defn :usr::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::i64::> n 100))

;; ── POSITIVE CONTROL — ordinary call position. Collection here MUST work.
(:wat::core::defn :usr::calls-helper-plainly [] -> :wat::core::bool
  (:usr::big? 150))

;; ── BASELINE — a where body with NO user dep (rete-core only)
(:wat::rete::defrule :usr::rule-baseline
  :when [(:usr::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::i64::> ?c 100))]
  :then [(:usr::Hot :c ?c)])

;; ── SUBJECT — identical shape; the where body calls the USER fn instead
(:wat::rete::defrule :usr::rule-userfn
  :when [(:usr::Temp (?c <- :c))
         (:wat::rete::where (:usr::big? ?c))]
  :then [(:usr::Hot :c ?c)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pc  (:wat::kernel::fn-forms :usr::calls-helper-plainly
           (:wat::core::keyword/from-string "user::root-pc"))
     _p  (:wat::kernel::println
           (:wat::string::concat "POSITIVE-CONTROL (ordinary call) forms="
             (:wat::i64::to-string (:wat::core::length pc))))
     bl  (:wat::kernel::fn-forms :usr::rule-baseline
           (:wat::core::keyword/from-string "user::root-bl"))
     _b  (:wat::kernel::println
           (:wat::string::concat "BASELINE (where, no user dep)   forms="
             (:wat::i64::to-string (:wat::core::length bl))))
     sj  (:wat::kernel::fn-forms :usr::rule-userfn
           (:wat::core::keyword/from-string "user::root-sj"))
     _s  (:wat::kernel::println
           (:wat::string::concat "SUBJECT  (where CALLS :usr::big?) forms="
             (:wat::i64::to-string (:wat::core::length sj))))]
    (:wat::kernel::println
      "compare SUBJECT vs BASELINE: equal => the where-body dep was NOT collected")))
