;; 277-layout-shape-probe.wat — CAN RETE SEE LAYOUT? The disconfirming probe for wat-fmt's
;; step 2, written BEFORE the style table (DESIGN-wat-fmt-the-rule-set-is-the-product, SEQUENCING).
;;
;; ⛔ ITS JOB IS TO FAIL CHEAPLY IF THE SHAPE IS WRONG. Layout is not "does a symbol appear" —
;; every layout rule reduces to ONE capability: join two children of the same parent and compare
;; their LINES. If a rete rule cannot do that, no style table is expressible and the whole
;; sequencing is wrong. This probe does exactly that and nothing else.
;;
;; It also needs a DERIVED fact to feed a SECOND rule (:ls::Head is asserted by one rule and
;; joined by the next) — the forward-chaining the DESIGN predicted layout would need for width
;; propagation. If derived facts did not re-fire, this prints nothing.
;;
;; WHAT IT EMITS — one Match per child of every keyword-headed form, carrying the child's
;; OFFSET FROM ITS HEAD. That tuple set IS the layout shape of a form:
;;   head   the form's head keyword         (dispatch key — the DESIGN's "exclusivity by shape")
;;   idx    child index                     (0 = the head itself, dline always 0)
;;   dline  child.line - head.line          (0 = same line as the head)
;;   col    child.col                       (the indent column)
;;   kind   child's ast kind
;;
;; NO JUDGEMENT IS ENCODED HERE. It measures; the builder adjudicates. A rule that decided
;; "good" or "bad" would be my taste wearing a measurement's clothes.
;;
;; Usage:
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/scratch-pad/277-layout-shape-probe.wat

;; The head of a form: the child at index 0 that carries a name. DERIVED — rule 2 joins it.
(:wat::core::defrecord :ls::Head
  [form <- :wat::core::i64
   name <- :wat::core::String
   line <- :wat::core::i64
   col  <- :wat::core::i64])

(:wat::rete::defrule :ls::head-of
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::grep::Span  (?h <- :id) (?l <- :line) (?c <- :col))]
  :then [(:ls::Head :form ?p :name ?n :line ?l :col ?c)])

;; ★ THE CAPABILITY UNDER TEST: join a DERIVED head fact with a SIBLING child and subtract lines.
(:wat::rete::defrule :ls::child-offset
  :when [(:ls::Head         (?p <- :form) (?n <- :name) (?hl <- :line))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?i <- :index) (?ck <- :kind))
         (:wat::grep::Span  (?c <- :id) (?cl <- :line) (?cc <- :col) (?cel <- :end-line) (?cec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file ?f :line ?cl :col ?cc :end-line ?cel :end-col ?cec
           :rule "layout"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "head"  :value ?n)
                       (:wat::grep::Capture :name "idx"   :value (:wat::rete::i64::to-string ?i))
                       (:wat::grep::Capture :name "dline" :value (:wat::rete::i64::to-string
                                                                   (:wat::rete::i64::- ?cl ?hl :undefined 0)))
                       (:wat::grep::Capture :name "col"   :value (:wat::rete::i64::to-string ?cc))
                       (:wat::grep::Capture :name "kind"  :value ?ck)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :ls))
