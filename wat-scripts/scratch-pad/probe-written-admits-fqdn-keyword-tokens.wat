;; wat-scripts/scratch-pad/probe-written-admits-fqdn-keyword-tokens.wat — arc 294, the grok-rete
;; replay, STEP 0 (the recorded grep-rules codemods must work on today's binary).
;;
;; QUESTION. Since 0b5742cc7 a rule sees `:wat::grep::Named`'s name FOLDED
;; (`:wat::core::string::concat` -> `wat.core.string/concat`), so a rename rule can no longer read
;; the spelling it must rewrite. Step 0's design gives `:wat::grep::Written` a verbatim `:text`.
;; That design rests on ONE premise, asked here: does `Written` exist for an FQDN keyword token,
;; in BOTH head position and argument position? `Written` exists iff the node is nameable, sits on
;; a single line, and its span's width equals the length of its verbatim `ast-name`
;; (wat/grep.wat, the `written?` guard).
;;
;; This rule joins `Written` (not `Span`) and matches the FOLDED spelling of the string-verb family.
;; Every Match it prints is a token that `Written` admits.
;;
;; Run (the target lives OUTSIDE wat-scripts/ because it spells retired names on purpose):
;;   printf '["<abs path>/headarg.wat"]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/scratch-pad/probe-written-admits-fqdn-keyword-tokens.wat
;;   target (2 lines):
;;     (:wat::core::defn :u::a [] -> :wat::core::String (:wat::core::string::interpolate "x"))
;;     (:wat::core::defn :u::b [] -> :wat::core::nil (:u::take :wat::core::string::concat))
;; EXPECT, per rule (read the `:rule` field of each Match):
;;   written-string-verb  2   the QUESTION. One is `wat.core.string/interpolate` at line 1, the head
;;                            of a call. The other is `wat.core.string/concat` at line 2, an argument.
;;                            0 means Written refuses FQDN tokens and the design is dead. 1 means it
;;                            refuses one position.
;;   span-string-verb     2   CONTROL. The same match joined on Span. It proves the folded match
;;                            itself works, so a shortfall above can only be the Written join.
;;   written-fqdn         0   NEGATIVE CONTROL. Today's dead codemods, reproduced on purpose: a rule
;;                            comparing the unfolded spelling sees nothing, because Named is folded.
;;                            Any Match here means the fold is not live, and the diagnosis is wrong.

(:wat::rete::defrule :pw::span-string-verb
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::core::enum::= ?k (:wat::grep::NodeKind.Keyword {})))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n "wat.core.string/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "span-string-verb"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "folded" :value ?n)))])

(:wat::rete::defrule :pw::written-fqdn
  :when [(:wat::grep::Node    (?id <- :id) (?k <- :kind))
         (:wat::grep::Named   (?id <- :id) (?n <- :name))
         (:wat::grep::Written (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source  (?f <- :file))
         (:wat::rete::where (:wat::rete::core::enum::= ?k (:wat::grep::NodeKind.Keyword {})))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::string::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "written-fqdn"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "folded" :value ?n)))])

(:wat::rete::defrule :pw::written-string-verb
  :when [(:wat::grep::Node    (?id <- :id) (?k <- :kind))
         (:wat::grep::Named   (?id <- :id) (?n <- :name))
         (:wat::grep::Written (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source  (?f <- :file))
         (:wat::rete::where (:wat::rete::core::enum::= ?k (:wat::grep::NodeKind.Keyword {})))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n "wat.core.string/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "written-string-verb"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "folded" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :pw))
