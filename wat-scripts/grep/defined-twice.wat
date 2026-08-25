;; defined-twice.wat — THE SELF-JOIN. Two occurrences compared TO EACH OTHER.
;;
;; The question: does this file define the same name twice?
;;
;; This is the shape text cannot reach even in principle. Every other program in this corpus asks
;; "is there a node like X" — one fact at a time, and a sufficiently clever regex could approximate
;; some of them. This one asks "are there TWO nodes that AGREE with each other", which is a
;; relation between occurrences, and a search that returns a list of hits has no way to express it.
;; You would have to sort the hits afterwards, in another language, by hand.
;;
;; In rete it is one condition repeated with a shared variable:
;;
;;     (:dt::Defines (?a <- :id) (?n <- :name))     ;; some definition, named ?n
;;     (:dt::Defines (?b <- :id) (?n <- :name))     ;; ANOTHER, and the SAME ?n binds both
;;
;; The second condition is the first one again. Binding `?n` twice is the join: rete will only
;; pair rows whose names are equal, because a variable means the same value everywhere it appears.
;;
;; ⚠ AND THE ORDERING GUARD, which is the thing to learn. A self-join matches BOTH directions —
;; (a,b) and (b,a) — so every duplicate is reported twice, mirrored. `(i64::< ?a ?b)` keeps one of
;; each pair. Without it the count is exactly doubled, which is the kind of wrong number that looks
;; plausible enough to publish.

;; ─── WHAT IT FOUND, AND WHY THE ANSWER IS "NONE" ────────────────────────────────────────────
;;
;; Across all 54 stdlib files: 2 hits, both `:user::main` in `wat/bracket.wat` (549, 557) — and
;; BOTH ARE FALSE POSITIVES. They sit inside a QUASIQUOTE, in the two mutually-exclusive arms of
;; an `if` in a macro template (DIAL / NON-DIAL). Exactly one is ever emitted. So the honest
;; reading is: the stdlib has NO duplicate definitions, and this program has a blind spot.
;;
;; ⛔ THE BLIND SPOT IS WORTH MORE THAN THE ANSWER. `facts-of` walks EVERY node, including
;; quasiquoted macro templates — it has no notion of quote-as-data. `src/rete/purity.rs:1257`
;; does: it has an explicit arm skipping `quote` / `quasiquote` / `holon-literal` sub-forms
;; because they are DATA, not calls. The fact base carries no such distinction, so a rule asking
;; about "definitions" cannot currently tell a definition from a TEMPLATE FOR one.
;;
;; Structure beats text here — and structure has its own way of being wrong. Naming it is the
;; point: a future `:wat::grep::Quoted [id]` fact would let a rule exclude template interiors,
;; and until that exists, any rule about definitions carries this caveat.

;; a definition's NAME node: child index 1 of a list whose child index 0 is a declaring keyword
(:wat::core::defrecord :dt::Declarator [parent <- :wat::core::i64])
(:wat::core::defrecord :dt::Defines    [id <- :wat::core::i64  name <- :wat::core::String])

(:wat::rete::defrule :dt::declarator
  :when [(:wat::grep::Node  (?id <- :id) (?p <- :parent) (?i <- :index) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::i64::= ?i 0))
         (:wat::rete::where
           (:wat::rete::core::or
             (:wat::rete::core::string::= ?n ":wat::core::defn")
             (:wat::rete::core::or
               (:wat::rete::core::string::= ?n ":wat::core::defrecord")
               (:wat::rete::core::string::= ?n ":wat::core::defmacro"))))]
  :then [(:dt::Declarator :parent ?p)])

(:wat::rete::defrule :dt::defines
  :when [(:dt::Declarator (?p <- :parent))
         (:wat::grep::Node  (?id <- :id) (?p <- :parent) (?i <- :index))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::i64::= ?i 1))]
  :then [(:dt::Defines :id ?id :name ?n)])

;; ★ THE SELF-JOIN — the same condition twice, `?n` shared, `?a < ?b` keeping one of each mirror
(:wat::rete::defrule :dt::twice
  :when [(:dt::Defines (?a <- :id) (?n <- :name))
         (:dt::Defines (?b <- :id) (?n <- :name))
         ;; the ordering guard, back where it belongs: immediately after the two conditions
         ;; whose variables it relates, before the span lookup that only the survivor needs.
         (:wat::rete::where (:wat::rete::core::i64::< ?a ?b))
         (:wat::grep::Span (?b <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "defined-more-than-once-in-one-file"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :dt))
