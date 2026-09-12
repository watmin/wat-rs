;; wat-scripts/fixes/break-kind-string-to-enum.wat — arc 277: Break.kind becomes an enum.
;; SCOPE: wat-scripts/fmt/rules/*.wat
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of rule files.
;;
;; Rewrites the 23 `:then` sites:
;;   (:wat::fmt::Break :id ?x :kind "block")
;;     -> (:wat::fmt::Break :id ?x :kind (:wat::fmt::BreakKind::Block))
;;   (:wat::fmt::Break :id ?x :kind "align")
;;     -> (:wat::fmt::Break :id ?x :kind (:wat::fmt::BreakKind::Align))
;;
;; SURGICAL: only a string child immediately after the `:kind` keyword of a
;; `:wat::fmt::Break` constructor. Node.kind string comparisons are untouched.
;;
;; Also rewrites the three identical header comments that named the old string
;; literals, so grep for `"block"`/`"align"` over rules/ is 0.
;;
;; Idempotent: after one run the `:kind` value is a list, not a string, and the
;; comment phrase is gone. A second run reports 0 changes.
;;
;; Usage:
;;   printf '["wat-scripts/fmt/rules/defn.wat" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/break-kind-string-to-enum.wat

(:wat::core::defn :user::empty-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :user::one-edit
  [off <- :wat::core::i64  old <- :wat::core::String  new <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple off old new)))

(:wat::core::defn :user::start-off
  [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n)
    ""))

(:wat::core::defn :user::kind-target [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= s "block")
    "(:wat::fmt::BreakKind::Block)"
    (:wat::core::if (:wat::core::= s "align")
      "(:wat::fmt::BreakKind::Align)"
      "")))

(:wat::core::defn :user::break-ctor-edits
  [ch    <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::i64::>= (:wat::i64::+ i 1) (:wat::core::length ch))
    (:user::empty-edits)
    (:wat::core::let
      [a (:wat::core::nth ch i)
       b (:wat::core::nth ch (:wat::i64::+ i 1))
       rest (:user::break-ctor-edits ch (:wat::i64::+ i 1) lines)]
      (:wat::core::if
        (:wat::core::if (:wat::core::= (:user::kw-name a) ":kind")
          (:wat::core::= (:wat::core::ast-kind b) "string")
          false)
        (:wat::core::let [tgt (:user::kind-target (:wat::core::ast-name b))]
          (:wat::core::if (:wat::string::empty? tgt)
            rest
            (:wat::core::concat
              (:user::one-edit (:user::start-off b lines) (:wat::core::ast->source b) tgt)
              rest)))
        rest))))

(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:user::empty-edits)
        (:wat::core::let
          [here (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::fmt::Break")
                   (:user::break-ctor-edits ch 0 lines)
                   (:user::empty-edits))]
          (:wat::core::concat here (:user::seq-edits ch lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines)
      (:user::empty-edits))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     it  <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:user::empty-edits)
    items))

(:wat::core::defn :user::comment-edits
  [comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   lines    <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     c   <- :wat::fmt::Comment]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::let [t (:wat::fmt::Comment/text c)]
        (:wat::core::if (:wat::string::contains? t "\"block\"")
          (:wat::core::concat acc
            (:user::one-edit
              (:wat::core::+
                (:wat::fix::fix-text-line-start (:wat::fmt::Comment/line c) lines)
                (:wat::core::- (:wat::fmt::Comment/col c) 1))
              t
              ";; Break names a kind (BreakKind); the emitter computes the rest."))
          acc)))
    (:user::empty-edits)
    comments))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string-with-comments src)
    [:wat::core::ReadWithCommentsOutcome.Forms {:forms forms :comments comments}
      (:wat::core::let
        [lines (:wat::string::split src "\n")
         kids  (:wat::core::ast->children forms)
         eds   (:wat::core::concat
                 (:user::seq-edits kids lines)
                 (:user::comment-edits comments lines))
         rev   (:wat::core::reverse (:wat::core::sort eds))]
        (:wat::fix::fix-text-apply src rev))]
    [:wat::core::ReadWithCommentsOutcome.Malformed {:cause cause}
      (:wat::kernel::assertion-failed! :message (:wat::core::Error/message cause))]))

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
        (:user::apply-each (:wat::core::rest paths)
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
