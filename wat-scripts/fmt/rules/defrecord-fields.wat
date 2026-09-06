;; Field VECTOR of defrecord / defstruct / defenum. Claims the vector;
;; one field per line after the first; AlignStride 3 pads names so `<-`
;; lines up. The parent rule does not reach in here.

(:wat::rete::defrule :fmt::defrecord-fields-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defrecord"))
         (:wat::grep::Node  (?args <- :id) (?p <- :parent) (?vi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?args <- :parent) (?ai <- :index))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "<-"))
         (:wat::rete::where (:wat::rete::i64::= ?ai 1))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?vi 1 :undefined 0)))))]
  :then [(:wat::fmt::Claim :form ?args)
         (:wat::fmt::AlignStride :form ?args :stride 3)])

(:wat::rete::defrule :fmt::defstruct-fields-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defstruct"))
         (:wat::grep::Node  (?args <- :id) (?p <- :parent) (?vi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?args <- :parent) (?ai <- :index))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "<-"))
         (:wat::rete::where (:wat::rete::i64::= ?ai 1))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?vi 1 :undefined 0)))))]
  :then [(:wat::fmt::Claim :form ?args)
         (:wat::fmt::AlignStride :form ?args :stride 3)])

(:wat::rete::defrule :fmt::defenum-fields-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defenum"))
         (:wat::grep::Node  (?args <- :id) (?p <- :parent) (?vi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?args <- :parent) (?ai <- :index))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "<-"))
         (:wat::rete::where (:wat::rete::i64::= ?ai 1))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?vi 1 :undefined 0)))))]
  :then [(:wat::fmt::Claim :form ?args)
         (:wat::fmt::AlignStride :form ?args :stride 3)])

(:wat::rete::defrule :fmt::defrecord-field-per-line
  :when [(:wat::fmt::Claim (?args <- :form))
         (:wat::fmt::AlignStride (?args <- :form) (?s <- :stride))
         (:wat::rete::where (:wat::rete::i64::= ?s 3))
         (:wat::grep::Node  (?ch <- :id) (?args <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?ci 3 :undefined 1) 0))]
  :then [(:wat::fmt::Break :id ?ch :kind (:wat::fmt::BreakKind::Align))])
