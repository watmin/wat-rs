;; defclause-over-defn.wat — WHO DECLARES A CLAUSE, AND ON WHOSE NAME?
;;
;; Census instrument for docs/excursus/2026/08/001-sns-sqs/a-defclause-outranks-a-defn/.
;;
;; THE QUESTION. `(:wat::core::defclause :N …)` registers a clause table for `:N` that WINS
;; over `:N`'s registered scheme at every call site (`src/check.rs:6068`, before the `env.get`
;; scheme lookup). So a `defclause` on a name that is ALSO a `defn` re-points dispatch. Before
;; that precedence can be ruled on, the corpus has to be counted: how many `defclause`
;; declarations are there, how many name something a `defn` also declares, and is the pair in
;; the SAME file (an author extending their own name) or a DIFFERENT one (a hijack)?
;;
;; ⛔ WHY THIS IS NOT A TEXT GREP. `grep -c defclause` counts the word — in prose, in a string
;; literal (`wat/fix.wat`'s own rename tables carry it), inside a longer name
;; (`:wat::fix::rehead-rete-defn`), and in the ARGUMENT position of some other call. What is
;; wanted is "the child at index 1 of a list whose child at index 0 is the keyword
;; `:wat::core::defclause`" — two integers and a kind in the fact base, and nothing a regex can
;; say. Text got this family of question wrong three times on this branch
;; (docs/excursus/.../the-stdlib-vends-only-wat/SCORE.md rows 1 and 5).
;;
;; WHAT IT EMITS — three rules, and the third is the one that matters:
;;
;;   defclause-declaration                  one per `(:wat::core::defclause :N …)`, capture N
;;   defn-declaration                       one per `(:wat::core::defn :N …)` /
;;                                          `(:wat::rete::core::defn :N …)`, capture N
;;   defclause-and-defn-in-the-same-file    the SAME-ORIGIN collision, found by a self-join on
;;                                          the name — wat-grep's fact base is reset per file,
;;                                          so a join IS a same-file predicate, and needs no
;;                                          file comparison at all
;;
;; ⭑ CROSS-FILE IS THE COMPLEMENT, NOT A FOURTH RULE. Two files never share a fact base, so a
;; cross-origin pair cannot be a join here by construction. It is computed OUTSIDE, from the
;; first two rules' output: a `defclause` name that no file declares as a `defn` IN THE SAME
;; FILE, but some other file does. That is why rules 1 and 2 print every declaration rather
;; than only the colliding ones.
;;
;; Usage — census over every tracked .wat:
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/grep/defclause-over-defn.wat
;;
;; ⚠ TWO BLIND SPOTS, both inherited and both named rather than papered over:
;;
;; 1. QUOTED TEMPLATES. `facts-of` walks every node, including quasiquoted macro bodies, and
;;    the fact base carries no quote marker (defined-twice.wat names the same hole). A
;;    `defclause` inside a macro template counts here as a declaration. Read every hit.
;; 2. GENERATED NAMES. A `defrecord :A::P` vends `:A::P'`, `:A::P/f`, `:A::is-P?` — names no
;;    node spells, so a name-level intersection cannot see a `defclause` aimed at one. That
;;    class is measured from the SYMBOL TABLE, not from source; this instrument answers the
;;    `defn` question it is named for and no more.

(:wat::core::defrecord :dcd::ClauseHead [parent <- :wat::core::i64])
(:wat::core::defrecord :dcd::DefnHead   [parent <- :wat::core::i64])
(:wat::core::defrecord :dcd::ClauseName [id <- :wat::core::i64  name <- :wat::core::String])
(:wat::core::defrecord :dcd::DefnName   [id <- :wat::core::i64  name <- :wat::core::String])

;; ── the two declaring heads ─────────────────────────────────────────────────────────

(:wat::rete::defrule :dcd::clause-head
  :when [(:wat::grep::Node  (?id <- :id) (?p <- :parent) (?i <- :index) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defclause"))]
  :then [(:dcd::ClauseHead :parent ?p)])

;; BOTH spellings. `:wat::rete::core::defn` is a real declaring head in this corpus (47 forms);
;; a finder that knew only `:wat::core::defn` would under-report the collision set.
(:wat::rete::defrule :dcd::defn-head
  :when [(:wat::grep::Node  (?id <- :id) (?p <- :parent) (?i <- :index) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where
           (:wat::rete::core::or
             (:wat::rete::string::= ?n ":wat::core::defn")
             (:wat::rete::string::= ?n ":wat::rete::core::defn")))]
  :then [(:dcd::DefnHead :parent ?p)])

;; ── the declared name — child index 1 of such a list ────────────────────────────────

(:wat::rete::defrule :dcd::clause-name
  :when [(:dcd::ClauseHead (?p <- :parent))
         (:wat::grep::Node  (?id <- :id) (?p <- :parent) (?i <- :index))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?i 1))]
  :then [(:dcd::ClauseName :id ?id :name ?n)])

(:wat::rete::defrule :dcd::defn-name
  :when [(:dcd::DefnHead (?p <- :parent))
         (:wat::grep::Node  (?id <- :id) (?p <- :parent) (?i <- :index))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?i 1))]
  :then [(:dcd::DefnName :id ?id :name ?n)])

;; ── the report ──────────────────────────────────────────────────────────────────────

(:wat::rete::defrule :dcd::report-clause
  :when [(:dcd::ClauseName (?id <- :id) (?n <- :name))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "defclause-declaration"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :dcd::report-defn
  :when [(:dcd::DefnName (?id <- :id) (?n <- :name))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "defn-declaration"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

;; ★ THE SAME-ORIGIN COLLISION — the join IS the same-file predicate.
(:wat::rete::defrule :dcd::same-file-collision
  :when [(:dcd::ClauseName (?a <- :id) (?n <- :name))
         (:dcd::DefnName   (?b <- :id) (?n <- :name))
         (:wat::grep::Span (?a <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "defclause-and-defn-in-the-same-file"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :dcd))
