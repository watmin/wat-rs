;; ⛔ BLOCKED — THIS CODEMOD IS A SILENT NO-OP. It is correct in shape and cannot run.
;;
;; Moved out of `wat-scripts/fixes/` deliberately: that directory holds RECORDED, WORKING
;; migrations, and a tool that reports success while changing nothing does not belong beside them.
;; It is kept because when the gap below closes, this is exactly the file that ships.
;;
;; THE GAP — `rename-keyword-prefix` cannot move an OPEN NAMESPACE PREFIX.
;; `wat/fix.wat:641`'s `rename-valid-match?` says so in its own doc comment:
;;
;;     right-valid: i+old-len==len(name) OR char-at(name,i+old-len) ∉ ident-chars
;;
;; A prefix ending in `::` is BY DEFINITION followed by identifier characters — the verb name —
;; so `right-valid` can never hold. Measured:
;;
;;     (rename-keyword-prefix ":wat::core::string::" ":wat::string::" "(:wat::core::string::length \"hi\")")
;;       => "(:wat::core::string::length \"hi\")"        UNCHANGED
;;     (rename-keyword-prefix ":wat::kernel::Bound" ":wat::spawn::Bound" "(:wat::kernel::Bound/listener x)")
;;       => "(:wat::spawn::Bound/listener x)"           renamed
;;
;; The rule is right for a CLOSED name (`:wat::kernel::Bound` is always followed by `/accessor`,
;; `::Variant`, or nothing) — which is every prior art in `wat-scripts/fixes/`. None of them ever
;; renamed a `::`-terminated prefix, so the gap has never been exercised.
;;
;; It reports `[renamed]` on every file and changes zero bytes. A rider dry-ran it across 1559
;; files and `cmp` found NOT ONE byte different — which is the only reason this was caught before
;; a corpus-wide run "succeeded".
;;
;; ─────────────────────────────────────────────────────────────────────────────────────────

;; wat-scripts/fixes/rename-core-string-to-string.wat — arc 255 Stone E, the string home.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Moves the string verbs off the `core::` junk-drawer to their honest home, mirroring the
;; rete DSL clone the same way (it is a restricted clone of wat's language, not a separate
;; vocabulary — DESIGN-STONE-E-the-string-home.md, "the rete mirror MOVES"):
;;   :wat::core::string::*        -> :wat::string::*
;;   :wat::rete::core::string::*  -> :wat::rete::string::*
;;
;; TWO separate full-name PREFIX renames, rete FIRST. `:wat::core::string::` is not a prefix
;; of `:wat::rete::core::string::` (the `rete::` segment sits between `:wat::` and `core::`
;; there), so the two prefixes are disjoint sets of leaves and order is irrelevant — this is
;; re-verified by dry-run diff, not trusted on the strength of this comment.
;;
;; The prefix is the FULL name including trailing `::`, exactly per rename-kernel-to-spawn.wat's
;; discipline: `:wat::core::String` (capital S, the TYPE) and `:wat::core::string::` (lowercase,
;; trailing `::`) share the parent `:wat::core::`. The trailing colons are what keep this rename
;; off the type.
;;
;; Usage (one EDN vector of paths on stdin):
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | cargo wat ./wat-scripts/fixes/rename-core-string-to-string.wat
;;
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes — the old
;; prefixes are gone), so it is safe to run over a clean tree, including over itself: its own
;; verb CALLS (`:wat::core::string::concat` in apply-each's println line) migrate along with
;; everything else; its STRING LITERAL arguments (the two prefixes above) do not, because
;; rename-prefix-edits rewrites keyword leaves, and a string literal is not a keyword leaf.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::rete::core::string::" ":wat::rete::string::"
    (:wat::fix::rename-keyword-prefix ":wat::core::string::" ":wat::string::"
      src)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
