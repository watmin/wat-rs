;; R3 — `let` layout. THE ORCHESTRATOR'S ACCEPTANCE PROBE, 2026-09-05.
;;
;; ⛔ WHY THIS FILE EXISTS. The stone's acceptance was "R11 is added as a NEW FILE and nothing
;; else", and it passed — but R11 was the SECOND rule, and the interesting claim is about the
;; THIRD. After the refutation, R11 became a default rule gated on `Claim`, and the SCORE asserted
;; "a later let/match rule asserts its own Claim; R11 is not edited." **That is a claim about a
;; rule nobody had written.** This is that rule, written by the orchestrator, to falsify it.
;;
;; If adding this file requires ONE edit to fmt.wat, defn.wat or siblings.wat, the extensibility
;; requirement has failed and that is the finding.
;;
;; ★ AND IT IS NOW A RULED STYLE. Builder, 2026-09-05, after this file was first written as a
;; throwaway probe:
;;
;;   (:wat::core::let           ;; open the block   <- NOTHING rides the head line
;;     [y (:wat::core::+ x 1)]  ;; one binder per line
;;     y)                       ;; body after binders
;;
;; ⛔ NOTE HOW THIS DIFFERS FROM R1. A `defn`'s NAME rides its head line; a `let`'s head line
;; carries nothing at all. And the arithmetic differs: a `defn` arg is a TRIPLE (`x <- T`, so
;; every 3rd child starts a line) while a `let` binder is a PAIR (`y expr`, so every 2nd does).
;; A rule cannot be copied between forms; each names its own shape.
;;
;; Break names a kind ("block" / "align"); the emitter computes the rest.
;; The binding VECTOR is its own dispatch target — see let-bindings.wat.

(:wat::load-file! "let-bindings.wat")

(:wat::rete::defrule :fmt::let-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))]
  :then [(:wat::fmt::Claim :form ?p)])

(:wat::rete::defrule :fmt::let-bindings-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?bi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))]
  :then [(:wat::fmt::Break :id ?b :kind "block")])

;; the BODY — every child after the binding vector — starts its own line.
(:wat::rete::defrule :fmt::let-body-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?body <- :id) (?p <- :parent) (?bi <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?bi 1))]
  :then [(:wat::fmt::Break :id ?body :kind "block")])
