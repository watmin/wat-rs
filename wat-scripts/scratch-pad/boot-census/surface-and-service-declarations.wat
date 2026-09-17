;; surface-and-service-declarations.wat — WHICH manifest entries actually DECLARE a surface or a
;; service, and how many?
;;
;; Drawn for `docs/excursus/2026/08/001-sns-sqs/boot-names-where-its-time-goes/`, EXPECTATIONS row
;; 5: *"17 such forms exist. Is their per-line cost above average, and by how much?"* That row
;; cannot be answered from a text count, and the failure is not hypothetical — a `grep -o` for
;; `:wat::core::defsurface|:wat::core::defservice` over the 55 manifest entries returns 18
;; occurrences across 10 files, and they are NOT 18 declarations: the token also appears in prose
;; comments, in `defservice`'s own macro template inside `wat/service.wat`, and at `:impls`-style
;; mention sites. A declaration is the token in HEAD POSITION — child at index 0 of its form — and
;; that is one integer in the fact base, exactly the `head-position.wat` shape this copies.
;;
;;     printf '["wat/core.wat" …every manifest entry…]\n' | wat --grep \
;;       wat-scripts/scratch-pad/boot-census/surface-and-service-declarations.wat
;;
;; ⛔ The census's per-file table is what says where the TIME went; this says which files carry the
;; declaration. Reading one as the other is the mistake the excursus exists to avoid.

(:wat::core::defrecord :ssd::IsHead [id <- :wat::core::i64])

;; a keyword node in head position — index 0 of its parent form
(:wat::rete::defrule :ssd::head
  :when [(:wat::grep::Node (?id <- :id) (?k <- :kind) (?i <- :index))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))]
  :then [(:ssd::IsHead :id ?id)])

;; ...whose name is `defsurface`
(:wat::rete::defrule :ssd::declares-surface
  :when [(:ssd::IsHead (?id <- :id))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::grep::Span  (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defsurface"))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "declares-a-surface"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "head" :value ?n)))])

;; ...or `defservice`
(:wat::rete::defrule :ssd::declares-service
  :when [(:ssd::IsHead (?id <- :id))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::grep::Span  (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defservice"))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "declares-a-service"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "head" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :ssd))
