;; wat-scripts/fixes/wrap-session-facts-in-factbag.wat — arc 278 FactBag ownership.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Two uniform wraps:
;;   (:wat::rete::Session/facts X)
;;     → (:wat::rete::factbag::items (:wat::rete::Session/facts X))
;;   inside (:wat::rete::Session … :facts EXPR …)
;;     → :facts (:wat::rete::FactBag :items EXPR)
;;
;; FireStratAcc's :facts is NOT touched (head is not Session).
;; Idempotent: an items-wrap of Session/facts is not re-wrapped; a :facts value
;; that is already a FactBag / factbag::* form is not re-wrapped.
;;
;; After this wrap, a small second pass (not this file) moves insert/retract/merge/retain
;; onto the semantic doors.
;;
;; Usage:
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-session-facts-in-factbag.wat

(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::head-kw-name [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    ""))

(:wat::core::defn :user::session-facts-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:user::head-kw-name node) ":wat::rete::Session/facts"))

(:wat::core::defn :user::items-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:user::head-kw-name node) ":wat::rete::factbag::items"))

(:wat::core::defn :user::session-ctor? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:user::head-kw-name node) ":wat::rete::Session"))

(:wat::core::defn :user::factbag-form? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [h (:user::head-kw-name node)]
    (:wat::core::if (:wat::core::= h ":wat::rete::FactBag")
      true
      (:wat::core::if (:wat::core::= h ":wat::rete::factbag::empty")
        true
        (:wat::core::if (:wat::core::= h ":wat::rete::factbag::add")
          true
          (:wat::core::if (:wat::core::= h ":wat::rete::factbag::of")
            true
            (:wat::core::if (:wat::core::= h ":wat::rete::factbag::items")
              true
              (:wat::core::if (:wat::core::= h ":wat::rete::factbag::add-if-absent")
                true
                (:wat::core::if (:wat::core::= h ":wat::rete::factbag::retain")
                  true
                  (:wat::core::= h ":wat::rete::factbag::remove-every-equal"))))))))))

(:wat::core::defn :user::already-items-wrapped? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:user::items-call? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        false
        (:user::session-facts-call? (:wat::core::Option/expect (:wat::core::get ch 1) "arg"))))
    false))

(:wat::core::defn :user::wrap-items-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
    (:wat::core::Tuple (:user::start-off node lines) 0 "(:wat::rete::factbag::items ")
    (:wat::core::Tuple (:user::end-off node lines) 0 ")")))

(:wat::core::defn :user::wrap-bag-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
    (:wat::core::Tuple (:user::start-off node lines) 0 "(:wat::rete::FactBag :items ")
    (:wat::core::Tuple (:user::end-off node lines) 0 ")")))

;; :facts value edits inside a Session ctor — wrap each value that is not already a bag form.
(:wat::core::defn :user::session-facts-field-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let [ch (:wat::core::ast->children node)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                       i   <- :wat::core::i64]
        -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
        (:wat::core::if (:wat::core::i64::< i (:wat::core::i64::- (:wat::core::length ch) 1))
          (:wat::core::let [k (:wat::core::Option/expect (:wat::core::get ch i) "k")
                            v (:wat::core::Option/expect (:wat::core::get ch (:wat::core::i64::+ i 1)) "v")]
            (:wat::core::if (:wat::core::= (:user::kw-name k) ":facts")
              (:wat::core::if (:user::factbag-form? v)
                acc
                (:wat::core::concat acc (:user::wrap-bag-edits v lines)))
              acc))
          acc))
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
      (:wat::core::range 0 (:wat::core::length ch)))))

(:wat::core::defn :user::node-edits-no-top
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::concat
            (:wat::core::if (:user::session-facts-call? node)
              (:user::wrap-items-edits node lines)
              (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))
            (:wat::core::if (:user::session-ctor? node)
              (:user::session-facts-field-edits node lines)
              (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))]
    (:wat::core::if (:user::already-items-wrapped? node)
      (:wat::core::let
        [ch    (:wat::core::ast->children node)
         head  (:wat::core::first ch)
         inner (:wat::core::Option/expect (:wat::core::get ch 1) "inner")]
        (:wat::core::concat
          (:user::node-edits head lines)
          (:user::node-edits-no-top inner lines)))
      (:wat::core::if (:wat::fix::structural? node)
        (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines))
        this))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    items))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::core::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds   (:user::seq-edits forms lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[wrap-factbag] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
