;; wat-scripts/fixes/locus-names-its-transport.wat — arc 255 Stone 255.18 (C-b1a),
;; "the locus names its transport".
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; `:wat::spawn::Locus` became the parametric surface `(Locus :- [T])`: ThreadOpts binds
;; `Shared`, ProcessOpts binds `Wire`, and `Locus/launch` returns the 5-argument
;; `(Launched :- [S R Sh Lu T])`. A CONSUMER that holds a locus of either transport says so
;; by declaring `T` in its binder and taking `(Locus :- [T])`.
;;
;; THE RULE — a `:wat::core::defn` whose params vector holds a param typed EXACTLY the bare
;; keyword `:wat::spawn::Locus` (the token right after a `<-`):
;;   1. binder: none → insert ` :- [T]` after the name; present → append ` T` inside it
;;      (a binder that already declares `T` is REFUSED — raise, never guess a letter);
;;   2. the param type `:wat::spawn::Locus` → `(:wat::spawn::Locus :- [T])`;
;;   3. a return type `(:wat::spawn::Launched :- [a b c d])` of arity 4 → append ` T`.
;; Spec rows (the ORACLE quotes these):
;;   (defn f [l <- :wat::spawn::Locus] -> R)  =>  (defn f :- [T] [l <- (:wat::spawn::Locus :- [T])] -> R)
;;   (defn f :- [A] [l <- :wat::spawn::Locus] -> R)  =>  (defn f :- [A T] [l <- (:wat::spawn::Locus :- [T])] -> R)
;;   -> (:wat::spawn::Launched :- [a b c d])  =>  -> (:wat::spawn::Launched :- [a b c d T])
;; Idempotent by construction: after the rewrite the param type is a LIST, not the bare
;; keyword, so rule 1 no longer matches and neither do 2 or 3.
;;
;; ⚠ SCOPE IS THE PATH LIST, NOT THE RULE. A generic `(Locus :- [T])` cannot today narrow to a
;; defclause keyed on the CONCRETE loci (`:wat::spawn::runner-count`, `:wat::spawn::with-label`)
;; — measured in wat-scripts/scratch-pad/255-18-generic-locus-narrows-to-a-clause.wat. A defn
;; that calls one of those (e.g. `:wat::bracket::map-worker`, `:user::read-blind`) must NOT be
;; listed. Pass only the consumers the stone names.
;; 255.19 lifted that restriction: both defclauses became `Locus` surface methods
;; (wat-scripts/fixes/locus-methods-on-the-waist.wat), and this codemod was then run on
;; `wat-scripts/probes/arc-170/probe-s2-runner-count.wat` (`:user::read-blind`).
;;
;; Comment-faithful: span splices through `fix-text-apply` (each edit carries its old text).
;;
;; Usage:
;;   printf '[…EVERY path…]\n' | cargo wat ./wat-scripts/fixes/locus-names-its-transport.wat

(:wat::core::defn :user::no-edits [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :user::one-edit
  [off <- :wat::core::i64 old <- :wat::core::String new <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple off old new)))

;; leaf-named? — a symbol/keyword leaf whose ast-name is exactly `nm`.
(:wat::core::defn :user::leaf-named?
  [node <- :wat::WatAST nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::fix::structural? node)
    false
    (:wat::core::let [k (:wat::core::ast-kind node)]
      (:wat::core::if (:wat::core::if (:wat::core::= k "symbol") true (:wat::core::= k "keyword"))
        (:wat::core::= (:wat::core::ast-name node) nm)
        false))))

;; locus-param-edits — walk a params vector's children pairwise; for each `<-` followed by the
;; bare `:wat::spawn::Locus` keyword, one replace edit on that keyword.
(:wat::core::defn :user::locus-param-edits
  [ps    <- (:wat::core::Vector :- [:wat::WatAST])
   prev-arrow? <- :wat::core::bool
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? ps)
    (:user::no-edits)
    (:wat::core::let [h  (:wat::core::first ps)
                      tl (:wat::core::rest ps)
                      here (:wat::core::if prev-arrow?
                             (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
                               (:wat::core::if (:wat::core::= (:wat::core::ast-name h) ":wat::spawn::Locus")
                                 (:wat::core::if (:wat::fix::source-matches-name? h lines src)
                                   (:user::one-edit (:wat::fix::node-start-offset h lines)
                                     ":wat::spawn::Locus" "(:wat::spawn::Locus :- [T])")
                                   (:user::no-edits))
                                 (:user::no-edits))
                               (:user::no-edits))
                             (:user::no-edits))]
      (:wat::core::concat here
        (:user::locus-param-edits tl (:user::leaf-named? h "<-") lines src)))))

;; launched-4? — `(:wat::spawn::Launched :- [a b c d])`: a list, head the Launched keyword,
;; child[1] the `:-` operator, child[2] a 4-element vector.
(:wat::core::defn :user::launched-4? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
        (:wat::core::if (:user::leaf-named? (:wat::core::first ch) ":wat::spawn::Launched")
          (:wat::core::let [v (:wat::core::nth ch 2)]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind v) "vector")
              (:wat::core::= (:wat::core::length (:wat::core::ast->children v)) 4)
              false))
          false)
        false))
    false))

;; binder-has-t? — does a binder vector already declare a `T`?
(:wat::core::defn :user::binder-has-t?
  [b <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool n <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true (:user::leaf-named? n "T")))
    false
    (:wat::core::ast->children b)))

;; close-bracket-edit — insert ` T` just before a vector node's closing `]`.
(:wat::core::defn :user::close-bracket-edit
  [v <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:user::one-edit (:wat::core::- (:wat::fix::node-end-offset v lines) 1) "]" " T]"))

;; defn-edits — the rule, on one `(:wat::core::defn …)` list.
(:wat::core::defn :user::defn-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [ch      (:wat::core::ast->children node)
     n       (:wat::core::length ch)
     binder? (:wat::core::if (:wat::core::> n 3) (:user::leaf-named? (:wat::core::nth ch 2) ":-") false)
     pi      (:wat::core::if binder? 4 2)
     ok?     (:wat::core::> n (:wat::core::+ pi 2))]
    (:wat::core::if ok?
      (:wat::core::let
        [name   (:wat::core::nth ch 1)
         params (:wat::core::nth ch pi)
         ret    (:wat::core::nth ch (:wat::core::+ pi 2))
         p-edits (:wat::core::if (:wat::core::= (:wat::core::ast-kind params) "vector")
                   (:user::locus-param-edits (:wat::core::ast->children params) false lines src)
                   (:user::no-edits))]
        (:wat::core::if (:wat::core::empty? p-edits)
          (:user::no-edits)
          (:wat::core::let
            [b-edits (:wat::core::if binder?
                       (:wat::core::if (:user::binder-has-t? (:wat::core::nth ch 3))
                         (:wat::kernel::assertion-failed!
                           :message (:wat::string::concat
                                      "locus-names-its-transport: binder already declares T in "
                                      (:wat::core::ast-name name)))
                         (:user::close-bracket-edit (:wat::core::nth ch 3) lines))
                       (:user::one-edit (:wat::fix::node-end-offset name lines) "" " :- [T]"))
             r-edits (:wat::core::if (:user::launched-4? ret)
                       (:user::close-bracket-edit (:wat::core::nth (:wat::core::ast->children ret) 2) lines)
                       (:user::no-edits))]
            (:wat::core::concat b-edits (:wat::core::concat p-edits r-edits)))))
      (:user::no-edits))))

;; walk — every list whose head is the `:wat::core::defn` keyword gets `defn-edits`; every
;; other structural node recurses into its children. Edits come out in ascending order.
(:wat::core::defn :user::walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:user::no-edits)
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)
                      here (:wat::core::if (:wat::fix::calls-to? h ":wat::core::defn")
                             (:user::defn-edits h lines src)
                             (:wat::core::if (:wat::fix::structural? h)
                               (:user::walk (:wat::core::ast->children h) lines src)
                               (:user::no-edits)))]
      (:wat::core::concat here (:user::walk tl lines src)))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::walk forms lines src)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
