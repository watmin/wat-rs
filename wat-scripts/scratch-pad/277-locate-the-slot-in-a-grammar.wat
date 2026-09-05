
;; 277-locate-the-slot-in-a-grammar.wat — IS THE RET-SPEC SLOT DERIVABLE FROM at-syntax?
;;
;; All 36 grammars parse (277-can-wat-read-its-own-grammar). Parsing is not the question the
;; builder ruling rests on. The question is whether the GRAMMAR SHOWS THE SLOT -- whether the
;; arrow and its return type sit adjacent in a way a rule can read, so the default can withhold
;; a break between them and honour "ret-spec is a single line".

(:wat::core::defn :s::describe [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [k (:wat::core::ast-kind n)]
    (:wat::core::if (:wat::core::= k "list") "(list)"
      (:wat::core::if (:wat::core::= k "vector") "[vector]"
        (:wat::core::ast->source n)))))

(:wat::core::defn :s::show-kid [pair <- (:wat::core::Tuple :- [:wat::core::i64 :wat::WatAST])] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::interpolate "  idx {i}  kind={k}  src={s}"
    :i (:wat::i64::to-string (:wat::core::first pair))
    :k (:wat::core::ast-kind (:wat::core::second pair))
    :s (:s::describe (:wat::core::second pair)))))

(:wat::core::defn :s::walk [name <- :wat::core::String  syntax <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:wat::core::read-string syntax)
    ((:wat::core::ReadOutcome::Forms forms)
      (:wat::core::let
        [form (:wat::core::first (:wat::core::ast->children forms))
         kids (:wat::core::ast->children form)
         idx  (:wat::core::into (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::WatAST])])
                (:wat::core::map-indexed
                  (:wat::core::fn [i <- :wat::core::i64  k <- :wat::WatAST]
                    -> (:wat::core::Tuple :- [:wat::core::i64 :wat::WatAST])
                    (:wat::core::Tuple i k))
                  kids))]
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat "GRAMMAR OF " name))
          (:wat::core::run! :s::show-kid idx))))
    ((:wat::core::ReadOutcome::Malformed c)
      (:wat::kernel::println "unreadable"))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:s::walk ":wat.core/fn"  "(:wat::core::fn [<param> <- :T ...] -> :RetType <body>+)")
    (:s::walk ":wat.core/let" "(:wat::core::let [<binder> <expr> ...] <body>+)")))
