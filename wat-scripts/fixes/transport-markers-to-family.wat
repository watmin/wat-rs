;; wat-scripts/fixes/transport-markers-to-family.wat — arc 255 Stone 255.25 (C-b4),
;; "the markers are a Transport".
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; The two phantom transport markers were declared as two EMPTY `defstruct`s in
;; `wat/spawn.wat` — and a struct reads IMPURE, so a generated child main could not spell its
;; own transport. They become the variants of ONE closed `Pure` family:
;;
;;   (:wat::core::defstruct :wat::kernel::Shared [])  =>  (deleted, with its line)
;;   (:wat::core::defstruct :wat::kernel::Wire [])    =>  (:wat::core::defenum :wat::kernel::Transport :wat::enum::Pure
;;                                                          :Shared []
;;                                                          :Wire [])
;;
;; and every REFERENCE is respelled, whole-token:
;;
;;   :wat::kernel::Shared    =>  :wat::kernel::Transport.Shared     (keyword leaf)
;;   :wat::kernel::Wire      =>  :wat::kernel::Transport.Wire       (keyword leaf)
;;   ":wat::kernel::Shared"  =>  ":wat::kernel::Transport.Shared"   (string leaf — a keyword-node name)
;;   ":wat::kernel::Wire"    =>  ":wat::kernel::Transport.Wire"     (string leaf — a keyword-node name)
;;
;; The string rows exist for `wat/service.wat`'s `handle-shared-tp-syms`/`handle-wire-tp-syms`,
;; which BUILD the marker keyword with `(:wat::core::keyword-node "…")`: the text is a name
;; that the macro emits as a keyword, so it is a reference, not prose. Only a string whose
;; WHOLE content equals the old name moves.
;;
;; ⭐ The declaration rewrite runs FIRST (on the source text), the renames after it (each
;; re-reads the rewritten text), so the declared name is replaced, never renamed.
;; ⭐ Whole-name equality (`rename-keyword-exact`): `:wat::kernel::SharedPeer`-style siblings,
;; the variant keywords `:Shared`/`:Wire` inside the new `defenum`, and the new spellings
;; themselves are not equal to an old name — IDEMPOTENT BY CONSTRUCTION.
;; ⛔ COMMENTS ARE NOT REWRITTEN (a comment is not a node). `wat/spawn.wat`'s marker comment
;; and prose elsewhere are updated by hand in the same commit.
;; ⛔ NOT in the path list: `wat-scripts/fixes/*.wat` (recorded migrations — a tool is never
;; its own input; `unstamp-transport-wire.wat` and `address-transport-arity.wat` SEARCH for
;; the old spelling, and moving it would falsify their record) and their replay fixtures.
;;
;; Comment-faithful: span splices through `fix-text-apply` (each edit carries its old text).
;;
;; Usage (ONE EDN vector of EVERY path on stdin):
;;   printf '["wat/spawn.wat" "wat/service.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/transport-markers-to-family.wat

(:wat::core::defn :user::no-edits [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :user::one-edit
  [off <- :wat::core::i64 old <- :wat::core::String new <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple off old new)))

;; declares? — a top-level `(:wat::core::defstruct <nm> [])` form whose name is exactly `nm`.
(:wat::core::defn :user::declares?
  [form <- :wat::WatAST nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? form ":wat::core::defstruct")
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
        (:wat::core::= (:wat::fix::kw-name (:wat::core::Option/expect (:wat::core::get ch 1) "defstruct name")) nm)
        false))
    false))

;; decl-edits — the declaration rewrite over the top-level forms. Each edit's old text is the
;; rule's BELIEF (the exact declaration text), checked by `fix-text-apply` before it splices.
(:wat::core::defn :user::decl-edits
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:user::no-edits)
    (:wat::core::let [h    (:wat::core::first forms)
                      tl   (:wat::core::rest forms)
                      here (:wat::core::if (:user::declares? h ":wat::kernel::Shared")
                             (:user::one-edit (:wat::fix::node-start-offset h lines)
                               "(:wat::core::defstruct :wat::kernel::Shared [])\n" "")
                             (:wat::core::if (:user::declares? h ":wat::kernel::Wire")
                               (:user::one-edit (:wat::fix::node-start-offset h lines)
                                 "(:wat::core::defstruct :wat::kernel::Wire [])"
                                 "(:wat::core::defenum :wat::kernel::Transport :wat::enum::Pure\n  :Shared []\n  :Wire [])")
                               (:user::no-edits)))]
      (:wat::core::concat here (:user::decl-edits tl lines)))))

;; string-edits — every STRING leaf whose whole content equals `old`: one edit replacing the
;; quoted literal. The span of a string leaf covers its quotes; its ast-name does not.
(:wat::core::defn :user::string-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   old   <- :wat::core::String
   new   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:user::no-edits)
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)
                      here (:wat::core::if (:wat::fix::structural? h)
                             (:user::string-edits (:wat::core::ast->children h) old new lines)
                             (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "string")
                                                (:wat::core::= (:wat::core::ast-name h) old)
                                                false)
                               (:user::one-edit (:wat::fix::node-start-offset h lines)
                                 (:wat::string::concat "\"" old "\"")
                                 (:wat::string::concat "\"" new "\""))
                               (:user::no-edits)))]
      (:wat::core::concat here (:user::string-edits tl old new lines)))))

;; forms-of — the top-level forms of `src` (a malformed file RAISES; never a silent skip).
(:wat::core::defn :user::forms-of
  [src <- :wat::core::String] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::ast->children
    (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])))

;; pass 1 — the declaration rewrite.
(:wat::core::defn :user::rewrite-decls
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines (:wat::string::split src "\n")]
    (:wat::fix::fix-text-apply src
      (:wat::core::reverse (:user::decl-edits (:user::forms-of src) lines)))))

;; pass 2 — ONE string-leaf row per pass (each pass re-reads, so its edits come out in one
;; ascending walk and `reverse` applies them high-offset-first).
(:wat::core::defn :user::rewrite-strings
  [old <- :wat::core::String new <- :wat::core::String src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines (:wat::string::split src "\n")]
    (:wat::fix::fix-text-apply src
      (:wat::core::reverse (:user::string-edits (:user::forms-of src) old new lines)))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [strung (:user::rewrite-strings ":wat::kernel::Wire" ":wat::kernel::Transport.Wire"
              (:user::rewrite-strings ":wat::kernel::Shared" ":wat::kernel::Transport.Shared"
                (:user::rewrite-decls src)))]
    (:wat::fix::rename-keyword-exact ":wat::kernel::Shared" ":wat::kernel::Transport.Shared"
      (:wat::fix::rename-keyword-exact ":wat::kernel::Wire" ":wat::kernel::Transport.Wire"
        strung))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[transport-family] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
