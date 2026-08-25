;; probe-grep-cli.wat — the fixture for BRIEF-STONE-the-grep-mode's acceptance rows.
;;
;; Declares `:user::grep` (and NO `:user::main`) — exactly the shape `wat --grep` dispatches to.
;; Rules are copied verbatim from `wat-scripts/scratch-pad/probe-grep-driver.wat` (proven, Part
;; A). That file calls `:wat::grep::run` directly from `:user::main`; this one hands the same
;; rules to `:user::grep` and lets the CLI (`--grep`) do the dispatch instead — nothing about the
;; rules themselves changes.
;;
;; Rule 1 — `:pg::arrow` — fires on the `<-` binder symbol, the exact shape
;; `rules-corpus-03-source-to-facts.wat`'s `:fx::match-arrow` proved. A file containing at least
;; one `<-` (i.e. any real wat function/rule with a binder) matches; a file with none does not —
;; that asymmetry is what row 5 (facts do not leak between files) needs.
(:wat::core::defrecord :pg::IsArrow [id <- :wat::core::i64])

(:wat::rete::defrule :pg::arrow
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::string::= ?n "<-"))]
  :then [(:pg::IsArrow :id ?id)])

(:wat::rete::defrule :pg::match-arrow
  :when [(:wat::grep::Node (?id <- :id) (?k <- :kind))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:pg::IsArrow (?id <- :id))
         ;; the ONE Source fact — this is how a rule learns which file it is matching in.
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file     ?f
           :line     ?l
           :col      ?c
           :end-line ?el
           :end-col  ?ec
           :rule     "pg::match-arrow"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "kind" :value ?k)))])

;; `:user::grep` — the mode's entry point. `--grep` validates this shape (the mirror wall) and
;; hands the result straight to `:wat::grep::run`. NO `:user::main` in this file — the direct
;; proof that the main wall (`src/distribution/mod.rs:443`) is not on the `--grep` path.
(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector :- [:wat::rete::Rule] (:pg::arrow) (:pg::match-arrow)))
