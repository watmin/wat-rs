;; bare-variant-constructors.wat — THE MIGRATION CENSUS, ASKED STRUCTURALLY.
;;
;; `:wat::core::{Some,Ok,Err}` are the BARE aliases of `Option::Some` / `Result::{Ok,Err}`. They
;; produce byte-identical values (measured: both spellings render `#wat.core.Option/Some [42]`),
;; but only the qualified path is a DECLARATION — the bare ones are special-cased by string
;; equality in the checker and runtime, which is why rete's constructor door cannot see them and
;; a `:then` refuses them. `296/DESIGN-STONE-H` carries the migration.
;;
;; A text census of these names counts 6346 in `.wat`. That number is not the migration's size,
;; because it cannot separate:
;;
;;     (:wat::core::Some x)            a CONSTRUCTOR CALL     — the codemod must rewrite it
;;     [x <- (:wat::core::Option …)]   a TYPE reference       — different rewrite
;;     ;; returns :wat::core::None     a COMMENT              — must NOT be rewritten
;;     ":wat::core::Some"              a STRING               — must NOT be rewritten
;;
;; This asks only for the first: a keyword in HEAD position bearing one of the three names. That
;; is the population a `wat-fix` rewrite actually has to move, and it is the number worth having
;; before anyone estimates the work.
;;
;; ⚠ `None` is deliberately absent. It is a UNIT variant — it appears as a bare keyword operand,
;; never in head position — so it is a different structural question and gets its own rule when
;; the migration is drawn. Counting it here would silently merge two populations, which is the
;; exact defect that made four censuses wrong in one day this session.

(:wat::core::defrecord :bv::Head [id <- :wat::core::i64  name <- :wat::core::String])

(:wat::rete::defrule :bv::head
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind) (?i <- :index))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::i64::= ?i 0))]
  :then [(:bv::Head :id ?id :name ?n)])

(:wat::rete::defrule :bv::some
  :when [(:bv::Head (?id <- :id) (?n <- :name))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::core::string::= ?n ":wat::core::Some"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "bare-variant-constructor"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "bare"      :value ?n)
                       (:wat::grep::Capture :name "qualified" :value ":wat::core::Option::Some")))])

(:wat::rete::defrule :bv::ok
  :when [(:bv::Head (?id <- :id) (?n <- :name))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::core::string::= ?n ":wat::core::Ok"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "bare-variant-constructor"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "bare"      :value ?n)
                       (:wat::grep::Capture :name "qualified" :value ":wat::core::Result::Ok")))])

(:wat::rete::defrule :bv::err
  :when [(:bv::Head (?id <- :id) (?n <- :name))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::core::string::= ?n ":wat::core::Err"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "bare-variant-constructor"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "bare"      :value ?n)
                       (:wat::grep::Capture :name "qualified" :value ":wat::core::Result::Err")))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector :- [:wat::rete::Rule]
    (:bv::head) (:bv::some) (:bv::ok) (:bv::err)))
