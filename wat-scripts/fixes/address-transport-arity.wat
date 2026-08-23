;; wat-scripts/fixes/address-transport-arity.wat — 293.W.2f
;; Self-hosted fix-wat codemod: no hand-editing of .wat files.
;;
;; Walks keyword leaves. A `:wat::kernel::Address<…>` / `:wat::spawn::Bound<…>`
;; token (or an uncoloned `wat::kernel::Address<…>` / `wat::spawn::Bound<…>`
;; embedded as a type-arg) with EXACTLY two top-level type args is rewritten
;; to append `,wat::kernel::Wire`. Nested `<…>` / `(…)` do not count inner
;; commas. A 3-arg form is left alone (idempotent).
;;
;; Comment-faithful: rides `fix-text-apply` span splices (copy rename-prefix).
;; Comments, formatting, and non-matching keywords survive byte-identical.
;;
;; ⚠ THE STASH-DANCE APPLIES (wat/fix.wat header) if the checker change
;; rejects 2-arg Address/Bound: stash the rust, build, rewrite, pop, rebuild.
;; GENERATE the path list — never hand-type it.
;;
;; Usage:
;;   printf '[…EVERY path…]\n' | cargo wat ./wat-scripts/fixes/address-transport-arity.wat

;; One char of s at index i (i in [0, length)).
(:wat::core::defn :user::ch
  [s <- :wat::core::String i <- :wat::core::i64] -> :wat::core::String
  (:wat::core::string::subs s i (:wat::core::+ i 1)))

;; Left-valid for an UNCOLONED embed: preceded by "<" "," " " or "(".
;; A leading ":" is NOT left-valid — that is the colon-form of the same name.
(:wat::core::defn :user::embed-left-ok?
  [name <- :wat::core::String i <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::core::< i 1)
    false
    (:wat::core::let [prev (:user::ch name (:wat::core::- i 1))]
      (:wat::core::if (:wat::core::= prev "<") true
        (:wat::core::if (:wat::core::= prev ",") true
          (:wat::core::if (:wat::core::= prev " ") true
            (:wat::core::= prev "(")))))))

;; Does `name` contain `pref` starting at i?
(:wat::core::defn :user::at-prefix?
  [name <- :wat::core::String i <- :wat::core::i64 pref <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [end (:wat::core::+ i (:wat::core::string::length pref))
                    nlen (:wat::core::string::length name)]
    (:wat::core::if (:wat::core::> end nlen)
      false
      (:wat::core::= (:wat::core::string::subs name i end) pref))))

;; Depth-walk: find the `>` that closes the `<` at `open` (open is the `<` index).
;; Tracks `<>` and `()` so tuple / nested parametric commas stay inner.
;; Returns the index of the matching `>`, or name-len if unbalanced.
(:wat::core::defn :user::matching-gt
  [name <- :wat::core::String open <- :wat::core::i64 i <- :wat::core::i64 depth <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let [nlen (:wat::core::string::length name)]
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

;; Count top-level commas in name[open+1 .. close) — depth 0 relative to that span.
(:wat::core::defn :user::count-commas
  [name <- :wat::core::String i <- :wat::core::i64 close <- :wat::core::i64 depth <- :wat::core::i64 acc <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::>= i close)
    acc
    (:wat::core::let [c (:user::ch name i)]
      (:wat::core::if (:wat::core::= c "<")
        (:user::count-commas name (:wat::core::+ i 1) close (:wat::core::+ depth 1) acc)
        (:wat::core::if (:wat::core::= c "(")
          (:user::count-commas name (:wat::core::+ i 1) close (:wat::core::+ depth 1) acc)
          (:wat::core::if (:wat::core::= c ">")
            (:user::count-commas name (:wat::core::+ i 1) close
              (:wat::core::if (:wat::core::= depth 0) 0 (:wat::core::- depth 1)) acc)
            (:wat::core::if (:wat::core::= c ")")
              (:user::count-commas name (:wat::core::+ i 1) close
                (:wat::core::if (:wat::core::= depth 0) 0 (:wat::core::- depth 1)) acc)
              (:wat::core::if (:wat::core::if (:wat::core::= c ",") (:wat::core::= depth 0) false)
                (:user::count-commas name (:wat::core::+ i 1) close depth (:wat::core::+ acc 1))
                (:user::count-commas name (:wat::core::+ i 1) close depth acc)))))))))

;; Top-level type-arg arity of the `<…>` whose `<` sits at `open`. Empty → 0.
(:wat::core::defn :user::type-arity
  [name <- :wat::core::String open <- :wat::core::i64 close <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::>= (:wat::core::+ open 1) close)
    0
    (:wat::core::+ 1 (:user::count-commas name (:wat::core::+ open 1) close 0 0))))

;; Match length at i, or 0 if no Address/Bound `<` starts here.
;; Prefers the colon form; uncoloned only when left-valid (embed).
(:wat::core::defn :user::match-len
  [name <- :wat::core::String i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:user::at-prefix? name i ":wat::kernel::Address<")
    (:wat::core::string::length ":wat::kernel::Address<")
    (:wat::core::if (:user::at-prefix? name i ":wat::spawn::Bound<")
      (:wat::core::string::length ":wat::spawn::Bound<")
      (:wat::core::if (:wat::core::if (:user::at-prefix? name i "wat::kernel::Address<")
                          (:user::embed-left-ok? name i)
                          false)
        (:wat::core::string::length "wat::kernel::Address<")
        (:wat::core::if (:wat::core::if (:user::at-prefix? name i "wat::spawn::Bound<")
                            (:user::embed-left-ok? name i)
                            false)
          (:wat::core::string::length "wat::spawn::Bound<")
          0)))))

;; Rewrite every 2-arg Address/Bound occurrence in `name`. Tail-recursive.
;; i is the current index; acc accumulates the output.
(:wat::core::defn :user::rewrite-name
  [name <- :wat::core::String i <- :wat::core::i64 acc <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [nlen (:wat::core::string::length name)]
    (:wat::core::if (:wat::core::>= i nlen)
      acc
      (:wat::core::let [ml (:user::match-len name i)]
        (:wat::core::if (:wat::core::> ml 0)
          (:wat::core::let [open (:wat::core::- (:wat::core::+ i ml) 1)
                            close (:user::matching-gt name open (:wat::core::+ open 1) 1)
                            arity (:user::type-arity name open close)]
            (:wat::core::if (:wat::core::= arity 2)
              (:user::rewrite-name name (:wat::core::+ close 1)
                (:wat::core::string::concat
                  (:wat::core::string::concat acc (:wat::core::string::subs name i close))
                  ",wat::kernel::Wire>"))
              (:user::rewrite-name name (:wat::core::+ close 1)
                (:wat::core::string::concat acc (:wat::core::string::subs name i (:wat::core::+ close 1))))))
          (:user::rewrite-name name (:wat::core::+ i 1)
            (:wat::core::string::concat acc (:user::ch name i))))))))

;; Walk a vector of nodes, concating arity-append edits.
(:wat::core::defn :user::arity-edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::concat
        (:user::arity-edits h lines)
        (:user::arity-edits-walk tl lines)))))

;; Keyword leaf → whole-token replace when the rewritten name differs.
;; Structural nodes recurse. Non-keyword leaves are untouched.
(:wat::core::defn :user::arity-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::arity-edits-walk (:wat::core::ast->children node) lines)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::let [name     (:wat::core::ast-name node)
                        name-len (:wat::core::string::length name)
                        new-name (:user::rewrite-name name 0 "")]
        (:wat::core::if (:wat::core::= new-name name)
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
          (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)]
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
              (:wat::core::Tuple off name-len new-name)))))
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
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
