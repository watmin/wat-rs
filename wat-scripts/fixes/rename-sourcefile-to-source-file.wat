;; wat-scripts/fixes/rename-sourcefile-to-source-file.wat — the arc-283 source-unit lift,
;; run over real wat source files IN WAT, through the wat CLI. The migration tool, self-hosted:
;; no Rust harness, no hand-edit of wat source (use-the-tool, not hand-fix).
;;
;; Lifts the generic source-unit type out of the deporder junk-drawer to its honest neutral home
;; (intueri-named: `File`, since the `:wat::source` namespace already carries the "source code" domain):
;;   :wat::deporder::SourceFile -> :wat::source::File   (+ /path /source accessors AND <type-arg> uses)
;;
;; ONE full-name PREFIX rename. Catches every form because rename-keyword-prefix was hardened in
;; arc 283.1 to a boundary-aware whole-name rewrite: the head (`:wat::deporder::SourceFile`), the
;; accessors (`…SourceFile/path`), AND the type-argument occurrences (`:wat::core::Vector<wat::deporder::SourceFile>`
;; — the colon-stripped name embedded inside another keyword), while NEVER corrupting a prefix-sibling
;; (`:wat::deporder::SourceFileX`) or an unrelated path (`:other::…SourceFile`).
;;
;; Usage (one EDN vector of EVERY path holding the symbol on stdin — list them ALL; arc 283 learned
;; that a hand-listed subset which missed wat-tests/deporder.wat broke the build):
;;   printf '["wat/deporder.wat" "wat/lint.wat" "wat-tests/lint.wat" "wat-tests/deporder.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-sourcefile-to-source-file.wat
;;
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes — the old prefix is
;; gone). The def itself + its stdlib registration (wat/source.wat, before deporder) are the manual
;; seams the codemod cannot do; see docs/arc/2026/06/283-source-file-lift/DESIGN.md.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::deporder::SourceFile" ":wat::source::File" src))

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
