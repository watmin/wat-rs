;; probe-grep-driver.wat — the fixture for BRIEF-STONE-the-grep-driver's acceptance rows.
;;
;; Declares two rules and calls `:wat::grep::run` directly with them (part B, the `--grep` CLI
;; mode, later replaces "the probe calls run" with "the CLI looks up `:user::grep` and calls
;; run" — nothing else about this file's rules changes).
;;
;; Rule 1 — `:pg::arrow` — fires on the `<-` binder symbol, the exact shape
;; `rules-corpus-03-source-to-facts.wat`'s `:fx::match-arrow` proved. A file containing at least
;; one `<-` (i.e. any real wat function/rule with a binder) matches; a file with none does not —
;; that asymmetry is what row 3 (facts do not leak between files) needs.
(:wat::core::defrecord :pg::IsArrow [id <- :wat::core::i64])

(:wat::rete::defrule :pg::arrow
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::core::string::= ?n "<-"))]
  :then [(:pg::IsArrow :id ?id)])

(:wat::rete::defrule :pg::match-arrow
  :when [(:wat::grep::Node (?id <- :id) (?k <- :kind))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:pg::IsArrow (?id <- :id))]
  :then [(:wat::grep::Match
           :file     "probe"
           :line     ?l
           :col      ?c
           :end-line ?el
           :end-col  ?ec
           :rule     "pg::match-arrow"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "kind" :value ?k)))])

(:wat::core::defn :user::the-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector :- [:wat::rete::Rule] (:pg::arrow) (:pg::match-arrow)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::grep::run (:user::the-rules)))
