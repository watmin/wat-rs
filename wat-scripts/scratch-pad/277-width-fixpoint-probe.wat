;; 277-width-fixpoint-probe.wat — CAN A RULE DERIVE A FORM'S RENDERED WIDTH, BOTTOM-UP, TO A FIXPOINT?
;;
;; ⛔ THE DISCONFIRMING PROBE FOR THE BUDGET RULING. Builder ruled 120 columns and a TIERED `fn`
;; (whole form / signature / full breakout). Tier selection needs a form's width AS IT WOULD RENDER
;; ON ONE LINE — which is NOT its current span width, because the current text may already be
;; broken across lines. That number has to be DERIVED, and a parent's derivation depends on its
;; children's. `[[DESIGN-wat-fmt-the-rule-set-is-the-product]]` predicted exactly this:
;;   "a node's rendered WIDTH depends on its children's widths -> bottom-up derivation to a FIXPOINT"
;; and nothing has ever driven it. If this fails, the tiering is not buildable as designed and the
;; brief that rests on it would be impossible work.
;;
;; THE DERIVATION
;;   leaf     (no children)  width = its own span width          ← the base case
;;   interior (n children)   width = 2 parens + Σ child widths + (n-1) separators
;;                                 = Σ + n + 1
;;   The interior rule fires ONLY when the number of children WITH a width equals the number of
;;   children — that completeness test is what makes it a fixpoint rather than a race.
;;
;; ★ LEAFNESS IS ASKED, NOT ASSUMED. A leaf is a node with ZERO children (`acc::count` = 0), never
;;   a hard-coded kind list — so a new AST kind cannot silently be misclassified.
;;
;; ★★ THE NON-VACUITY CONTROL IS BUILT IN. For a form ALREADY on one line, the DERIVED width must
;;    EQUAL its actual span width. The report emits both, so the check is a subtraction, not a
;;    belief. A derivation that agrees with the source everywhere it can be checked is one that can
;;    be trusted where it cannot (the multi-line forms, which is the whole point).
;;
;; Usage:
;;   printf '["wat/io.wat"]\n' | ./target/release/wat --grep ./wat-scripts/scratch-pad/277-width-fixpoint-probe.wat

(:wat::core::defrecord :w::Width
  [id     <- :wat::core::i64
   parent <- :wat::core::i64
   w      <- :wat::core::i64])

;; BASE CASE — zero children, and its own span does not straddle a line.
(:wat::rete::defrule :w::leaf
  :when [(:wat::grep::Node  (?id <- :id) (?p <- :parent))
         (:wat::grep::Span  (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (?n <- (:wat::rete::acc::count) :from (:wat::grep::Node (?id <- :parent)))
         (:wat::rete::where (:wat::rete::i64::= ?n 0))
         (:wat::rete::where (:wat::rete::i64::= ?l ?el))]
  :then [(:w::Width :id ?id :parent ?p
           :w (:wat::rete::i64::- ?ec ?c :undefined 0))])

;; INDUCTIVE STEP — fires only once EVERY child carries a width. This is the fixpoint.
(:wat::rete::defrule :w::interior
  :when [(:wat::grep::Node (?id <- :id) (?p <- :parent))
         (?n  <- (:wat::rete::acc::count) :from (:wat::grep::Node (?id <- :parent)))
         (:wat::rete::where (:wat::rete::i64::> ?n 0))
         (?nw <- (:wat::rete::acc::count) :from (:w::Width (?id <- :parent)))
         (:wat::rete::where (:wat::rete::i64::= ?n ?nw))
         (?s  <- (:wat::rete::acc::sum ?cw) :from (:w::Width (?id <- :parent) (?cw <- :w)))]
  :then [(:w::Width :id ?id :parent ?p
           :w (:wat::rete::i64::+ ?s
                (:wat::rete::i64::+ ?n 1 :undefined 0) :undefined 0))])

;; THE REPORT — derived beside actual, so the control is a subtraction.
(:wat::rete::defrule :w::report
  :when [(:w::Width          (?id <- :id) (?w <- :w))
         (:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::rete::where  (:wat::rete::string::= ?k "list"))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "width"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "derived" :value (:wat::rete::i64::to-string ?w))
                       (:wat::grep::Capture :name "actual"  :value (:wat::rete::i64::to-string
                                                                     (:wat::rete::i64::- ?ec ?c :undefined 0)))
                       (:wat::grep::Capture :name "endline" :value (:wat::rete::i64::to-string ?el))))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :w))
