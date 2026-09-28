;; wat-scripts/fixes/drop-retired-locidiederror-arms.wat — excursus 003 strike A (F7).
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; F7 retires two `:wat::kernel::LociDiedError` variants that never had a producer
;; (`EntryFormFailure`) or whose only producer is rerouted to `Panic` (`BadReturn`,
;; BRIEF-shape-strike-A-one-death-shape.md). Once `wat/kernel/diagnostics.wat`'s
;; `defenum` drops both variants, any `(:wat::core::match cause [...])` arm still
;; naming `:wat::kernel::LociDiedError.EntryFormFailure` or `.BadReturn` refuses to
;; type-check (an unknown enum variant tag) — six corpus files carry such an arm.
;;
;; This walks EVERY node in the tree (not just top-level forms — the offending
;; matches sit inside `defn` bodies), and for every `(:wat::core::match ...)` node
;; found, deletes each arm whose pattern head is one of the two retired variants.
;; Deletion covers ONLY the arm's own span (mirrors `drop-deftest-prelude.wat`'s
;; precedent): the residual blank line is wat-fmt's job, never eat a neighboring
;; comment or the next arm by guessing at whitespace boundaries.
;;
;; Idempotent: once an arm is gone, `retired-arm?` never matches it again on a
;; second run — 0 edits.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/drop-retired-locidiederror-arms.wat

;; arm-head-name — a match arm's pattern-tag name, e.g. ":wat::kernel::LociDiedError.BadReturn".
;; NOT `:wat::fix::arm-head-name` (that helper assumes an arm lowers to a "list" node; measured
;; directly against this reader: a `[PATTERN-KEYWORD {bindings} BODY]` arm is a "vector" node
;; whose FIRST child is the bare pattern keyword — no list wrapping at all).
(:wat::core::defn :user::arm-head-name [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children arm)]
    (:wat::core::if (:wat::core::empty? ch)
      ""
      (:wat::core::let [head (:wat::core::first ch)]
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
          (:wat::core::ast-name head)
          "")))))

;; retired-arm? — this match arm's pattern head is one of F7's two retired variants.
(:wat::core::defn :user::retired-arm? [arm <- :wat::WatAST] -> :wat::core::bool
  (:wat::fix::str-in? (:user::arm-head-name arm)
    (:wat::core::Vector :- [:wat::core::String]
      ":wat::kernel::LociDiedError.EntryFormFailure"
      ":wat::kernel::LociDiedError.BadReturn")))

;; match-node? — a `(:wat::core::match ...)` call.
(:wat::core::defn :user::match-node? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name node) ":wat::core::match"))

;; arm-edits — one deletion edit per retired arm in `arms` (ascending offset order).
(:wat::core::defn :user::arm-edits
  [arms  <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? arms)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::let [arm  (:wat::core::first arms)
                       tl   (:wat::core::rest arms)
                       rest (:user::arm-edits tl src lines)]
      (:wat::core::if (:user::retired-arm? arm)
        (:wat::core::concat
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
            (:wat::core::Tuple
              (:wat::fix::fix-text-offset-of (:wat::core::ast-span arm) lines)
              (:wat::fix::fix-text-span-text (:wat::core::ast-span arm) (:wat::core::ast-end-span arm) lines src)
              ""))
          rest)
        rest))))

;; node-edits — 0+ deletion edits for retired arms directly under THIS node, when it's a match.
(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::match-node? node)
    ;; children = [head-keyword scrutinee arm...]; drop the first two to reach the arms.
    (:user::arm-edits (:wat::core::into [] (:wat::core::drop (:wat::core::ast->children node) 2)) src lines)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))

;; walk / walk-children — full-tree recursive scan (ast->children is [] on a leaf, so
;; this terminates and touches every nested form, not just top-level ones).
(:wat::core::defn :user::walk
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::concat
    (:user::node-edits node src lines)
    (:user::walk-children (:wat::core::ast->children node) src lines)))

(:wat::core::defn :user::walk-children
  [nodes <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? nodes)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::walk (:wat::core::first nodes) src lines)
      (:user::walk-children (:wat::core::rest nodes) src lines))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src)
                                 [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
                                 [:wat::core::ReadOutcome.Malformed {:cause __cause}
                                   (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::walk-children forms src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[drop-retired-locidiederror-arms] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
