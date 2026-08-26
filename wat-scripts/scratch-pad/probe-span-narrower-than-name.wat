;; wat-scripts/scratch-pad/probe-span-narrower-than-name.wat — arc 255, the four-homes STOP-1.
;;
;; THE ENCODED QUESTION: *which named keyword nodes carry a span that is not as wide as the name
;; they claim?* A node whose `Named` name is N and whose `Span` covers fewer columns than N has
;; characters cannot have been WRITTEN — the reader MANUFACTURED it and handed it a span borrowed
;; from whatever source text produced it.
;;
;; This is not a char question. `crates/wat-reader/src/parser.rs:390-410` shows SIX reader macros
;; that synthesize a keyword node and clone the literal's span onto it:
;;
;;   '  -> (:wat::core::quote X)              ~  -> (:wat::core::unquote X)
;;   `  -> (:wat::core::quasiquote X)         ~@ -> (:wat::core::unquote-splicing X)
;;   \c -> (:wat::core::char/of "c")          #holon -> (:wat::holon::literal X)
;;
;; Every one of those names is a rename waiting to corrupt its own corpus, because a rules codemod
;; that splices a replacement into `Span` writes N characters into a 1-or-2-character hole.
;;
;; ⚠ THIS PROBE CANNOT BE WRITTEN WITH grep. The defect is a DISAGREEMENT BETWEEN TWO FACTS about
;; the same node — text has no way to ask it.
;;
;; Usage:
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/scratch-pad/probe-span-narrower-than-name.wat

(:wat::rete::defrule :spn::span-disagrees-with-name
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         ;; a keyword cannot straddle a line, so a single-line span is the only comparable case
         (:wat::rete::where (:wat::rete::i64::= ?l ?el))
         ;; ★ THE DISAGREEMENT: the span is not as wide as the name it claims to cover.
         (:wat::rete::where (:wat::rete::i64::not=
                              (:wat::rete::i64::- ?ec ?c :undefined 0)
                              (:wat::rete::string::length ?n)))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "span-disagrees-with-name"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name"  :value ?n)
                       (:wat::grep::Capture :name "width" :value (:wat::rete::i64::to-string
                                                                  (:wat::rete::i64::- ?ec ?c :undefined 0)))))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :spn))
