;; wat-scripts/fixes/node-kind-string-to-enum.wat — arc 277: Node.kind becomes an enum.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of consumers.
;;
;; Three rewrites:
;;   1. rete equality, only when the var is bound from :wat::grep::Node :kind:
;;        (:wat::rete::string::= ?ak "list")
;;          -> (:wat::rete::core::enum::= ?ak (:wat::grep::NodeKind::List))
;;   2. rete inequality, same binder restriction (kwargs' "every even child IS a
;;      keyword" guard is string::not= — leaving it as a String compare against
;;      an enum field makes the :not fire on every key and the rule never claims):
;;        (:wat::rete::string::not= ?bk "keyword")
;;          -> (:wat::rete::core::enum::not= ?bk (:wat::grep::NodeKind::Keyword))
;;   3. :wat::grep::Node constructor :kind "symbol"
;;          -> :kind (:wat::grep::NodeKind::Symbol)
;;
;; ast-kind String comparisons are NOT touched (the intrinsic stays a String).
;; :fix::Node / :user::StrNode :kind strings are NOT touched.
;;
;; Idempotent: after one run the comparison head is enum::= and the constructor
;; value is a list. A second run reports 0 changes.
;;
;; Usage:
;;   printf '["pathA" "pathB" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/node-kind-string-to-enum.wat

(:wat::core::defn :user::empty-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :user::one-edit
  [off <- :wat::core::i64  old <- :wat::core::String  new <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple off old new)))

(:wat::core::defn :user::start-off
  [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n)
    ""))

(:wat::core::defn :user::sym-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "symbol")
    (:wat::core::ast-name n)
    ""))

(:wat::core::defn :user::variant-of [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::= s "int") "IntLit")
    ((:wat::core::= s "float") "FloatLit")
    ((:wat::core::= s "rational") "RationalLit")
    ((:wat::core::= s "bigint") "BigIntLit")
    ((:wat::core::= s "char") "CharLit")
    ((:wat::core::= s "bool") "BoolLit")
    ((:wat::core::= s "string") "StringLit")
    ((:wat::core::= s "nil") "NilLit")
    ((:wat::core::= s "keyword") "Keyword")
    ((:wat::core::= s "symbol") "Symbol")
    ((:wat::core::= s "list") "List")
    ((:wat::core::= s "vector") "Vector")
    ((:wat::core::= s "set") "Set")
    ((:wat::core::= s "map") "Map")
    (:else "")))

(:wat::core::defn :user::kind-binding?
  [n <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "list")
    (:wat::core::let [ch (:wat::core::ast->children n)]
      (:wat::core::if (:wat::i64::= (:wat::core::length ch) 3)
        (:wat::core::if (:wat::core::= (:user::sym-name (:wat::core::nth ch 1)) "<-")
          (:wat::core::= (:user::kw-name (:wat::core::nth ch 2)) ":kind")
          false)
        false))
    false))

(:wat::core::defn :user::collect-kind-vars
  [node <- :wat::WatAST
   acc  <- (:wat::core::HashSet :- [:wat::core::String])]
  -> (:wat::core::HashSet :- [:wat::core::String])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        acc
        (:wat::core::let
          [here (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::grep::Node")
                   (:wat::core::foldl
                     (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::String]) c <- :wat::WatAST]
                       -> (:wat::core::HashSet :- [:wat::core::String])
                       (:wat::core::if (:user::kind-binding? c)
                         (:wat::hashset::conj s (:user::sym-name (:wat::core::first (:wat::core::ast->children c))))
                         s))
                     acc
                     ch)
                   acc)]
          (:wat::core::foldl
            (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::String]) c <- :wat::WatAST]
              -> (:wat::core::HashSet :- [:wat::core::String])
              (:user::collect-kind-vars c s))
            here
            ch))))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::String]) c <- :wat::WatAST]
          -> (:wat::core::HashSet :- [:wat::core::String])
          (:user::collect-kind-vars c s))
        acc
        (:wat::core::ast->children node))
      acc)))

(:wat::core::defn :user::enum-cmp-head [h <- :wat::core::String] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::= h ":wat::rete::string::=") ":wat::rete::core::enum::=")
    ((:wat::core::= h ":wat::rete::string::not=") ":wat::rete::core::enum::not=")
    (:else "")))

(:wat::core::defn :user::string-eq-edit
  [node <- :wat::WatAST
   vars <- (:wat::core::HashSet :- [:wat::core::String])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::i64::= (:wat::core::length ch) 3)
        (:wat::core::let
          [newh (:user::enum-cmp-head (:user::kw-name (:wat::core::first ch)))
           v (:wat::core::nth ch 1)
           s (:wat::core::nth ch 2)]
          (:wat::core::if
            (:wat::core::if (:wat::string::empty? newh) false
              (:wat::core::if (:wat::hashset::contains? vars (:user::sym-name v))
                (:wat::core::= (:wat::core::ast-kind s) "string")
                false))
            (:wat::core::let [varn (:user::variant-of (:wat::core::ast-name s))]
              (:wat::core::if (:wat::string::empty? varn)
                (:user::empty-edits)
                (:user::one-edit
                  (:user::start-off node lines)
                  (:wat::core::ast->source node)
                  (:wat::string::concat
                    "("
                    (:wat::string::concat
                      newh
                      (:wat::string::concat
                        " "
                        (:wat::string::concat
                          (:wat::core::ast->source v)
                          (:wat::string::concat
                            " (:wat::grep::NodeKind::"
                            (:wat::string::concat varn "))")))))))))
            (:user::empty-edits)))
        (:user::empty-edits)))
    (:user::empty-edits)))

(:wat::core::defn :user::ctor-kind-edit
  [ch    <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::i64::>= (:wat::i64::+ i 1) (:wat::core::length ch))
    (:user::empty-edits)
    (:wat::core::let
      [a (:wat::core::nth ch i)
       b (:wat::core::nth ch (:wat::i64::+ i 1))
       rest (:user::ctor-kind-edit ch (:wat::i64::+ i 1) lines)]
      (:wat::core::if
        (:wat::core::if (:wat::core::= (:user::kw-name a) ":kind")
          (:wat::core::= (:wat::core::ast-kind b) "string")
          false)
        (:wat::core::let [varn (:user::variant-of (:wat::core::ast-name b))]
          (:wat::core::if (:wat::string::empty? varn)
            rest
            (:wat::core::concat
              (:user::one-edit
                (:user::start-off b lines)
                (:wat::core::ast->source b)
                (:wat::string::concat "(:wat::grep::NodeKind::" (:wat::string::concat varn ")")))
              rest)))
        rest))))

(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   vars  <- (:wat::core::HashSet :- [:wat::core::String])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:user::empty-edits)
        (:wat::core::let
          [here (:wat::core::concat
                  (:user::string-eq-edit node vars lines)
                  (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::grep::Node")
                    (:user::ctor-kind-edit ch 0 lines)
                    (:user::empty-edits)))]
          (:wat::core::concat here (:user::seq-edits ch vars lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) vars lines)
      (:user::empty-edits))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   vars  <- (:wat::core::HashSet :- [:wat::core::String])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     it  <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it vars lines)))
    (:user::empty-edits)
    items))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string src)
    [:wat::core::ReadOutcome.Forms {:forms forms}
      (:wat::core::let
        [lines (:wat::string::split src "\n")
         kids  (:wat::core::ast->children forms)
         vars  (:wat::core::foldl
                 (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::String]) n <- :wat::WatAST]
                   -> (:wat::core::HashSet :- [:wat::core::String])
                   (:user::collect-kind-vars n s))
                 (:wat::core::HashSet :- [:wat::core::String])
                 kids)
         eds   (:user::seq-edits kids vars lines)
         rev   (:wat::core::reverse (:wat::core::sort eds))]
        (:wat::fix::fix-text-apply src rev))]
    [:wat::core::ReadOutcome.Malformed {:cause cause}
      (:wat::kernel::assertion-failed! :message (:wat::core::Error/message cause))]))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   n     <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    (:wat::kernel::println
      (:wat::string::interpolate "CHANGED={n}" :n (:wat::i64::to-string n)))
    (:wat::core::let
      [path   (:wat::core::first paths)
       before (:wat::io::read-file path)
       after  (:user::migrate before)
       hit?   (:wat::core::not (:wat::core::= before after))]
      (:wat::core::do
        (:wat::core::if hit? (:wat::io::write-file path after) nil)
        (:wat::kernel::println
          (:wat::string::concat
            (:wat::core::if hit? "[changed] " "[unchanged] ")
            path))
        (:user::apply-each (:wat::core::rest paths)
          (:wat::core::if hit? (:wat::i64::+ n 1) n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v d} d]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])
    0))
