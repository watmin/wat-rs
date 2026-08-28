;; rete-numerics-ops.wat — THE RETE NUMERICS MIGRATION CENSUS, ASKED STRUCTURALLY.
;;
;; Arc 255 Stone B-ii: `:wat::rete::core::i64::*` and `:wat::rete::core::f64::*` (the rete DSL's
;; per-type numeric surface) move to `:wat::rete::i64::*` / `:wat::rete::f64::*`, adjacent to
;; `:wat::rete::string::`. Copied from `core-numerics-ops.wat` (Stone B-i) one namespace segment
;; over — same reasoning, same discrimination, only the prefixes change.
;;
;; A text census cannot hold this line safely — it cannot tell a keyword LEAF (a call the codemod
;; must rewrite) from a COMMENT or a STRING LITERAL naming the same spelling (neither of which may
;; be touched). This asks for keyword leaves only — the population wat-fix actually rewrites.

(:wat::rete::defrule :nm::rete-i64-op
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::rete::core::i64::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-core-i64-op"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :nm::rete-f64-op
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::rete::core::f64::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-core-f64-op"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :nm))
