;; wat-scripts/lib/wat-grep.wat — general form-aware grep primitives for wat codemods.
;;
;; Provides two public verbs:
;;
;;   (:user::wat-grep src pred)       → (Vector :- [WatAST])
;;     Parse src, filter TOP-LEVEL forms with pred; return matching nodes.
;;
;;   (:user::wat-grep-strip src pred) → String
;;     Parse src, span-delete every TOP-LEVEL form satisfying pred; return rewritten src.
;;
;; Both operate ONLY on the direct children of the parse root — TOP-LEVEL forms.
;; Embedded forms inside (:wat::core::forms …) blocks are NOT walked, giving spawned-child
;; entrypoints (:user::main inside forms) protection for free.
;;
;; pred: [wat::WatAST :-> wat::core::bool]
;;   Passed directly to (:wat::core::filter …). Named defns and lambdas both work.
;;
;; (:user::wat-grep-strip …) uses the :wat::fix:: span-edit machinery from fix.wat (stdlib):
;;   fix-text-offset-of + fix-text-span-len + fix-text-apply (all :wat::fix:: stdlib verbs)
;; Collects edits in ascending offset order, reverses, applies right-to-left so offsets
;; remain stable. A trailing '\n' after the closing ')' is consumed so no dangling blank
;; lines remain after deletion.
;;
;; Namespace: :user:: (the only writable prefix for user scripts outside wat/ stdlib)
;; Loaded by: (:wat::load-file! "../lib/wat-grep.wat") from wat-scripts/fixes/*.wat

;; ── Internal: whole-form deletion edit ───────────────────────────────────────────────
;;
;; wat-grep-form-edit — a one-element Vector containing a deletion Tuple for `form`.
;; Covers ast-span(form) → ast-end-span(form) plus one trailing '\n' if present.
(:wat::core::defn :user::wat-grep-form-edit
  [form  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let [start-span (:wat::core::ast-span form)
                    end-span   (:wat::core::ast-end-span form)
                    off        (:wat::fix::fix-text-offset-of start-span lines)
                    old-len    (:wat::fix::fix-text-span-len start-span end-span lines)
                    ;; eat the trailing newline (if any) to avoid a dangling blank line
                    src-len    (:wat::core::string::length src)
                    end-off    (:wat::core::+ off old-len)
                    next-is-nl (:wat::core::if (:wat::core::< end-off src-len)
                                  (:wat::core::= (:wat::core::string::subs src end-off
                                                   (:wat::core::+ end-off 1)) "\n")
                                  false)
                    eat        (:wat::core::if next-is-nl 1 0)]
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
      (:wat::core::Tuple off (:wat::core::+ old-len eat) ""))))

;; ── Internal: map a vector of matched forms to deletion edits ─────────────────────────
(:wat::core::defn :user::wat-grep-strip-edits
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    (:wat::core::concat
      (:user::wat-grep-form-edit (:wat::core::first forms) src lines)
      (:user::wat-grep-strip-edits (:wat::core::rest forms) src lines))))

;; ── Public: find top-level matching forms ────────────────────────────────────────────
;;
;; wat-grep — parse src, return every top-level form satisfying pred.
;; pred signature: WatAST -> bool.
(:wat::core::defn :user::wat-grep
  [src  <- :wat::core::String
   pred <- :wat::core::Fn(wat::WatAST)->wat::core::bool]
  ;; Arc 118.2a — `filter` flipped LAZY; this fn's declared return type is `(Vector :- [WatAST])`, so `filterv`.
  -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::let [tree  (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms (:wat::core::ast->children tree)]
    (:wat::core::filterv pred forms)))

;; ── Public: delete top-level matching forms ───────────────────────────────────────────
;;
;; wat-grep-strip — parse src, collect deletion edits for every pred-matched top-level form,
;; apply them right-to-left. Returns the rewritten source string. Comment-faithful: only the
;; matched forms' character spans are deleted; everything else survives byte-identical.
(:wat::core::defn :user::wat-grep-strip
  [src  <- :wat::core::String
   pred <- :wat::core::Fn(wat::WatAST)->wat::core::bool]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    ;; Arc 118.2a — `filter` flipped LAZY; `matches` feeds `wat-grep-strip-edits`
                    ;; ((Vector :- [WatAST]) param), so `filterv`.
                    matches   (:wat::core::filterv pred forms)
                    all-edits (:user::wat-grep-strip-edits matches src lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))
