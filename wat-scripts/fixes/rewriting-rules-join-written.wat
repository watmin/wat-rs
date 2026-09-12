;; wat-scripts/fixes/rewriting-rules-join-written.wat — arc 294 stone 0a, codemod A.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; In every `(:wat::rete::defrule …)` whose `:when` holds
;;   `(:wat::grep::Named (?I <- :id) (?N <- :name))`
;;   AND
;;   `(:wat::grep::Span (?I <- :id) (?L <- :line) (?C <- :col) (?EL <- :end-line) (?EC <- :end-col))`
;; on the same id variable: delete the Named pattern's whole line, and turn the Span
;; pattern into
;;   `(:wat::grep::Written (?I <- :id) (?N <- :text) (?L <- :line) (?C <- :col) (?EL <- :end-line) (?EC <- :end-col))`.
;;
;; Span-faithful, idempotent. Never run over this file. Rewriters join Written; readers keep
;; joining Named. `?n` stays the variable name, so every `where` and `:then` below the join
;; is unchanged.
;;
;; SCOPE: wat-scripts/fixes/rename-core-string-to-string.wat wat-scripts/fixes/rename-four-families-to-their-homes.wat wat-scripts/fixes/rename-core-numerics-to-their-homes.wat wat-scripts/fixes/rename-rete-numerics-to-their-homes.wat wat-scripts/fixes/rename-core-bigint-rational-to-their-homes.wat wat-scripts/fixes/rename-core-maps-to-their-homes.wat wat-scripts/fixes/rename-core-vectors-to-their-homes.wat wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat wat-scripts/fixes/rename-keyword-to-its-home.wat wat-scripts/fixes/rename-string-verbs-to-their-home.wat wat-scripts/fixes/rename-math-stat-seq-to-their-homes.wat
;;
;; Usage:
;;   printf '["wat-scripts/fixes/rename-core-string-to-string.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/rewriting-rules-join-written.wat

(:wat::core::defn :user::empty-edits []
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::Vector :- [:wat::fix::Edit]))

(:wat::core::defn :user::one-edit
  [off <- :wat::core::i64  old <- :wat::core::String  new <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::Vector :- [:wat::fix::Edit]
    (:wat::core::Tuple off old new)))

(:wat::core::defn :user::src-of
  [n     <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::fix::fix-text-span-text
    (:wat::core::ast-span n) (:wat::core::ast-end-span n) lines src))

(:wat::core::defn :user::line-of [n <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::hashmap::get (:wat::core::ast-span n) :line)
    "rewriting-rules-join-written: node has no :line"))

(:wat::core::defn :user::binder-sym [binder <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children binder)]
    (:wat::core::if (:wat::core::empty? ch)
      ""
      (:wat::core::ast-name (:wat::core::first ch)))))

(:wat::core::defn :user::named-pat? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? n ":wat::grep::Named")
    (:wat::core::= (:wat::core::count (:wat::core::ast->children n)) 3)
    false))

(:wat::core::defn :user::span-pat? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? n ":wat::grep::Span")
    (:wat::core::= (:wat::core::count (:wat::core::ast->children n)) 6)
    false))

(:wat::core::defn :user::pat-id [n <- :wat::WatAST] -> :wat::core::String
  (:user::binder-sym (:wat::core::nth (:wat::core::ast->children n) 1)))

(:wat::core::defn :user::named-var [n <- :wat::WatAST] -> :wat::core::String
  (:user::binder-sym (:wat::core::nth (:wat::core::ast->children n) 2)))

(:wat::core::defn :user::when-kw? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::kw-name n) ":when"))

(:wat::core::defn :user::no-node [] -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::Option.None {}))

(:wat::core::defn :user::find-when-vec
  [ch <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::if (:wat::core::< (:wat::core::count ch) 2)
    (:user::no-node)
    (:wat::core::if (:user::when-kw? (:wat::core::first ch))
      (:wat::core::Option.Some {:value (:wat::core::nth ch 1)})
      (:user::find-when-vec (:wat::core::into [] (:wat::core::rest ch))))))

(:wat::core::defn :user::find-span-with-id
  [id   <- :wat::core::String
   pats <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Option :- [:wat::WatAST])
                     p   <- :wat::WatAST]
      -> (:wat::core::Option :- [:wat::WatAST])
      (:wat::core::match acc
        [:wat::core::Option.Some {:value __v} acc]
        [:wat::core::Option.None {}
          (:wat::core::if (:wat::core::if (:user::span-pat? p)
                            (:wat::core::= (:user::pat-id p) id)
                            false)
            (:wat::core::Option.Some {:value p})
            acc)]))
    (:user::no-node)
    pats))

(:wat::core::defn :user::line-slice
  [line  <- :wat::core::i64
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [start (:wat::fix::fix-text-line-start line lines)
     n     (:wat::core::count lines)
     end   (:wat::core::if (:wat::core::< line n)
             (:wat::fix::fix-text-line-start (:wat::core::+ line 1) lines)
             (:wat::string::length src))]
    (:wat::core::Tuple start (:wat::string::subs src start end))))

(:wat::core::defn :user::written-text
  [named <- :wat::WatAST
   span  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::let
    [sch   (:wat::core::ast->children span)
     id-s  (:user::src-of (:wat::core::nth sch 1) src lines)
     line-s (:user::src-of (:wat::core::nth sch 2) src lines)
     col-s (:user::src-of (:wat::core::nth sch 3) src lines)
     el-s  (:user::src-of (:wat::core::nth sch 4) src lines)
     ec-s  (:user::src-of (:wat::core::nth sch 5) src lines)
     nv    (:user::named-var named)]
    (:wat::string::interpolate
      "(:wat::grep::Written {id} ({nv} <- :text) {line} {col} {el} {ec})"
      :id id-s :nv nv :line line-s :col col-s :el el-s :ec ec-s)))

(:wat::core::defn :user::pair-edits
  [named <- :wat::WatAST
   span  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [del (:user::line-slice (:user::line-of named) src lines)
     off (:wat::fix::node-start-offset span lines)
     old (:user::src-of span src lines)
     new (:user::written-text named span src lines)]
    (:wat::core::concat
      (:user::one-edit (:wat::core::first del) (:wat::core::second del) "")
      (:user::one-edit off old new))))

(:wat::core::defn :user::named-pair-edits
  [named <- :wat::WatAST
   pats  <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::named-pat? named)
    (:wat::core::match (:user::find-span-with-id (:user::pat-id named) pats)
      [:wat::core::Option.None {} (:user::empty-edits)]
      [:wat::core::Option.Some {:value span} (:user::pair-edits named span src lines)])
    (:user::empty-edits)))

(:wat::core::defn :user::when-edits
  [when-vec <- :wat::WatAST
   src      <- :wat::core::String
   lines    <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind when-vec) "vector")
    (:wat::core::let [pats (:wat::core::ast->children when-vec)]
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::fix::Edit])
                         p   <- :wat::WatAST]
          -> (:wat::core::Vector :- [:wat::fix::Edit])
          (:wat::core::concat acc (:user::named-pair-edits p pats src lines)))
        (:user::empty-edits)
        pats))
    (:user::empty-edits)))

(:wat::core::defn :user::defrule-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::fix::calls-to? node ":wat::rete::defrule")
    (:wat::core::match (:user::find-when-vec (:wat::core::ast->children node))
      [:wat::core::Option.None {} (:user::empty-edits)]
      [:wat::core::Option.Some {:value wv} (:user::when-edits wv src lines)])
    (:user::empty-edits)))

(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [this (:user::defrule-edits node src lines)]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::walk-seq-edits (:wat::core::ast->children node) src lines))
      this)))

(:wat::core::defn :user::walk-seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:user::empty-edits)
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) src lines)
      (:user::walk-seq-edits (:wat::core::into [] (:wat::core::rest items)) src lines))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
             [:wat::core::ReadOutcome.Malformed {:cause __cause}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
     forms (:wat::core::ast->children tree)
     edits (:user::walk-seq-edits forms src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort edits)))))

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
        (:user::apply-each (:wat::core::into [] (:wat::core::rest paths))
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
