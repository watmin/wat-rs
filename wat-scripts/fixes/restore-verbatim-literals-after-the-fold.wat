;; wat-scripts/fixes/restore-verbatim-literals-after-the-fold.wat — arc 294 stone 0a, codemod B.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Inverse of `fmt-head-fqdn-to-clojure.wat`, driven by an explicit table. Commit `0b5742cc7`
;; rewrote 45 STRING literals inside five grep-rules codemods from verbatim FQDN
;; (`:wat::core::Uuid/`) to the folded spelling (`wat.core.Uuid/`). Those literals are the
;; names the rules compare against (and, for exact-match rules, the replacement they emit).
;; After the fold they no longer equal the source token, so `fix-text-apply` refuses.
;;
;; Rewrite STRING nodes (never comments) whose value equals a folded key. Span-faithful,
;; idempotent. Never run over this file: the table IS the folded keys.
;;
;; SCOPE: wat-scripts/fixes/rename-four-families-to-their-homes.wat wat-scripts/fixes/rename-core-vectors-to-their-homes.wat wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat wat-scripts/fixes/rename-string-verbs-to-their-home.wat wat-scripts/fixes/rename-math-stat-seq-to-their-homes.wat
;;
;; Usage:
;;   printf '["wat-scripts/fixes/rename-four-families-to-their-homes.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/restore-verbatim-literals-after-the-fold.wat

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

;; THE TABLE — 45 (folded → verbatim) pairs, zipped from `git show 0b5742cc7` on the five
;; files (minus=verbatim, plus=folded, keep the pairs that differ). Each folded key occurs
;; exactly once in today's file.
(:wat::core::defn :user::pairs []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])]
    (:wat::core::Tuple "wat.core.Uuid/" ":wat::core::Uuid/")
    (:wat::core::Tuple "wat.core.List/of" ":wat::core::List/of")
    (:wat::core::Tuple "wat.core/List" ":wat::core::List")
    (:wat::core::Tuple "wat.core.char/of" ":wat::core::char/of")
    (:wat::core::Tuple "wat.core/char" ":wat::core::char")
    (:wat::core::Tuple "wat.core.PersistentVector/" ":wat::core::PersistentVector/")
    (:wat::core::Tuple "wat.core.Vector/" ":wat::core::Vector/")
    (:wat::core::Tuple "wat.rete.core.PersistentVector/length" ":wat::rete::core::PersistentVector/length")
    (:wat::core::Tuple "wat.rete.vector/length" ":wat::rete::vector::length")
    (:wat::core::Tuple "wat.rete.core.PersistentVector/contains?" ":wat::rete::core::PersistentVector/contains?")
    (:wat::core::Tuple "wat.rete.vector/contains?" ":wat::rete::vector::contains?")
    (:wat::core::Tuple "wat.rete.core.PersistentVector/get" ":wat::rete::core::PersistentVector/get")
    (:wat::core::Tuple "wat.rete.vector/get" ":wat::rete::vector::get")
    (:wat::core::Tuple "wat.rete.core.Vector/get" ":wat::rete::core::Vector/get")
    (:wat::core::Tuple "wat.rete.vec/get" ":wat::rete::vec::get")
    (:wat::core::Tuple "wat.core.HashSet/" ":wat::core::HashSet/")
    (:wat::core::Tuple "wat.core.List/" ":wat::core::List/")
    (:wat::core::Tuple "wat.rete.core.List/get" ":wat::rete::core::List/get")
    (:wat::core::Tuple "wat.rete.linkedlist/get" ":wat::rete::linkedlist::get")
    (:wat::core::Tuple "wat.core.String/concat" ":wat::core::String/concat")
    (:wat::core::Tuple "wat.string/concat" ":wat::string::concat")
    (:wat::core::Tuple "wat.core.String/starts-with?" ":wat::core::String/starts-with?")
    (:wat::core::Tuple "wat.string/starts-with?" ":wat::string::starts-with?")
    (:wat::core::Tuple "wat.core.String/ends-with?" ":wat::core::String/ends-with?")
    (:wat::core::Tuple "wat.string/ends-with?" ":wat::string::ends-with?")
    (:wat::core::Tuple "wat.core.String/contains?" ":wat::core::String/contains?")
    (:wat::core::Tuple "wat.string/contains?" ":wat::string::contains?")
    (:wat::core::Tuple "wat.core.String/empty?" ":wat::core::String/empty?")
    (:wat::core::Tuple "wat.string/empty?" ":wat::string::empty?")
    (:wat::core::Tuple "wat.rete.core.String/concat" ":wat::rete::core::String/concat")
    (:wat::core::Tuple "wat.rete.string/concat" ":wat::rete::string::concat")
    (:wat::core::Tuple "wat.rete.core.String/starts-with?" ":wat::rete::core::String/starts-with?")
    (:wat::core::Tuple "wat.rete.string/starts-with?" ":wat::rete::string::starts-with?")
    (:wat::core::Tuple "wat.rete.core.String/ends-with?" ":wat::rete::core::String/ends-with?")
    (:wat::core::Tuple "wat.rete.string/ends-with?" ":wat::rete::string::ends-with?")
    (:wat::core::Tuple "wat.rete.core.String/contains?" ":wat::rete::core::String/contains?")
    (:wat::core::Tuple "wat.rete.string/contains?" ":wat::rete::string::contains?")
    (:wat::core::Tuple "wat.rete.core.String/empty?" ":wat::rete::core::String/empty?")
    (:wat::core::Tuple "wat.rete.string/empty?" ":wat::rete::string::empty?")
    (:wat::core::Tuple "wat.std.list/zip" ":wat::std::list::zip")
    (:wat::core::Tuple "wat.seq/zip" ":wat::seq::zip")
    (:wat::core::Tuple "wat.std.list/window" ":wat::std::list::window")
    (:wat::core::Tuple "wat.seq/window" ":wat::seq::window")
    (:wat::core::Tuple "wat.std.list/remove-at" ":wat::std::list::remove-at")
    (:wat::core::Tuple "wat.seq/remove-at" ":wat::seq::remove-at")))

(:wat::core::defn :user::no-verbatim [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::Option.None {}))

(:wat::core::defn :user::lookup
  [folded <- :wat::core::String
   pairs  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])]
  -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Option :- [:wat::core::String])
                     p   <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])]
      -> (:wat::core::Option :- [:wat::core::String])
      (:wat::core::match acc
        [:wat::core::Option.Some {:value __v} acc]
        [:wat::core::Option.None {}
          (:wat::core::if (:wat::core::= folded (:wat::core::first p))
            (:wat::core::Option.Some {:value (:wat::core::second p)})
            acc)]))
    (:user::no-verbatim)
    pairs))

(:wat::core::defn :user::string-edit
  [n     <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "string")
    (:wat::core::match (:user::lookup (:wat::core::ast-name n) (:user::pairs))
      [:wat::core::Option.None {} (:user::empty-edits)]
      [:wat::core::Option.Some {:value v}
        (:user::one-edit
          (:user::start-off n lines)
          (:wat::core::ast->source n)
          (:wat::string::concat "\"" (:wat::string::concat v "\"")))])
    (:user::empty-edits)))

(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                       c   <- :wat::WatAST]
        -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::concat acc (:user::node-edits c lines)))
      (:user::string-edit node lines)
      (:wat::core::ast->children node))
    (:user::string-edit node lines)))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string src)
    [:wat::core::ReadOutcome.Forms {:forms forms}
      (:wat::core::let
        [lines (:wat::string::split src "\n")
         kids  (:wat::core::ast->children forms)
         eds   (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                                  n   <- :wat::WatAST]
                   -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                   (:wat::core::concat acc (:user::node-edits n lines)))
                 (:user::empty-edits)
                 kids)
         rev   (:wat::core::reverse (:wat::core::sort eds))]
        (:wat::fix::fix-text-apply src rev))]
    [:wat::core::ReadOutcome.Malformed {:cause cause}
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
