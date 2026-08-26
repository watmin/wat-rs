;; can-raise.wat — WHICH FUNCTIONS CAN PANIC? (transitive containment, via a recursive rule)
;;
;; Every other program in this corpus asks about a node, or about two nodes in a fixed
;; relationship — a head, a sibling, a parent, a self-join. This one asks a question with NO
;; bounded shape: *does this function contain, at ANY depth, a call that can raise?*
;;
;; "At any depth" is transitive closure, and a rete rule can compute it by FEEDING ITSELF:
;;
;;     direct : a node is Under its own parent
;;     step   : if X is Under A, and N's parent is X, then N is Under A     <- matches its own :then
;;
;; Forward chaining runs that to a fixed point. Verified on `(a (b (c (d))))`: 24 Under facts,
;; which is exactly the sum over nodes of each node's ancestor count. Not "looks right" — counted.
;;
;; THE SUBJECTS are the substrate's partial verbs — the ones with a domain hole:
;;   :wat::core::first          raises on an empty sequence
;;   :wat::core::Option/expect  raises on None
;;   :wat::core::nth            raises out of range
;; A call to one is a place this function can die. A DEFN CONTAINING one is a function whose
;; caller inherits that.
;;
;; ⚠ WHAT THIS DOES NOT KNOW, stated because a census that hides its blind spots is a liar:
;;   - a guarded call is still reported. `(if (empty? xs) … (first xs))` is safe and matches.
;;     This finds the SITES to audit, not the bugs.
;;   - macro-template interiors are included (see defined-twice.wat's finding).
;;   - it says nothing about transitive risk through a CALLEE — only lexical containment.

(:wat::core::defrecord :cr::Under   [anc <- :wat::core::i64  node <- :wat::core::i64])
(:wat::core::defrecord :cr::Partial [id  <- :wat::core::i64  verb <- :wat::core::String])
(:wat::core::defrecord :cr::Defn    [id  <- :wat::core::i64  name <- :wat::core::String])

;; ── the transitive closure ──────────────────────────────────────────────────────────
(:wat::rete::defrule :cr::a-direct
  :when [(:wat::grep::Node (?n <- :id) (?p <- :parent))]
  :then [(:cr::Under :anc ?p :node ?n)])

(:wat::rete::defrule :cr::b-step
  :when [(:cr::Under (?a <- :anc) (?mid <- :node))
         (:wat::grep::Node (?n <- :id) (?mid <- :parent))]
  :then [(:cr::Under :anc ?a :node ?n)])

;; ── a call that can raise: a partial verb in HEAD position ──────────────────────────
(:wat::rete::defrule :cr::c-partial
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind) (?i <- :index))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where
           (:wat::rete::core::or
             (:wat::rete::string::= ?n ":wat::core::first")
             (:wat::rete::core::or
               (:wat::rete::string::= ?n ":wat::core::Option/expect")
               (:wat::rete::string::= ?n ":wat::core::nth"))))]
  :then [(:cr::Partial :id ?id :verb ?n)])

;; ── a TOP-LEVEL defn — its parent is 0, the walk's root ─────────────────────────────
(:wat::rete::defrule :cr::d-defn
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::grep::Named (?h <- :id) (?hn <- :name))
         (:wat::grep::Node  (?nm <- :id) (?p <- :parent) (?ni <- :index))
         (:wat::grep::Named (?nm <- :id) (?fname <- :name))
         (:wat::grep::Node  (?p <- :id) (?root <- :parent))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::where (:wat::rete::i64::= ?ni 1))
         (:wat::rete::where (:wat::rete::i64::= ?root 0))
         (:wat::rete::where (:wat::rete::string::= ?hn ":wat::core::defn"))]
  :then [(:cr::Defn :id ?p :name ?fname)])

;; ── ★ the containment join: a defn that CONTAINS a raising call, at any depth ───────
(:wat::rete::defrule :cr::e-match
  :when [(:cr::Defn    (?d <- :id) (?fname <- :name))
         (:cr::Partial (?c <- :id) (?verb <- :verb))
         (:cr::Under   (?d <- :anc) (?c <- :node))
         ;; ⚠ the span is the CALL's, not the defn's. Reporting at the defn gave one Match per
         ;; (defn, call) PAIR — the same function repeated with identical coordinates, which reads
         ;; as several findings and is one. At the call site every Match is a distinct place, and
         ;; the containing function rides along as a capture. Same facts, honest granularity.
         (:wat::grep::Span (?c <- :id) (?l <- :line) (?c2 <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c2 :end-line ?el :end-col ?ec
           :rule "defn-contains-a-partial-call"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "fn"   :value ?fname)
                       (:wat::grep::Capture :name "verb" :value ?verb)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :cr))
