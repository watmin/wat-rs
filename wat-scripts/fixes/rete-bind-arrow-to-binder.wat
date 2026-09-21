;; wat-scripts/fixes/rete-bind-arrow-to-binder.wat — 251.8d-i-b AMEND: rete learns `:-`.
;; SCOPE: corpus
;;
;; Surgical: a rete field/fact/accum binding `(?k <- :k)` becomes `(?k :- :k)`.
;; The arrow converts; the field-name keyword stays. Param annotations
;; `[x <- :T]` and return arrows `->` are left alone (previous sibling is not
;; a `?`-prefixed rete-var). Comments are not in the AST and stay.
;;
;; `(?k <- :k)` → `(?k :- :k)`
;; `[x <- :wat::core::i64]` stays (near-miss: not a rete-var)
;;
;; Idempotent: a second pass sees `:-` (not `<-`) and emits zero edits.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/grep.wat" …]\n' | ./target/release/wat ./wat-scripts/fixes/rete-bind-arrow-to-binder.wat

;; left-arrow? — the rete binding arrow specifically (`<-`), not `->`.
(:wat::core::defn :user::left-arrow? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
    (:wat::core::= (:wat::core::ast-name node) "<-")
    false))

;; leaf: convert `<-` only when the previous sibling was a rete-var.
(:wat::core::defn :user::leaf-edits
  [node           <- :wat::WatAST
   prev-rete-var? <- :wat::core::bool
   lines          <- (:wat::core::Vector :- [:wat::core::String])
   src            <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::if prev-rete-var?
                    (:wat::core::if (:user::left-arrow? node)
                      (:wat::fix::source-matches-name? node lines src)
                      false)
                    false)
    (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                      nm  (:wat::core::ast-name node)]
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
        (:wat::core::Tuple off nm ":-")))
    (:wat::fix::empty-edits)))

(:wat::core::defn :user::node-edits
  [node           <- :wat::WatAST
   prev-rete-var? <- :wat::core::bool
   lines          <- (:wat::core::Vector :- [:wat::core::String])
   src            <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) false lines src)
    (:user::leaf-edits node prev-rete-var? lines src)))

(:wat::core::defn :user::seq-edits
  [items          <- (:wat::core::Vector :- [:wat::WatAST])
   prev-rete-var? <- :wat::core::bool
   lines          <- (:wat::core::Vector :- [:wat::core::String])
   src            <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::fix::empty-edits)
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::concat
        (:user::node-edits h prev-rete-var? lines src)
        (:user::seq-edits tl (:wat::fix::carry-rete-var? h prev-rete-var?) lines src)))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines (:wat::string::split src "\n")
                    tree  (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms (:wat::core::ast->children tree)
                    edits (:user::seq-edits forms false lines src)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse edits))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[rete-bind-arrow-to-binder] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
