;; wat-scripts/fixes/to-faithful-clojure-net.wat — the faithful-Clojure conversion as a
;; FORWARD-CHAINING RETE NETWORK (PORTA PORTAM APERIT). Recognition is decomposed into many
;; trivial single-condition activation gates; each :then inserts one intermediate fact that
;; unlocks the next rule. The walk is PURE OBSERVATION (emits :fix::Node facts only); ALL
;; classification lives in rete. rete stays pure — rules DEDUCE, the drive queries out + actions.
;;
;; Identity = the node's char OFFSET (unique per node). parent-id = the enclosing list's offset.
;; The synthesized-head skip is EMERGENT: a desugared sigil (span-len != name-len) never becomes
;; :fix::Genuine, so it can never reach HeadConv/TypeConv. The absence of an activation IS the skip.
;;
;; Driven by the native kernel :wat::rete::fire-rules' (arc 278; the O(N²) oracle fire-rules is the
;; reference spec, not for the corpus). Gate = round-trip parse of the output (NOT byte-match fix-text).

;; ══ LAYER 0 · OBSERVATION FACT ══════════════════════════════════════════════
;; a node's enclosure — a sum type, matched (NOT a magic sentinel). Root = top-level form.
(:wat::core::defenum :fix::Parent :wat::enum::Pure
  :Root
  :Enclosed [id <- :wat::core::i64  head <- :wat::core::String])

(:wat::core::defrecord :fix::Node
  [kind      <- :wat::core::String
   name      <- :wat::core::String
   offset    <- :wat::core::i64
   len       <- :wat::core::i64
   span-len  <- :wat::core::i64
   parent    <- :fix::Parent
   child-idx <- :wat::core::i64])

;; ══ INTERMEDIATE (activation-gate) FACTS — carry only the offset identity ════
(:wat::core::defrecord :fix::Keyword       [offset <- :wat::core::i64])
(:wat::core::defrecord :fix::Symbol        [offset <- :wat::core::i64])
(:wat::core::defrecord :fix::Genuine       [offset <- :wat::core::i64])
(:wat::core::defrecord :fix::Namespaced    [offset <- :wat::core::i64])
(:wat::core::defrecord :fix::TypeShaped    [offset <- :wat::core::i64])
(:wat::core::defrecord :fix::TypeCandidate [offset <- :wat::core::i64])
(:wat::core::defrecord :fix::Arrow         [offset <- :wat::core::i64])
(:wat::core::defrecord :fix::PostArrow     [offset <- :wat::core::i64])

;; ══ TERMINAL FACTS — carry the edit payload (the drive queries these) ════════
(:wat::core::defrecord :fix::HeadConv  [offset <- :wat::core::i64  len <- :wat::core::i64  name <- :wat::core::String])
(:wat::core::defrecord :fix::TypeConv  [offset <- :wat::core::i64  len <- :wat::core::i64  name <- :wat::core::String])
(:wat::core::defrecord :fix::ArrowConv [offset <- :wat::core::i64  len <- :wat::core::i64])

;; ══ PURE STRING PREDICATES (used in :where guards) ══════════════════════════
(:wat::core::defn :fix::has-ns? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::string::contains? name "::"))

(:wat::core::defn :fix::type-shaped? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::if (:wat::core::string::contains? name "<")
                    (:wat::core::string::contains? name ">")
                    false)
    true
    (:wat::core::if (:wat::core::string::contains? name "(")
      (:wat::core::string::contains? name ")")
      false)))

;; ══ LAYER 1 · TOKEN TYPING ══════════════════════════════════════════════════
;; G1 keyword?
(:wat::rete::defrule :fix::g1-keyword
  :when [(:fix::Node (?off <- :offset) (?kind <- :kind) (:wat::rete::core::string::= ?kind "keyword"))]
  :then [(:fix::Keyword ?off)])

;; G2 symbol?
(:wat::rete::defrule :fix::g2-symbol
  :when [(:fix::Node (?off <- :offset) (?kind <- :kind) (:wat::rete::core::string::= ?kind "symbol"))]
  :then [(:fix::Symbol ?off)])

;; G3 genuine?  — span source-len == name-len (a desugared sigil never passes; THE SKIP)
(:wat::rete::defrule :fix::g3-genuine
  :when [(:fix::Keyword (?off <- :offset))
         (:fix::Node (?off <- :offset) (?len <- :len) (?slen <- :span-len))
         (:wat::rete::where (:wat::rete::core::string::= ?slen ?len))]
  :then [(:fix::Genuine ?off)])

;; ══ LAYER 2 · LEXICAL SHAPE (only genuine keywords) ═════════════════════════
;; G4 namespaced?
(:wat::rete::defrule :fix::g4-namespaced
  :when [(:fix::Genuine (?off <- :offset))
         (:fix::Node (?off <- :offset) (?name <- :name))
         (:wat::rete::where (:fix::has-ns? ?name))]
  :then [(:fix::Namespaced ?off)])

;; G5 type-shaped?
(:wat::rete::defrule :fix::g5-type-shaped
  :when [(:fix::Genuine (?off <- :offset))
         (:fix::Node (?off <- :offset) (?name <- :name))
         (:wat::rete::where (:fix::type-shaped? ?name))]
  :then [(:fix::TypeShaped ?off)])

;; ══ LAYER 3 · POSITION (joins) ══════════════════════════════════════════════
;; G6 arrow?  — a bare <- / -> symbol
(:wat::rete::defrule :fix::g6-arrow
  :when [(:fix::Symbol (?off <- :offset))
         (:fix::Node (?off <- :offset) (?name <- :name))
         (:wat::rete::where (:wat::rete::core::or (:wat::rete::core::string::= ?name "<-") (:wat::rete::core::string::= ?name "->")))]
  :then [(:fix::Arrow ?off)])

;; G7 post-arrow?  — the node one child-index after an arrow, same parent (SELF-JOIN)
(:wat::rete::defrule :fix::g7-post-arrow
  :when [(:fix::Arrow (?aoff <- :offset))
         (:fix::Node (?aoff <- :offset) (?p <- :parent) (?ai <- :child-idx))
         (:fix::Node (?boff <- :offset) (?p <- :parent) (?bi <- :child-idx))
         (:wat::rete::where (:wat::rete::core::string::= ?bi (:wat::core::+ ?ai 1)))]
  :then [(:fix::PostArrow ?boff)])

;; TypeCandidate ← type-shaped OR post-arrow (the ∪, as two trivial gates)
(:wat::rete::defrule :fix::tc-from-shaped
  :when [(:fix::TypeShaped (?off <- :offset))]
  :then [(:fix::TypeCandidate ?off)])

(:wat::rete::defrule :fix::tc-from-postarrow
  :when [(:fix::PostArrow (?off <- :offset))
         (:fix::Genuine (?off <- :offset))]
  :then [(:fix::TypeCandidate ?off)])

;; ══ LAYER 4 · TERMINAL CLASSIFICATION ═══════════════════════════════════════
;; T1 HeadConv ← Namespaced ∩ ¬TypeShaped ∩ ¬PostArrow
(:wat::rete::defrule :fix::t1-head-conv
  :when [(:fix::Namespaced (?off <- :offset))
         (:fix::Node (?off <- :offset) (?len <- :len) (?name <- :name))
         (:wat::rete::not (:fix::TypeShaped (?off <- :offset)))
         (:wat::rete::not (:fix::PostArrow (?off <- :offset)))]
  :then [(:fix::HeadConv ?off ?len ?name)])

;; T2 TypeConv ← TypeCandidate (∩ ¬IfType — added in Stage B)
(:wat::rete::defrule :fix::t2-type-conv
  :when [(:fix::TypeCandidate (?off <- :offset))
         (:fix::Node (?off <- :offset) (?len <- :len) (?name <- :name))]
  :then [(:fix::TypeConv ?off ?len ?name)])

;; T3 ArrowConv ← Arrow (∩ ¬IfArrow — added in Stage B)
(:wat::rete::defrule :fix::t3-arrow-conv
  :when [(:fix::Arrow (?off <- :offset))
         (:fix::Node (?off <- :offset) (?len <- :len))]
  :then [(:fix::ArrowConv ?off ?len)])

(:wat::rete::defquery :fix::q-HeadConv
  :params []
  :when [(:fix::HeadConv (?offset <- :offset) (?len <- :len) (?name <- :name))])

(:wat::rete::defquery :fix::q-ArrowConv
  :params []
  :when [(:fix::ArrowConv (?offset <- :offset) (?len <- :len))])

(:wat::rete::defquery :fix::q-TypeConv
  :params []
  :when [(:fix::TypeConv (?offset <- :offset) (?len <- :len) (?name <- :name))])


;; ══ THE OBSERVATION WALK — emit :fix::Node facts (pure; zero classification) ══
(:wat::core::defn :fix::walk-seq
  [items  <- (:wat::core::Vector :- [:wat::WatAST])
   parent <- :fix::Parent
   idx    <- :wat::core::i64
   lines  <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:fix::Node])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :fix::Node)
    (:wat::core::concat
      (:fix::walk-node (:wat::core::first items) parent idx lines)
      (:fix::walk-seq (:wat::core::rest items) parent (:wat::core::+ idx 1) lines))))

(:wat::core::defn :fix::walk-node
  [node      <- :wat::WatAST
   parent    <- :fix::Parent
   child-idx <- :wat::core::i64
   lines     <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:fix::Node])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [my-id (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                      ch    (:wat::core::ast->children node)
                      my-head (:wat::core::if (:wat::core::empty? ch)
                                ""
                                (:wat::core::let [h (:wat::core::first ch)]
                                  (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
                                    (:wat::core::ast-name h)
                                    "")))]
      (:fix::walk-seq ch (:fix::Parent::Enclosed my-id my-head) 0 lines))
    (:wat::core::let [kind (:wat::core::ast-kind node)]
      (:wat::core::if (:wat::core::or (:wat::core::= kind "keyword") (:wat::core::= kind "symbol"))
        (:wat::core::let [name (:wat::core::ast-name node)
                          off  (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                          len  (:wat::core::string::length name)
                          slen (:wat::fix::fix-text-span-len
                                 (:wat::core::ast-span node)
                                 (:wat::core::ast-end-span node)
                                 lines)]
          (:wat::core::Vector :fix::Node
            (:fix::Node :kind kind :name name :offset off :len len :span-len slen :parent parent :child-idx child-idx)))
        (:wat::core::Vector :fix::Node)))))

;; stage the facts: fold insert over the Node vector
(:wat::core::defn :fix::insert-nodes
  [session <- :wat::rete::Session  nodes <- (:wat::core::Vector :- [:fix::Node])]
  -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  n <- :fix::Node] -> :wat::rete::Session
      (:wat::rete::insert s n))
    session nodes))

;; ══ QUERY OUT + ACTION — the transform lives HERE (outside rete) ═════════════
(:wat::core::defn :fix::head-edits
  [convs <- :wat::core::PersistentVector
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                     hc <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat a
        (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
          (:wat::core::Tuple
            (:wat::core::Option/expect (:wat::core::PersistentMap/get hc "?offset") "q-HeadConv: ?offset")
            (:wat::core::Option/expect (:wat::core::PersistentMap/get hc "?len") "q-HeadConv: ?len")
            (:wat::core::ast-name (:wat::core::keyword/to-symbol
              (:wat::core::keyword-node
                (:wat::core::Option/expect
                  (:wat::core::PersistentMap/get hc "?name")
                  "q-HeadConv: ?name"))))))))
    acc convs))

(:wat::core::defn :fix::arrow-edits
  [convs <- :wat::core::PersistentVector
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                     ac <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat a
        (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
          (:wat::core::Tuple
            (:wat::core::Option/expect (:wat::core::PersistentMap/get ac "?offset") "q-ArrowConv: ?offset")
            (:wat::core::Option/expect (:wat::core::PersistentMap/get ac "?len") "q-ArrowConv: ?len")
            ":-"))))
    acc convs))

(:wat::core::defn :fix::type-edits
  [convs <- :wat::core::PersistentVector
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                     tc <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat a
        (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
          (:wat::core::Tuple
            (:wat::core::Option/expect (:wat::core::PersistentMap/get tc "?offset") "q-TypeConv: ?offset")
            (:wat::core::Option/expect (:wat::core::PersistentMap/get tc "?len") "q-TypeConv: ?len")
            (:wat::core::write-forms (:wat::core::keyword/to-type-form
              (:wat::core::keyword-node
                (:wat::core::Option/expect
                  (:wat::core::PersistentMap/get tc "?name")
                  "q-TypeConv: ?name"))))))))
    acc convs))

;; ══ CONVERT — walk → fire the network → query out → edit → batch-apply ══════
(:wat::core::defn :fix::convert
  [src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines   (:wat::core::string::split src "\n")
                    tree    (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms   (:wat::core::ast->children tree)
                    ;; top-level forms have no enclosing list → :fix::Parent::Root (no sentinel).
                    nodes   (:fix::walk-seq forms :fix::Parent::Root 0 lines)
                    rules   (:wat::rete::collect-rules :fix)
                    session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
                    staged  (:fix::insert-nodes session nodes)
                    fired   (:wat::rete::fire-fixpoint staged)
                    empty-e (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
                    e1      (:fix::head-edits  (:wat::rete::query fired (:fix::q-HeadConv))  empty-e)
                    e2      (:fix::arrow-edits (:wat::rete::query fired (:fix::q-ArrowConv)) e1)
                    e3      (:fix::type-edits  (:wat::rete::query fired (:fix::q-TypeConv))  e2)
                    sorted  (:wat::core::sort
                              (:wat::core::fn [a <- :(wat::core::i64,wat::core::i64,wat::core::String)
                                               b <- :(wat::core::i64,wat::core::i64,wat::core::String)]
                                -> :wat::core::bool
                                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
                              e3)]
    (:wat::fix::fix-text-apply src sorted)))

;; ══ DRIVE — read → convert → write, per path ════════════════════════════════
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:fix::convert (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[to-faithful-clojure-net] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
