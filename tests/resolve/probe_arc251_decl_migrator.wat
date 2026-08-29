;; Migrator code baked in from wat-migrate/fix-decl.wat at migration time (arc 251 throwaway,
;; retires at hard-cut; non-blessed so cannot be auto-loaded — captured here verbatim).

(:wat::core::defn :migrate::name-fix [kw <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let [stripped (:wat::core::first
                                (:wat::string::split (:wat::core::ast-name kw) "<"))]
    (:wat::core::if (:wat::string::contains? stripped "::")
      (:wat::keyword::to-symbol (:wat::core::keyword-node stripped))
      (:wat::core::symbol-node
        (:wat::string::subs stripped 1 (:wat::string::length stripped))))))

(:wat::core::defn :migrate::type-slot-2? [head-name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= head-name ":wat::core::typealias") true
    (:wat::core::if (:wat::core::= head-name ":wat::core::newtype") true
      (:wat::core::= head-name ":wat::core::recordtype"))))

(:wat::core::defn :migrate::name-head? [head-name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= head-name ":wat::core::defn") true
    (:wat::core::if (:wat::core::= head-name ":wat::core::def") true
      (:wat::core::if (:wat::core::= head-name ":wat::core::typealias") true
        (:wat::core::if (:wat::core::= head-name ":wat::core::newtype") true
          (:wat::core::if (:wat::core::= head-name ":wat::core::recordtype") true
            (:wat::core::if (:wat::core::= head-name ":wat::core::defstruct") true
              (:wat::core::if (:wat::core::= head-name ":wat::core::defclause") true
                (:wat::core::if (:wat::core::= head-name ":wat::core::defenum") true
                  (:wat::core::= head-name ":wat::core::typeunion"))))))))))

(:wat::core::defn :migrate::fix-types [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::WatAST])
    (:wat::core::let [h   (:wat::core::first items)
                      out (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
                            (:wat::keyword::to-type-form h)
                            (:wat::fix::fix-source h))]
      (:wat::core::concat (:wat::core::Vector :- [:wat::WatAST] out)
                          (:migrate::fix-types (:wat::core::rest items))))))

(:wat::core::defn :migrate::fix-type-vector [vec <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::with-children vec (:migrate::fix-types (:wat::core::ast->children vec))))

(:wat::core::defn :migrate::fix-form [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch   (:wat::core::ast->children node)
                      head (:wat::core::first ch)]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                        (:migrate::name-head? (:wat::core::ast-name head))
                        false)
        ;; Arc 118.2a — `drop` flipped LAZY; `rest2` feeds `:wat::fix::fix-seq` ((Vector :- [WatAST])
        ;; param) both directly and via further `rest`/`concat`, so materialize here.
        (:wat::core::let [ch1   (:wat::core::nth ch 1)
                          rest2  (:wat::core::into [] (:wat::core::drop ch 2))
                          fixed-head (:wat::keyword::to-symbol head)
                          fixed-name (:migrate::name-fix ch1)
                          fixed-rest (:wat::core::if (:migrate::type-slot-2? (:wat::core::ast-name head))
                                       (:wat::core::if (:wat::core::empty? rest2)
                                         (:wat::core::Vector :- [:wat::WatAST])
                                         (:wat::core::let [ch2   (:wat::core::first rest2)
                                                           rest3  (:wat::core::rest rest2)]
                                           (:wat::core::concat
                                             (:wat::core::Vector :- [:wat::WatAST]
                                               (:wat::keyword::to-type-form ch2))
                                             (:wat::fix::fix-seq rest3 false))))
                                     (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::typeunion")
                                       (:wat::core::if (:wat::core::empty? rest2)
                                         (:wat::core::Vector :- [:wat::WatAST])
                                         (:wat::core::let [uch2  (:wat::core::first rest2)
                                                           urest (:wat::core::rest rest2)]
                                           (:wat::core::concat
                                             (:wat::core::Vector :- [:wat::WatAST]
                                               (:migrate::fix-type-vector uch2))
                                             (:wat::fix::fix-seq urest false))))
                                       (:wat::fix::fix-seq rest2 false)))]
          (:wat::core::with-children node
            (:wat::core::concat
              (:wat::core::Vector :- [:wat::WatAST] fixed-head)
              (:wat::core::concat
                (:wat::core::Vector :- [:wat::WatAST] fixed-name)
                fixed-rest))))
        (:wat::fix::fix-source node)))
    (:wat::fix::fix-source node)))

(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))

(:wat::core::defn :user::c01 [] -> :wat::core::String
  (:wat::core::write-forms (:migrate::fix-form (:user::topform "(:wat::core::typealias :svc::Alias :wat::core::i64)"))))
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::write-forms (:migrate::fix-form (:user::topform "(:wat::core::defn :my::ns::identity [x <- :i64] -> :i64 x)"))))
;; Arc 109 "annihilate the angle bracket" — re-pointed as a REFUSAL control that RETURNS
;; the cause's message instead of diverging through `assertion-failed!`. That return is
;; exactly the `(:wat::core::Error/message __cause)` path which was DEAD until the
;; ReadOutcome::Malformed cause started riding under a real `:wat::core::Fault` — so this
;; control now proves both halves: the reader refuses the angle form, AND the refusal is
;; reportable. The source never reaches the tool under test at all.
(:wat::core::defn :user::c03 [] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string "(:wat::core::typealias :Foo<T> :wat::core::Vector<wat::core::i64>)")
    ((:wat::core::ReadOutcome::Forms __forms) "READ-OK — the angle form was NOT refused")
    ((:wat::core::ReadOutcome::Malformed __cause) (:wat::core::Error/message __cause))))
(:wat::core::defn :user::c04 [] -> :wat::core::String
  (:wat::core::write-forms (:migrate::fix-form (:user::topform "(:wat::core::typealias :demo::edn::Tagged :wat::holon::HolonAST)"))))
(:wat::core::defn :user::c05 [] -> :wat::core::String
  (:wat::core::write-forms (:migrate::fix-form (:user::topform "(:wat::core::newtype :demo::edn::NoTag :wat::holon::HolonAST)"))))
(:wat::core::defn :user::c06 [] -> :wat::core::String
  (:wat::core::write-forms (:migrate::fix-form (:user::topform "(:wat::core::typeunion :my::Foo [:wat::core::i64 :wat::core::f64])"))))
(:wat::core::defn :user::c07 [] -> :wat::core::String
  (:wat::core::write-forms (:migrate::fix-form (:user::topform "(:wat::core::typeunion :my::Shape [:my::Circle :my::Square])"))))
(:wat::core::defn :user::c08 [] -> :wat::core::String
  (:wat::core::write-forms (:migrate::fix-form (:user::topform "(:wat::core::defenum :counter::AdminReq :Provision [initial <- :wat::core::i64])"))))
