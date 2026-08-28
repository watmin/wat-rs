;; core-numerics-ops.wat — THE NUMERICS MIGRATION CENSUS, ASKED STRUCTURALLY.
;;
;; Arc 255: `:wat::core::i64::*` and `:wat::core::f64::*` (the per-type OPERATIONS) move to
;; `:wat::i64::*` / `:wat::f64::*`, adjacent to `:wat::string::`. The TYPE — `:wat::core::i64`
;; with NO trailing `::` — does not move here; it is arc 251's `wat.type/`.
;;
;; A text census cannot hold that line safely. `grep -oF ':wat::core::i64'` returns 8111 in
;; `.wat`, because it matches the TYPE and the prefix of every OP alike — one number covering
;; two populations that migrate to two different namespaces in two different arcs. Adding the
;; trailing `::` separates them, but a text tool still cannot separate:
;;
;;     (:wat::core::i64::+ a b)        a CALL          — the codemod must rewrite it
;;     ";; see :wat::core::i64::mod"   a COMMENT       — must NOT be rewritten
;;     ":wat::core::i64::+"            a STRING        — must NOT be rewritten, and splicing
;;                                                       into its span corrupts the literal,
;;                                                       because the span covers the quotes
;;                                                       and the `name` does not
;;
;; This asks for keyword LEAVES only — the population wat-fix actually rewrites. It is the
;; same discrimination `rename-core-string-to-string.wat` makes, asked as a question instead
;; of performed as an edit, so the number can be had BEFORE the migration rather than trusted
;; after it.

(:wat::rete::defrule :nm::i64-op
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::i64::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-i64-op"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :nm::f64-op
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::f64::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-f64-op"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :nm))
