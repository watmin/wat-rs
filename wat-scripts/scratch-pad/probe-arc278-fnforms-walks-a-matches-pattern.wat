;; probe-arc278-fnforms-walks-a-matches-pattern.wat — the SECOND instance of
;; "the closure walker refuses a valid program", found where the first fix declined to look.
;;
;; HISTORY, because this file changed role and the verdict line must say which role it is in.
;; It was written as a RED gate on 2026-08-12 to test a READ: that `closure_extract`'s
;; `Boundary::MatchesSubject => {}` arm — an empty arm indistinguishable from `Ordinary` — let
;; the walker descend into a `matches?` PATTERN and read its DSL tokens as code. It did.
;;
;;   free symbol `=` does not resolve to a parent define or substrate primitive
;;   (:26:21 — inside the pattern; it never even reached the pattern VARIABLE `?gr`)
;;
;; The empty arm carried a comment claiming the fall-through was SAFE. That claim was borrowed
;; wholesale from `make-rule`, where it IS true — `make-rule`'s `:when` is itself a `quote`
;; form, so the recursion meets `AllData` and stops. A `matches?` pattern is NOT quoted, so
;; nothing downstream stopped the walk. One case's reasoning was applied to another case.
;;
;; CONSEQUENCE while it stood: NO fn containing a `matches?` could be closure-extracted — so a
;; user service whose main used `matches?` could not ship its forms to the child. That is the
;; same defect class as probe-arc278-fnforms-walks-into-quoted-data.wat (the walker reading DATA
;; as CODE), left standing for a second form after the first was fixed.
;;
;; ★ THIS FILE IS NOW A REGRESSION GATE. The arm honours `MatchesSubject` (subject only, exactly
;; as `resolve::walk` does), so BOTH arms return and reaching the final line is the PASS. If the
;; subject arm ever raises again, the walker has gone back to reading a pattern as code.
;;
;; ⚠ NON-VACUITY — the two defns are shipped through the SAME `fn-forms` call shape and both
;; return `:bool`. They differ in exactly ONE thing: the subject's body holds a `matches?` whose
;; pattern binds `?gr`. If BOTH raise, `matches?` is not the discriminator and this instrument is
;; measuring something else. If NEITHER raised BEFORE the fix, the defect was not here.

(:wat::core::defstruct :probe::Paper
  [grace-residue <- :wat::core::f64])

;; ── the CONTROL: no matches?, no pattern, no DSL tokens ─────────────────────────────────────
(:wat::core::defn :probe::control [p <- :probe::Paper] -> :wat::core::bool
  (:wat::f64::< (:probe::Paper/grace-residue p) 5.0))

;; ── the SUBJECT: identical shipping, but the body holds a `matches?`. `=` and `<` here are
;; PATTERN GRAMMAR owned by check.rs's `infer_form_matches` walker — not call heads — and `?gr`
;; is bound by the pattern, not by any enclosing scope. Reading any of them as code is the bug.
(:wat::core::defn :probe::subject [p <- :probe::Paper] -> :wat::core::bool
  (:wat::form::matches? p
    (:probe::Paper (= ?gr :grace-residue) (< ?gr 5.0))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ARM 1 — the control. A closure comes back.
     ctl (:wat::kernel::fn-forms :probe::control
           (:wat::core::keyword/from-string "user::root-ctl"))
     _c  (:wat::kernel::println
           (:wat::string::concat "CONTROL forms="
             (:wat::i64::to-string (:wat::core::length ctl))))
     ;; ARM 2 — the subject. This raised before the fix, naming `=` from inside the pattern.
     sub (:wat::kernel::fn-forms :probe::subject
           (:wat::core::keyword/from-string "user::root-sub"))
     _s  (:wat::kernel::println
           (:wat::string::concat "SUBJECT forms="
             (:wat::i64::to-string (:wat::core::length sub))))]
    (:wat::kernel::println
      "PASS — both arms extracted; a matches? PATTERN is DATA to the closure walker, not code")))
