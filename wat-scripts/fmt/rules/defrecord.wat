;; defrecord / defstruct / defenum — NAME rides the head line; field
;; vectors (not the `:-` type-args vector) start their own line.
;; Bare enum variants are not broken. Field internals: defrecord-fields.wat.

(:wat::load-file! "defrecord-fields.wat")

(:wat::rete::defrule :fmt::defrecord-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defrecord"))]
  :then [(:wat::fmt::Claim :form ?p)])

(:wat::rete::defrule :fmt::defstruct-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defstruct"))]
  :then [(:wat::fmt::Claim :form ?p)])

(:wat::rete::defrule :fmt::defenum-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defenum"))]
  :then [(:wat::fmt::Claim :form ?p)])

(:wat::rete::defrule :fmt::defrecord-vec-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defrecord"))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?vk "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?v <- :parent) (?ai <- :index))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "<-"))
         (:wat::rete::where (:wat::rete::i64::= ?ai 1))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?vi 1 :undefined 0)))))]
  :then [(:wat::fmt::Break :id ?v :kind "block")])

(:wat::rete::defrule :fmt::defstruct-vec-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defstruct"))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?vk "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?v <- :parent) (?ai <- :index))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "<-"))
         (:wat::rete::where (:wat::rete::i64::= ?ai 1))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?vi 1 :undefined 0)))))]
  :then [(:wat::fmt::Break :id ?v :kind "block")])

(:wat::rete::defrule :fmt::defenum-vec-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defenum"))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?vk "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?v <- :parent) (?ai <- :index))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "<-"))
         (:wat::rete::where (:wat::rete::i64::= ?ai 1))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?vi 1 :undefined 0)))))]
  :then [(:wat::fmt::Break :id ?v :kind "block")])

;; A variant NAME whose next sibling is a vector starts a line.
;; Bare variants (next is not a vector) stay un-broken. `:-` is not a variant.
(:wat::rete::defrule :fmt::defenum-tag-before-fields
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defenum"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?ck "keyword"))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::grep::Named (?c <- :id) (?cn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?cn ":-"))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?vk "vector"))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?ci 1 :undefined 0)))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])
