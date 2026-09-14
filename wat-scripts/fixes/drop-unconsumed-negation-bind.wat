;; wat-scripts/fixes/drop-unconsumed-negation-bind.wat — composition repair (finding 18 / BRIEF 4b).
;; SCOPE: wat-scripts/fmt/rules/
;; Self-hosted fix-wat codemod: the wall names the sites; this tool only removes them.
;;
;; Input (stdin): a Vector of `path:line:col:var` strings, taken from the `--check`
;; report's `#wat.rete/UnconsumedWrapperBind` `:var` + `:span` `:file :line :col`.
;; Never re-derived — a second implementation of "fresh and unconsumed" would be
;; two slots that can disagree.
;;
;; Edit: remove the child list whose ast-span starts at line:col and one adjacent space
;; — via `fix-source` / `fix-text-apply` span edits.
;;
;; Guard: the node at the span must be exactly `(?<var> <- :<field>)` with the
;; reported var. A missing site (already dropped) is a no-op so a second run is
;; idempotent. A bind-shaped node of a different var at that span is skipped
;; (the first run already slid the neighbour into the column). Anything that
;; IS the reported var but not that bind shape refuses loudly and edits nothing.
;;
;; #50's message: "Declared must be consumed". The author fixed their 6
;; violations by dropping the dead bind. This is that same repair, recorded.
;;
;; Usage:
;;   printf '["wat-scripts/fmt/rules/kwargs.wat:31:33:?more" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/drop-unconsumed-negation-bind.wat

(:wat::core::defn :user::span-line [n <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::Option/expect (:wat::hashmap::get (:wat::core::ast-span n) :line) "span :line"))

(:wat::core::defn :user::span-col [n <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::Option/expect (:wat::hashmap::get (:wat::core::ast-span n) :col) "span :col"))

;; A bind list `(?v <- :field)` — three children, reported var, `<-`, a keyword field.
(:wat::core::defn :user::bind-of-var? [node <- :wat::WatAST  var <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
        (:wat::core::let [a (:wat::core::nth ch 0)
                          b (:wat::core::nth ch 1)
                          c (:wat::core::nth ch 2)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind a) "symbol")
            (:wat::core::if (:wat::core::= (:wat::core::ast-name a) var)
              (:wat::core::if (:wat::core::= (:wat::core::ast-kind b) "symbol")
                (:wat::core::if (:wat::core::= (:wat::core::ast-name b) "<-")
                  (:wat::core::= (:wat::core::ast-kind c) "keyword")
                  false)
                false)
              false)
            false))
        false))
    false))

;; Reported var at this span, but NOT the bind shape — refuse, do not edit.
(:wat::core::defn :user::wrong-shape-of-var? [node <- :wat::WatAST  var <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:user::bind-of-var? node var)
    false
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::empty? ch)
          false
          (:wat::core::let [a (:wat::core::first ch)]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind a) "symbol")
              (:wat::core::= (:wat::core::ast-name a) var)
              false))))
      false)))

(:wat::core::defn :user::find-at
  [node <- :wat::WatAST
   line <- :wat::core::i64
   col  <- :wat::core::i64]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::if (:wat::core::if (:wat::core::= (:user::span-line node) line)
                    (:wat::core::= (:user::span-col node) col)
                    false)
    (:wat::core::Option.Some {:value node})
    (:wat::core::if (:wat::fix::structural? node)
      (:user::find-at-children (:wat::core::ast->children node) line col)
      :wat::core::Option.None)))

(:wat::core::defn :user::find-at-children
  [kids <- (:wat::core::Vector :- [:wat::WatAST])
   line <- :wat::core::i64
   col  <- :wat::core::i64]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::if (:wat::core::empty? kids)
    :wat::core::Option.None
    (:wat::core::match (:user::find-at (:wat::core::first kids) line col)
      [:wat::core::Option.Some {:value n} (:wat::core::Option.Some {:value n})]
      [:wat::core::Option.None {}
        (:user::find-at-children (:wat::core::rest kids) line col)])))

(:wat::core::defn :user::parse-site [s <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [parts (:wat::string::split s ":")]
    (:wat::core::if (:wat::core::= (:wat::core::length parts) 4)
      parts
      (:wat::kernel::assertion-failed!
        :message (:wat::string::concat "drop-unconsumed-negation-bind: malformed site (want path:line:col:var): " s)))))

;; Deletion covers the bind's token span plus one adjacent space (trailing preferred).
(:wat::core::defn :user::deletion-for
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
  (:wat::core::let [start (:wat::core::ast-span node)
                    end   (:wat::core::ast-end-span node)
                    off   (:wat::fix::fix-text-offset-of start lines)
                    end-off (:wat::fix::fix-text-offset-of end lines)
                    body  (:wat::fix::fix-text-span-text start end lines src)
                    n     (:wat::string::length src)]
    (:wat::core::if (:wat::core::if (:wat::core::< end-off n)
                      (:wat::core::= (:wat::string::subs src end-off (:wat::i64::+ end-off 1)) " ")
                      false)
      (:wat::core::Tuple off (:wat::string::concat body " ") "")
      (:wat::core::if (:wat::core::if (:wat::core::> off 0)
                        (:wat::core::= (:wat::string::subs src (:wat::i64::- off 1) off) " ")
                        false)
        (:wat::core::Tuple (:wat::i64::- off 1) (:wat::string::concat " " body) "")
        (:wat::core::Tuple off body "")))))

(:wat::core::defn :user::site-edit
  [tree  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])
   line  <- :wat::core::i64
   col   <- :wat::core::i64
   var   <- :wat::core::String
   site  <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::match (:user::find-at tree line col)
    [:wat::core::Option.None {}
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
    [:wat::core::Option.Some {:value n}
      (:wat::core::if (:user::bind-of-var? n var)
        (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
          (:user::deletion-for n src lines))
        (:wat::core::if (:user::wrong-shape-of-var? n var)
          (:wat::kernel::assertion-failed!
            :message (:wat::string::concat
                       "drop-unconsumed-negation-bind: guard refused "
                       (:wat::string::concat site " — node at span is not `(?<var> <- :<field>)`")))
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))]))

;; Later in the file = higher line, or same line and higher col. Bottom-up so
;; a same-line neighbour's column stays valid after we splice.
(:wat::core::defn :user::later? [a <- :wat::core::String  b <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [pa (:user::parse-site a)
                    pb (:user::parse-site b)
                    la (:wat::core::Option/expect (:wat::string::to-i64 (:wat::core::nth pa 1)) "later line a")
                    ca (:wat::core::Option/expect (:wat::string::to-i64 (:wat::core::nth pa 2)) "later col a")
                    lb (:wat::core::Option/expect (:wat::string::to-i64 (:wat::core::nth pb 1)) "later line b")
                    cb (:wat::core::Option/expect (:wat::string::to-i64 (:wat::core::nth pb 2)) "later col b")]
    (:wat::core::if (:wat::core::> la lb)
      true
      (:wat::core::if (:wat::core::< la lb)
        false
        (:wat::core::> ca cb)))))

(:wat::core::defn :user::insert-sorted
  [site <- :wat::core::String
   acc  <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? acc)
    (:wat::core::Vector :- [:wat::core::String] site)
    (:wat::core::if (:user::later? site (:wat::core::first acc))
      (:wat::core::concat (:wat::core::Vector :- [:wat::core::String] site) acc)
      (:wat::core::concat
        (:wat::core::Vector :- [:wat::core::String] (:wat::core::first acc))
        (:user::insert-sorted site (:wat::core::rest acc))))))

(:wat::core::defn :user::sort-sites
  [sites <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                     s   <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:user::insert-sorted s acc))
    (:wat::core::Vector :- [:wat::core::String])
    sites))

(:wat::core::defn :user::sites-for-path
  [path  <- :wat::core::String
   sites <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? sites)
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::let [s (:wat::core::first sites)
                      p (:wat::core::nth (:user::parse-site s) 0)]
      (:wat::core::if (:wat::core::= p path)
        (:wat::core::concat
          (:wat::core::Vector :- [:wat::core::String] s)
          (:user::sites-for-path path (:wat::core::rest sites)))
        (:user::sites-for-path path (:wat::core::rest sites))))))

(:wat::core::defn :user::apply-one
  [path <- :wat::core::String
   site <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let [src   (:wat::io::read-file path)
                    lines (:wat::string::split src "\n")
                    tree  (:wat::core::match (:wat::core::read-string src)
                            [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
                            [:wat::core::ReadOutcome.Malformed {:cause __cause}
                              (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    parts (:user::parse-site site)
                    line  (:wat::core::Option/expect (:wat::string::to-i64 (:wat::core::nth parts 1)) "apply line")
                    col   (:wat::core::Option/expect (:wat::string::to-i64 (:wat::core::nth parts 2)) "apply col")
                    var   (:wat::core::nth parts 3)
                    edits (:user::site-edit tree src lines line col var site)]
    (:wat::core::if (:wat::core::empty? edits)
      nil
      (:wat::io::write-file path (:wat::fix::fix-text-apply src edits)))))

(:wat::core::defn :user::apply-sorted
  [path  <- :wat::core::String
   sites <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? sites)
    nil
    (:wat::core::do
      (:user::apply-one path (:wat::core::first sites))
      (:user::apply-sorted path (:wat::core::rest sites)))))

(:wat::core::defn :user::migrate-path
  [path  <- :wat::core::String
   sites <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::do
    (:user::apply-sorted path (:user::sort-sites (:user::sites-for-path path sites)))
    (:wat::kernel::println (:wat::string::concat "[drop-unconsumed-negation-bind] " path))
    nil))

(:wat::core::defn :user::paths-of
  [sites <- (:wat::core::Vector :- [:wat::core::String])
   acc   <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? sites)
    acc
    (:wat::core::let [p (:wat::core::nth (:user::parse-site (:wat::core::first sites)) 0)]
      (:wat::core::if (:wat::vec::contains? acc p)
        (:user::paths-of (:wat::core::rest sites) acc)
        (:user::paths-of (:wat::core::rest sites)
          (:wat::core::conj acc p))))))

(:wat::core::defn :user::apply-paths
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   sites <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:user::migrate-path (:wat::core::first paths) sites)
      (:user::apply-paths (:wat::core::rest paths) sites))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [sites (:wat::core::match (:wat::kernel::readln)
                            [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
                            [:wat::kernel::ReadlnOutcome.Eof {}
                              (:wat::kernel::assertion-failed! :message "readln: end of input")]
                            [:wat::kernel::ReadlnOutcome.Stopped {}
                              (:wat::kernel::assertion-failed! :message "readln: stop requested")])]
    (:user::apply-paths (:user::paths-of sites (:wat::core::Vector :- [:wat::core::String])) sites)))
