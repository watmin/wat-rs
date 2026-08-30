;; rules-corpus 03 — REAL SOURCE BECOMES FACTS
;;
;; Corpus 01 proved AST nodes CAN be facts. Corpus 02 proved the gate/unlock chain reasons over
;; them. Both seeded facts BY HAND. This is the load-bearing unknown between those probes and an
;; actual migration: can a real `.wat` file on disk be turned into the fact base the rules read?
;;
;; DESIGN-STONE-wat-grep-is-a-feature moved the extractor into the stdlib (`wat/grep.wat`):
;; `:wat::grep::Node`/`Named`/`Span`/`Facts`/`facts-of`/`extent-of`/`q-match`. This probe now
;; CONSUMES those stdlib verbs instead of declaring its own — it stays a probe, and it is this
;; stone's regression check: the numbers below must not move, because a move that changes
;; behaviour is not a move.
;;
;; ─── ★ THE JOIN: the extractor's facts FEED THE RULES, on real source ────────
;; Corpus 01's three verdicts, now over a file from disk instead of five hand-written facts.
;; Nothing about the rules changed to accept real input — that is the point. The rules never
;; knew the facts were hand-made, and they do not know now that they are not.
(:wat::core::defrecord :fx::IsArrow   [id <- :wat::core::i64])
(:wat::core::defrecord :fx::IsHeadKw  [id <- :wat::core::i64])
(:wat::core::defrecord :fx::IsTypePos [id <- :wat::core::i64])

(:wat::rete::defrule :fx::arrow
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::core::string::= ?n "<-"))]
  :then [(:fx::IsArrow :id ?id)])

(:wat::rete::defrule :fx::head-kw
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::String/contains? ?n "::"))]
  :then [(:fx::IsHeadKw :id ?id)])

;; the prev-sibling JOIN that replaces fix-seq's single carried boolean — over real source now
(:wat::rete::defrule :fx::type-pos
  :when [(:wat::grep::Node (?id <- :id)  (?p <- :parent) (?i <- :index))
         (:wat::grep::Node (?aid <- :id) (?p <- :parent) (?ai <- :index))
         (:fx::IsArrow (?aid <- :id))
         (:wat::rete::where
           (:wat::rete::core::i64::= ?i (:wat::rete::core::i64::+ ?ai 1 :undefined 0)))]
  :then [(:fx::IsTypePos :id ?id)])

;; ★ THE SPAN JOIN — proves the coordinate is reachable from a condition. A three-way join,
;; Node × Named × Span, all sharing ?id: the arrow-symbol condition from `:fx::arrow`, plus
;; `:wat::grep::Span` re-joined on the SAME ?id, binding ?l to the line the arrow starts on.
(:wat::core::defrecord :fx::ArrowLine [id <- :wat::core::i64  line <- :wat::core::i64])

(:wat::rete::defrule :fx::arrow-line
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::grep::Span  (?id <- :id) (?l <- :line))
         (:wat::rete::where (:wat::rete::core::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::core::string::= ?n "<-"))]
  :then [(:fx::ArrowLine :id ?id :line ?l)])

(:wat::rete::defquery :fx::q-ArrowLine
  :params []
  :when [(:fx::ArrowLine (?id <- :id) (?l <- :line))])

(:wat::rete::defquery :fx::q-IsArrow
  :params []
  :when [(?fact <- :fx::IsArrow)])


(:wat::rete::defquery :fx::q-IsHeadKw
  :params []
  :when [(?fact <- :fx::IsHeadKw)])


(:wat::rete::defquery :fx::q-IsTypePos
  :params []
  :when [(?fact <- :fx::IsTypePos)])

;; ─── ★ THE PROVING RULE — a real :wat::grep::Match, built in one RHS ─────────
;; Five coordinates LHS-bound off :wat::grep::Span (line/col/end-line/end-col, plus the node's
;; own `kind` off :wat::grep::Node re-joined on the same ?id), `file` a literal in the RHS
;; (a property of the RUN, not of a node — the DESIGN's own argument), and a non-empty
;; `captures` vector holding one real :wat::grep::Capture. This is row 3/4/5's rule.
;;
;; ⚠ the vector constructor is :wat::rete::core::PersistentVector, NOT :wat::core::PersistentVector
;; — core's fails the rete `:then` fence with "is not total" (measured, DESIGN-STONE session).
(:wat::rete::defrule :fx::match-arrow
  :when [(:wat::grep::Node (?id <- :id) (?k <- :kind))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         ;; the ONE Source fact per file — how a rule learns which file it is matching in.
         ;; This line used to be a hardcoded "wat/fix.wat", which was right by coincidence for
         ;; exactly one file and wrong for every other.
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file     ?f
           :line     ?l
           :col      ?c
           :end-line ?el
           :end-col  ?ec
           :rule     "fx::match-arrow"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "kind" :value ?k)))])

(:wat::core::defn :fx::report [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [facts (:wat::grep::facts-of path (:wat::io::read-file path))
                    n   (:wat::core::length (:wat::grep::Facts/nodes facts))
                    m   (:wat::core::length (:wat::grep::Facts/named facts))
                    sp  (:wat::core::length (:wat::grep::Facts/spans facts))]
    (:wat::kernel::println
      (:wat::core::string::concat path
        (:wat::core::string::concat "  Node=" (:wat::core::str n)
          (:wat::core::string::concat "  Named=" (:wat::core::str m)
            (:wat::core::string::concat "  Span=" (:wat::core::str sp))))))))


(:wat::core::defn :fx::classify [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [facts (:wat::grep::facts-of path (:wat::io::read-file path))
     rules (:wat::core::PersistentVector (:fx::arrow) (:fx::head-kw) (:fx::type-pos) (:fx::arrow-line))
     s0    (:wat::rete::insert-all
             (:wat::rete::compile-all rules
               (:wat::core::PersistentVector (:fx::q-IsArrow) (:fx::q-IsHeadKw) (:fx::q-IsTypePos) (:fx::q-ArrowLine)))
             (:wat::grep::Facts/nodes facts))
     s1    (:wat::rete::insert-all s0 (:wat::grep::Facts/named facts))
     s2    (:wat::rete::insert-all s1 (:wat::grep::Facts/spans facts))
     fired (:wat::core::match (:wat::rete::fire-rules s2) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     arrow-lines (:wat::rete::query fired (:fx::q-ArrowLine))
     n-arrow-lines (:wat::core::length arrow-lines)]
    (:wat::kernel::println
      (:wat::core::string::concat path
        (:wat::core::string::concat "  arrows=" (:wat::core::str (:wat::core::length (:wat::rete::query fired (:fx::q-IsArrow))))
          (:wat::core::string::concat "  head-kw=" (:wat::core::str (:wat::core::length (:wat::rete::query fired (:fx::q-IsHeadKw))))
            (:wat::core::string::concat "  type-pos=" (:wat::core::str (:wat::core::length (:wat::rete::query fired (:fx::q-IsTypePos))))
              (:wat::core::string::concat "  arrow-line=" (:wat::core::str n-arrow-lines)
                (:wat::core::string::concat "  sample-line="
                  (:wat::core::if (:wat::core::i64::> n-arrow-lines 0)
                    (:wat::core::str
                      (:wat::core::Option/expect
                        (:wat::core::PersistentMap/get (:wat::core::first arrow-lines) "?l")
                        "q-ArrowLine: ?l"))
                    "none"))))))))))

;; ─── ★ THE MATCH — proves the stdlib fact base can feed a user-declared Match rule ──
(:wat::core::defn :fx::match [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [facts (:wat::grep::facts-of path (:wat::io::read-file path))
     rules (:wat::core::PersistentVector (:fx::match-arrow))
     s0    (:wat::rete::insert-all
             (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wat::grep::q-match)))
             (:wat::grep::Facts/nodes facts))
     s1    (:wat::rete::insert-all s0 (:wat::grep::Facts/spans facts))
     ;; the ONE Source fact — the rule joins it for :file, so it must be inserted like any other.
     s2    (:wat::rete::insert s1 (:wat::grep::Facts/source facts))
     fired (:wat::core::match (:wat::rete::fire-rules s2) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     matches (:wat::rete::query fired (:wat::grep::q-match))
     m       (:wat::core::Option/expect
               (:wat::core::PersistentMap/get (:wat::core::first matches) "?fact")
               "q-match: ?fact")]
    (:wat::kernel::println (:wat::core::str m))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; Real files, deliberately spanning shapes: the codemod itself, a rules file, and a probe
    ;; whose macro forms carry the reader-macro sigils that CORRUPTED the text-edit engine.
    (:fx::report "wat/fix.wat")
    (:fx::report "wat-scripts/perf/grid/neg-consumer.wat")
    (:fx::report "tests/macros/probe_do_splice_define_via_macro.wat")
    ;; ★ and the rules, reading those same files
    (:fx::classify "wat-scripts/perf/grid/neg-consumer.wat")
    (:fx::classify "tests/macros/probe_do_splice_define_via_macro.wat")
    ;; ★ and the proving rule — a real Match, built from the stdlib facts
    (:fx::match "wat/fix.wat")))
