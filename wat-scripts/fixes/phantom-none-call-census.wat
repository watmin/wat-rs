;; wat-scripts/fixes/phantom-none-call-census.wat — the phantom-`None` finder (CENSUS ONLY).
;;
;; `(:wat::core::None <anything>)` type-checks for a non-primitive type keyword and raises
;; UnknownFunction at run time. `None` is a KEYWORD, not a callable. See
;; docs/arc/2026/04/109-kill-std/NOTE-none-is-not-a-function.md.
;;
;; ⛔ WHY THIS CANNOT BE A GREP. A match arm
;;      (:wat::core::None false)
;; is SYNTACTICALLY IDENTICAL to a call
;;      (:wat::core::None :fanout::Seen::Reply)
;; — a list whose head is the None keyword and whose index-1 child is anything. There are
;; ~150 of the first in this tree and a handful of the second. Only the ENCLOSING FORM tells
;; them apart, and a regex cannot see an enclosing form. A grep census of this returns a
;; number that means nothing; the NOTE itself was born from exactly that miscount.
;;
;; THE DISCRIMINATOR, structurally: a list L is a phantom call when
;;   1. L's child at index 0 is the keyword :wat::core::None (or the retiring bare :None), AND
;;   2. L has a child at index 1 (a nullary (None) is not a call), AND
;;   3. L is NOT an arm of a match — i.e. NOT (L's parent M has :wat::core::match at index 0
;;      and L sits at index >= 2 of M, the arm positions).
;;
;; Census:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/phantom-none-call-census.wat
;;
;; No applier. The repair is one token (`:wat::core::None` bare) and is hand-typed; this file
;; is the instrument that finds WHERE. Rung 2 on the extirpare ladder — a check that fires,
;; not a shape the mistake cannot take. Rung 3 is extending arc 242's Doctrine 1 to reject a
;; TYPE keyword in value position for every type, not only the primitives; that is held while
;; `main` is mid-migration (see the NOTE's ruling).

;; ── intermediate activation facts — identity only ────────────────────────────
(:wat::core::defrecord :pn::NoneHeaded [id <- :wat::core::i64])
(:wat::core::defrecord :pn::HasArg     [id <- :wat::core::i64])
(:wat::core::defrecord :pn::MatchList  [id <- :wat::core::i64])
(:wat::core::defrecord :pn::ArmOf      [id <- :wat::core::i64])

;; 1a. a list whose head (index 0) is the keyword :wat::core::None
(:wat::rete::defrule :pn::none-headed-qualified
  :when [(:wat::grep::Node  (?k <- :id) (?l <- :parent) (?i <- :index) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::None"))]
  :then [(:pn::NoneHeaded ?l)])

;; 1b. the retiring bare spelling (check.rs:2422 — "bare :None is a retiring grammar exception")
(:wat::rete::defrule :pn::none-headed-bare
  :when [(:wat::grep::Node  (?k <- :id) (?l <- :parent) (?i <- :index) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":None"))]
  :then [(:pn::NoneHeaded ?l)])

;; 2. that list has something at index 1 — a bare `(None)` is not a call
(:wat::rete::defrule :pn::has-arg
  :when [(:wat::grep::Node (?c <- :id) (?l <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 1))]
  :then [(:pn::HasArg ?l)])

;; 3a. a list that IS a match form — head keyword :wat::core::match at index 0
(:wat::rete::defrule :pn::match-list
  :when [(:wat::grep::Node  (?h <- :id) (?m <- :parent) (?i <- :index) (?kind <- :kind))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::match"))]
  :then [(:pn::MatchList ?m)])

;; 3a-bis. …and the rete namespace carries its OWN match. Found by the census itself:
;; the first run reported where-control.wat:221 and where-record.wat:163, and reading them
;; showed both were `:wat::rete::core::match` ARMS. A finder that knows one spelling of the
;; enclosing form reports the other spelling's arms as calls. Head spellings in this tree:
;; :wat::core::match (2499), :wat::rete::core::match (5). `:fx::match` is a userland defn.
(:wat::rete::defrule :pn::match-list-rete
  :when [(:wat::grep::Node  (?h <- :id) (?m <- :parent) (?i <- :index) (?kind <- :kind))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::rete::core::match"))]
  :then [(:pn::MatchList ?m)])

;; 3b. …so a list sitting at index >= 2 of a match form is an ARM, not a call.
;;     index 0 is the `match` head, index 1 the scrutinee; arms start at 2.
(:wat::rete::defrule :pn::arm-of
  :when [(:wat::grep::Node (?l <- :id) (?m <- :parent) (?i <- :index))
         (:pn::MatchList (?m <- :id))
         (:wat::rete::where (:wat::rete::i64::>= ?i 2))]
  :then [(:pn::ArmOf ?l)])

;; THE REPORT — None-headed, has an argument, and is not a match arm.
(:wat::rete::defrule :pn::phantom-none-call
  :when [(:pn::NoneHeaded (?l <- :id))
         (:pn::HasArg     (?l <- :id))
         (:wat::rete::not (:pn::ArmOf (?l <- :id)))
         (:wat::grep::Span (?l <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "phantom-none-call"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :pn))
