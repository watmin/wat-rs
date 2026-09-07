;; wat-scripts/fixes/assertion-failed-to-kwargs.wat — arc 109.
;;
;; Self-hosted, comment-faithful. Rewrites every `:wat::kernel::assertion-failed!`
;; (and `wat.kernel/assertion-failed!`) 3-arg POSITIONAL call:
;;
;;   (assertion-failed! "msg" :wat::core::None :wat::core::None)
;;     ->  (assertion-failed! :message "msg")
;;   (assertion-failed! "msg" a e)
;;     ->  (assertion-failed! :message "msg" :actual a :expected e)
;;
;; A trailing None is DROPPED, never rewritten as `:actual :None` (STOP-4).
;; Idempotent: a call whose first arg is already a keyword is kwargs and is left.
;; Nested calls inside a rewritten node are not separately edited (the parent
;; replacement is the whole list); a second pass is a no-op on kwargs.
;;
;; Skip: tests/kernel/probe_arc109_assertion_kwargs__positional.wat — the STOP-3
;; control that must remain positional so the retired form can be refused.
;;
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/assertion-failed-to-kwargs.wat

(:wat::core::defn :user::node-text
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::string::subs src
    (:wat::fix::node-start-offset node lines)
    (:wat::fix::node-end-offset node lines)))

(:wat::core::defn :user::head-spelling [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        ""
        (:wat::core::let [h (:wat::core::first ch) k (:wat::core::ast-kind h)]
          (:wat::core::if (:wat::core::or (:wat::core::= k "keyword") (:wat::core::= k "symbol"))
            (:wat::core::ast-name h)
            ""))))
    ""))

(:wat::core::defn :user::assertion-failed-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [h (:user::head-spelling node)]
    (:wat::core::or
      (:wat::core::= h ":wat::kernel::assertion-failed!")
      (:wat::core::= h "wat.kernel/assertion-failed!"))))

(:wat::core::defn :user::none-placeholder? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind n)]
    (:wat::core::if (:wat::core::= k "keyword")
      (:wat::core::let [nm (:wat::core::ast-name n)]
        (:wat::core::or (:wat::core::= nm ":wat::core::None")
          (:wat::core::or (:wat::core::= nm ":None")
            (:wat::core::= nm ":wat::core::Option::None"))))
      (:wat::core::if (:wat::core::= k "symbol")
        (:wat::core::let [nm (:wat::core::ast-name n)]
          (:wat::core::or (:wat::core::= nm "wat.core/None")
            (:wat::core::= nm "wat.core/Option.None")))
        false))))

;; 3 positional args, first not a keyword (kwargs already start with :message etc.).
(:wat::core::defn :user::positional-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:user::assertion-failed-call? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 4)
        (:wat::core::not (:wat::core::= (:wat::core::ast-kind
                                         (:wat::core::Option/expect (:wat::core::get ch 1) "pos first arg"))
                                       "keyword"))
        false))
    false))

(:wat::core::defn :user::rewrite-text
  [head-text <- :wat::core::String
   msg-text  <- :wat::core::String
   act-text  <- :wat::core::String
   exp-text  <- :wat::core::String
   drop-act  <- :wat::core::bool
   drop-exp  <- :wat::core::bool]
  -> :wat::core::String
  (:wat::core::let [base (:wat::string::concat "("
                         (:wat::string::concat head-text
                           (:wat::string::concat " :message " msg-text)))]
    (:wat::core::if drop-act
      (:wat::core::if drop-exp
        (:wat::string::concat base ")")
        (:wat::string::concat base
          (:wat::string::concat " :expected "
            (:wat::string::concat exp-text ")"))))
      (:wat::core::if drop-exp
        (:wat::string::concat base
          (:wat::string::concat " :actual "
            (:wat::string::concat act-text ")")))
        (:wat::string::concat base
          (:wat::string::concat " :actual "
            (:wat::string::concat act-text
              (:wat::string::concat " :expected "
                (:wat::string::concat exp-text ")")))))))))

(:wat::core::defn :user::call-edit
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
  (:wat::core::let
    [ch   (:wat::core::ast->children node)
     head (:wat::core::Option/expect (:wat::core::get ch 0) "call-edit head")
     msg  (:wat::core::Option/expect (:wat::core::get ch 1) "call-edit msg")
     act  (:wat::core::Option/expect (:wat::core::get ch 2) "call-edit actual")
     exp  (:wat::core::Option/expect (:wat::core::get ch 3) "call-edit expected")
     off  (:wat::fix::node-start-offset node lines)
     old  (:user::node-text node src lines)
     new  (:user::rewrite-text
            (:user::node-text head src lines)
            (:user::node-text msg src lines)
            (:user::node-text act src lines)
            (:user::node-text exp src lines)
            (:user::none-placeholder? act)
            (:user::none-placeholder? exp))]
    (:wat::core::Tuple off old new)))

;; Walk: a matching positional call emits ONE whole-node edit and does not
;; recurse (the replacement covers the list). Everything else recurses.
(:wat::core::defn :user::edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::positional-call? node)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:user::call-edit node src lines))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::edits-seq (:wat::core::ast->children node) src lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::edits-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::edits (:wat::core::first items) src lines)
      (:user::edits-seq (:wat::core::into [] (:wat::core::rest items)) src lines))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome::Forms {:forms __forms} __forms]
             [:wat::core::ReadOutcome::Malformed {:cause __cause}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
     eds   (:user::edits-seq (:wat::core::ast->children tree) src lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::rewrite-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::if (:wat::core::= path "tests/kernel/probe_arc109_assertion_kwargs__positional.wat")
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat "[assertion-kwargs] skip positional-control " path))
          (:user::rewrite-each (:wat::core::rest paths)))
        (:wat::core::do
          (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
          (:wat::kernel::println (:wat::string::concat "[assertion-kwargs] " path))
          (:user::rewrite-each (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome::Datum {:v __datum} __datum]
             [:wat::kernel::ReadlnOutcome::Eof {}
               (:wat::kernel::assertion-failed! :message "readln: end of input")]
             [:wat::kernel::ReadlnOutcome::Stopped {}
               (:wat::kernel::assertion-failed! :message "readln: stop requested")])]
    (:user::rewrite-each paths)))
