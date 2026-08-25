;; wat-scripts/fixes/unstamp-transport-wire.wat — 293.W.2f class B
;; Self-hosted fix-wat codemod: no hand-editing of .wat files.
;;
;; Inverse of address-transport-arity.wat on slots that must stay T-unknown.
;; Walks keyword leaves. A `:wat::kernel::Address<…>` / `:wat::spawn::Bound<…>`
;; (or uncoloned embed) whose LAST top-level type arg is exactly
;; `wat::kernel::Wire` / `:wat::kernel::Wire` is rewritten to drop that arg
;; (back to 2-arg = T unknown). Nested `<…>` / `(…)` do not count inner commas.
;; Address<S,R,T> / Address<S,R,Shared> / already-2-arg: left alone (idempotent).
;;
;; Declaration sites that must accept Shared OR Wire stay 2-arg. Process
;; listener still stamps Wire in the checker; Status<T>/Handle<T> carry T.
;;
;; Comment-faithful: rides `fix-text-apply` span splices (copy rename-prefix).
;;
;; Usage:
;;   printf '[…EVERY path…]\n' | cargo wat ./wat-scripts/fixes/unstamp-transport-wire.wat

(:wat::core::defn :user::ch
  [s <- :wat::core::String i <- :wat::core::i64] -> :wat::core::String
  (:wat::string::subs s i (:wat::core::+ i 1)))

(:wat::core::defn :user::embed-left-ok?
  [name <- :wat::core::String i <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::core::< i 1)
    false
    (:wat::core::let [prev (:user::ch name (:wat::core::- i 1))]
      (:wat::core::if (:wat::core::= prev "<") true
        (:wat::core::if (:wat::core::= prev ",") true
          (:wat::core::if (:wat::core::= prev " ") true
            (:wat::core::= prev "(")))))))

(:wat::core::defn :user::at-prefix?
  [name <- :wat::core::String i <- :wat::core::i64 pref <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [end (:wat::core::+ i (:wat::string::length pref))
                    nlen (:wat::string::length name)]
    (:wat::core::if (:wat::core::> end nlen)
      false
      (:wat::core::= (:wat::string::subs name i end) pref))))

(:wat::core::defn :user::matching-gt
  [name <- :wat::core::String open <- :wat::core::i64 i <- :wat::core::i64 depth <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let [nlen (:wat::string::length name)]
    (:wat::core::if (:wat::core::>= i nlen)
      nlen
      (:wat::core::let [c (:user::ch name i)]
        (:wat::core::if (:wat::core::= c "<")
          (:user::matching-gt name open (:wat::core::+ i 1) (:wat::core::+ depth 1))
          (:wat::core::if (:wat::core::= c "(")
            (:user::matching-gt name open (:wat::core::+ i 1) (:wat::core::+ depth 1))
            (:wat::core::if (:wat::core::= c ">")
              (:wat::core::if (:wat::core::= depth 1)
                i
                (:user::matching-gt name open (:wat::core::+ i 1) (:wat::core::- depth 1)))
              (:wat::core::if (:wat::core::= c ")")
                (:user::matching-gt name open (:wat::core::+ i 1)
                  (:wat::core::if (:wat::core::= depth 1) 1 (:wat::core::- depth 1)))
                (:user::matching-gt name open (:wat::core::+ i 1) depth)))))))))

;; Last top-level comma in name[open+1 .. close). -1 if none.
(:wat::core::defn :user::last-comma
  [name <- :wat::core::String i <- :wat::core::i64 close <- :wat::core::i64 depth <- :wat::core::i64 acc <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::>= i close)
    acc
    (:wat::core::let [c (:user::ch name i)]
      (:wat::core::if (:wat::core::= c "<")
        (:user::last-comma name (:wat::core::+ i 1) close (:wat::core::+ depth 1) acc)
        (:wat::core::if (:wat::core::= c "(")
          (:user::last-comma name (:wat::core::+ i 1) close (:wat::core::+ depth 1) acc)
          (:wat::core::if (:wat::core::= c ">")
            (:user::last-comma name (:wat::core::+ i 1) close
              (:wat::core::if (:wat::core::= depth 0) 0 (:wat::core::- depth 1)) acc)
            (:wat::core::if (:wat::core::= c ")")
              (:user::last-comma name (:wat::core::+ i 1) close
                (:wat::core::if (:wat::core::= depth 0) 0 (:wat::core::- depth 1)) acc)
              (:wat::core::if (:wat::core::if (:wat::core::= c ",") (:wat::core::= depth 0) false)
                (:user::last-comma name (:wat::core::+ i 1) close depth i)
                (:user::last-comma name (:wat::core::+ i 1) close depth acc)))))))))

(:wat::core::defn :user::wire-arg?
  [s <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= s "wat::kernel::Wire") true
    (:wat::core::= s ":wat::kernel::Wire")))

(:wat::core::defn :user::match-len
  [name <- :wat::core::String i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:user::at-prefix? name i ":wat::kernel::Address<")
    (:wat::string::length ":wat::kernel::Address<")
    (:wat::core::if (:user::at-prefix? name i ":wat::spawn::Bound<")
      (:wat::string::length ":wat::spawn::Bound<")
      (:wat::core::if (:wat::core::if (:user::at-prefix? name i "wat::kernel::Address<")
                          (:user::embed-left-ok? name i)
                          false)
        (:wat::string::length "wat::kernel::Address<")
        (:wat::core::if (:wat::core::if (:user::at-prefix? name i "wat::spawn::Bound<")
                            (:user::embed-left-ok? name i)
                            false)
          (:wat::string::length "wat::spawn::Bound<")
          0)))))

;; Drop `,wat::kernel::Wire` when it is the last top-level type arg.
(:wat::core::defn :user::rewrite-name
  [name <- :wat::core::String i <- :wat::core::i64 acc <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [nlen (:wat::string::length name)]
    (:wat::core::if (:wat::core::>= i nlen)
      acc
      (:wat::core::let [ml (:user::match-len name i)]
        (:wat::core::if (:wat::core::> ml 0)
          (:wat::core::let [open (:wat::core::- (:wat::core::+ i ml) 1)
                            close (:user::matching-gt name open (:wat::core::+ open 1) 1)
                            comma (:user::last-comma name (:wat::core::+ open 1) close 0 -1)]
            (:wat::core::if (:wat::core::< comma 0)
              (:user::rewrite-name name (:wat::core::+ close 1)
                (:wat::string::concat acc (:wat::string::subs name i (:wat::core::+ close 1))))
              (:wat::core::let [last-arg (:wat::string::subs name (:wat::core::+ comma 1) close)]
                (:wat::core::if (:user::wire-arg? last-arg)
                  (:user::rewrite-name name (:wat::core::+ close 1)
                    (:wat::string::concat
                      (:wat::string::concat acc (:wat::string::subs name i comma))
                      ">"))
                  (:user::rewrite-name name (:wat::core::+ close 1)
                    (:wat::string::concat acc (:wat::string::subs name i (:wat::core::+ close 1))))))))
          (:user::rewrite-name name (:wat::core::+ i 1)
            (:wat::string::concat acc (:user::ch name i))))))))

(:wat::core::defn :user::arity-edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::concat
        (:user::arity-edits h lines)
        (:user::arity-edits-walk tl lines)))))

(:wat::core::defn :user::arity-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::arity-edits-walk (:wat::core::ast->children node) lines)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::let [name     (:wat::core::ast-name node)
                        name-len name
                        new-name (:user::rewrite-name name 0 "")]
        (:wat::core::if (:wat::core::= new-name name)
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
          (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)]
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
              (:wat::core::Tuple off name-len new-name)))))
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::arity-edits-walk forms lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

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
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
