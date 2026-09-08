;; defrecord / defstruct / defenum — NAME rides the head line.
;; defrecord/defstruct: field vector on its own line.
;; defenum: each variant TAG starts a line; its vector rides the tag;
;; a bare tag gets ` []` written after it. Field internals: defrecord-fields.wat.

(:wat::load-file! "defrecord-fields.wat")

(:wat::rete::defrule :fmt::defrecord-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defrecord"))]
  :then [(:wat::fmt::Claim :form ?p)])

(:wat::rete::defrule :fmt::defstruct-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defstruct"))]
  :then [(:wat::fmt::Claim :form ?p)])

(:wat::rete::defrule :fmt::defenum-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defenum"))]
  :then [(:wat::fmt::Claim :form ?p)
         (:wat::fmt::AlignPairs :form ?p)])

(:wat::rete::defrule :fmt::defrecord-vec-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defrecord"))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
         (:wat::rete::where (:wat::rete::core::enum::= ?vk (:wat::grep::NodeKind::Vector {})))
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
  :then [(:wat::fmt::Break :id ?v :kind (:wat::fmt::BreakKind::Block {}))])

(:wat::rete::defrule :fmt::defstruct-vec-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defstruct"))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
         (:wat::rete::where (:wat::rete::core::enum::= ?vk (:wat::grep::NodeKind::Vector {})))
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
  :then [(:wat::fmt::Break :id ?v :kind (:wat::fmt::BreakKind::Block {}))])

;; Every variant TAG starts a line. Name, `:-`, purity stay on the head
;; line. The field vector rides the tag (no Break on the vector).
(:wat::rete::defrule :fmt::defenum-tag-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defenum"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::core::enum::= ?ck (:wat::grep::NodeKind::Keyword {})))
         (:wat::rete::where (:wat::rete::i64::> ?ci 1))
         (:wat::grep::Named (?c <- :id) (?cn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?cn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?cn "wat.enum/Pure"))
         (:wat::rete::where (:wat::rete::string::not= ?cn "wat.enum/Impure"))]
  :then [(:wat::fmt::Break :id ?c :kind (:wat::fmt::BreakKind::Block {}))])

;; A tag with no vector sibling gets ` []` written after it. A tag that
;; already has a vector is left alone — pass 2 must not insert again.
(:wat::rete::defrule :fmt::defenum-empty-vec
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defenum"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::core::enum::= ?ck (:wat::grep::NodeKind::Keyword {})))
         (:wat::rete::where (:wat::rete::i64::> ?ci 1))
         (:wat::grep::Named (?c <- :id) (?cn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?cn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?cn "wat.enum/Pure"))
         (:wat::rete::where (:wat::rete::string::not= ?cn "wat.enum/Impure"))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
             (:wat::rete::where (:wat::rete::core::enum::= ?vk (:wat::grep::NodeKind::Vector {})))
             (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?ci 1 :undefined 0)))))]
  :then [(:wat::fmt::EmptyVecAfter :id ?c)])
