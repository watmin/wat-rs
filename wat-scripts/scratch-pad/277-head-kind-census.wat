;; 277-head-kind-census.wat — CAN EVERY FORM BE DISPATCHED ON ITS HEAD SYMBOL?
;;
;; `[[DESIGN-wat-fmt-the-rule-set-is-the-product]]` rests its extensibility argument on ONE
;; structural claim: "a layout rule dispatches on HEAD SYMBOL", so two rules can never both fire
;; on one node and no conflict resolution is ever needed. That claim has a precondition nobody
;; measured: **every form must HAVE a head symbol.**
;;
;; It does not. `wat/core.wat:1409` emits  `(~(:wat::core::first step) ~a ~@(…))  — the head is an
;; UNQUOTE, computed at expansion. A rule keyed on the head name cannot see it.
;;
;; This census emits the KIND of every form's child-0. The distribution is the answer:
;; anything that is not `keyword` is a form head-dispatch cannot reach.
;;
;; ⚠ It is also a blind-spot census for `277-layout-shape-probe.wat` itself, whose `:ls::Head`
;; rule joins `Named` — so a form with an unnameable head emitted NO layout facts at all and was
;; silently absent from the survey. This counts what that survey could not see.
;;
;; Usage:
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/scratch-pad/277-head-kind-census.wat

;; ⚠ AMENDED after the first run. v1 fired on child-0 of EVERY node, so a `match` arm
;; `((:wat::core::Ok v) body)` — a list whose child-0 is a pattern list — counted as a
;; "computed head", and so did the first element of every vector and map. It reported 9,115
;; list-heads, which is NOT the number of forms head-dispatch cannot reach.
;; v2 joins the PARENT's kind and reports it, so a call can be told from an arm.
(:wat::rete::defrule :hk::head-kind
  :when [(:wat::grep::Node   (?p <- :id) (?pk <- :kind))
         (:wat::grep::Node   (?h <- :id) (?p <- :parent) (?i <- :index) (?k <- :kind))
         (:wat::rete::where  (:wat::rete::i64::= ?i 0))
         (:wat::grep::Span   (?h <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "head-kind"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "kind" :value ?k)
                       (:wat::grep::Capture :name "pkind" :value ?pk)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :hk))
