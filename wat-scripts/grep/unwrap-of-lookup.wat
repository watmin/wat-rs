;; unwrap-of-lookup.wat — A SHAPE, NOT A STRING.
;;
;; The pattern: `(Option/expect (HashMap/get m k) "msg")` — an unwrap whose argument is itself a
;; map lookup. This is the census I ran BY HAND earlier today across two files, and the reason it
;; matters is that it is the shape that panics: `HashMap/get` returns None for a key that is not
;; there, and `Option/expect` turns that None into a raise. Every instance is a place where a
;; missing key becomes a crash.
;;
;; ⛔ THERE IS NO TEXT THAT EXPRESSES THIS. The two verbs sit on different lines as often as not,
;; with the argument wrapped and indented between them; a regex over `Option/expect.*HashMap/get`
;; misses every multi-line instance and matches any two unrelated calls that share a line. The
;; pattern is not adjacency — it is PARENTAGE, three levels of it:
;;
;;     outer LIST              parent of everything below
;;       ├─ index 0  keyword   :wat::core::Option/expect      <- the unwrap
;;       └─ index 1  LIST      the argument
;;            └─ index 0  keyword  :wat::core::HashMap/get    <- the lookup
;;
;; Every line of that diagram is one join on `:parent` and `:index`. The fact base already holds
;; both; nothing here is a new capability, only a question finally asked in the right language.

(:wat::core::defrecord :ul::Unwrap    [id <- :wat::core::i64  parent <- :wat::core::i64])
(:wat::core::defrecord :ul::ArgIsList [outer <- :wat::core::i64  arg <- :wat::core::i64])

;; the unwrap head — a keyword in head position naming Option/expect
(:wat::rete::defrule :ul::unwrap
  :when [(:wat::grep::Node  (?id <- :id) (?p <- :parent) (?i <- :index) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::core::string::= ?n ":wat::core::Option/expect"))]
  :then [(:ul::Unwrap :id ?id :parent ?p)])

;; that unwrap's FIRST ARGUMENT, when the argument is itself a form
(:wat::rete::defrule :ul::arg
  :when [(:ul::Unwrap (?outer <- :parent))
         (:wat::grep::Node (?arg <- :id) (?outer <- :parent) (?ai <- :index) (?ak <- :kind))
         (:wat::rete::where (:wat::rete::core::i64::= ?ai 1))
         (:wat::rete::where (:wat::rete::core::string::= ?ak "list"))]
  :then [(:ul::ArgIsList :outer ?outer :arg ?arg)])

;; ...and that argument's own head is the lookup. Report at the OUTER form's span, because the
;; whole expression is the thing a reader wants to see, not one of its two verbs.
(:wat::rete::defrule :ul::match
  :when [(:ul::ArgIsList (?outer <- :outer) (?arg <- :arg))
         (:wat::grep::Node  (?h <- :id) (?arg <- :parent) (?hi <- :index))
         (:wat::grep::Named (?h <- :id) (?hn <- :name))
         (:wat::grep::Span  (?outer <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::core::i64::= ?hi 0))
         (:wat::rete::where (:wat::rete::core::string::= ?hn ":wat::core::HashMap/get"))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "unwrap-of-a-map-lookup"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "inner" :value ?hn)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector :- [:wat::rete::Rule] (:ul::unwrap) (:ul::arg) (:ul::match)))
