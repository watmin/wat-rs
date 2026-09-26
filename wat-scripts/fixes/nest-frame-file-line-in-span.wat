;; wat-scripts/fixes/nest-frame-file-line-in-span.wat — excursus 003, envelope step 2 (D3).
;; SCOPE: wat/bracket.wat wat/service.wat tests/kernel/probe_arc278_call_site.wat tests/services/probe_arc278_emitted_from.wat tests/services/probe_arc278_log_captures_call_line.wat tests/process/wat_arc170_closure6_label_wall_labeled.wat tests/macros/probe_arc278_macro_call_site.wat
;;
;; Self-hosted, comment-faithful fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; THE CHANGE (DESIGN-the-error-envelope-and-its-frames.md D3, ruled 2026-09-24; BRIEF
;; envelope-step-2): `:wat::kernel::Frame` moves its location off two flat fields
;; (`file`/`line`) onto a nested `:wat::core::Span` (`span`), and gains `kind`. Every prior
;; `(:wat::kernel::Frame/file X)` accessor call becomes
;; `(:wat::core::Span/file (:wat::kernel::Frame/span X))`, and every prior
;; `(:wat::kernel::Frame/line X)` accessor call becomes
;; `(:wat::core::Span/line (:wat::kernel::Frame/span X))`. `Frame/symbol` is
;; UNCHANGED (still a direct String field) — not touched by this codemod.
;;
;; This is a FORM FLIP, not a rename: `rename-keyword-prefix` (the D1 vehicle) swaps a
;; keyword's spelling in place and cannot introduce a wrapping call, so this codemod is new
;; rather than a copy of `retire-kernel-location-to-core-span.wat`.
;;
;; SPEC: a recursive structural scan (mirrors `:wat::fix::rename-prefix-edits`'s
;; `structural?`/`ast->children` walk). At each node: if it is a List whose head is EXACTLY
;; `:wat::kernel::Frame/file` or `:wat::kernel::Frame/line` with EXACTLY one argument, the
;; WHOLE node's span is replaced (`fix-text-span-text`, verbatim old-text — the node's own
;; identity is the belief being asserted, not a name) with
;; `(:wat::core::Span/<field> (:wat::kernel::Frame/span <arg-text>))`, where `<arg-text>` is
;; the source text between the HEAD keyword's end and the whole call's own end (trimmed) —
;; NOT the argument node's own span. MEASURED: a `~`-unquote-sugar argument (wat/service.wat's
;; `~origin-sym`, inside a quasiquote template) reports ast-kind "list" but an ast-span/
;; ast-end-span covering only the one-char `~` sigil, not the symbol it wraps — slicing the
;; arg's own span silently dropped it (`wat-scripts/scratch-pad/excursus-003/probe-unquote-
;; span.wat` records the probe). Deriving arg-text from the HEAD's and the whole CALL's
;; endpoints — both plain, non-sugar spans — sidesteps that defect and preserves whatever
;; expression sits there (bound symbol, nested call, or `~sym`) byte-identical. A matched
;; call's argument subtree is NOT independently recursed into: every argument in the SCOPE
;; above is a bare symbol, a nested call, or a `~`-unquoted symbol with no nested Frame
;; accessor of its own (confirmed by reading each site before writing this codemod), so the
;; simplification is exact for this corpus, not assumed in general — a future site nesting
;; one Frame accessor inside another's argument would need this codemod extended, and it
;; would silently not match today.
;;
;; IDEMPOTENCY: after one run, no node matches `frame-file-line-call?` (every accessor now
;; reads `Span/file`/`Span/line` off a nested `Frame/span` call, never `Frame/file`/
;; `Frame/line` directly) — a re-run's scan finds nothing and emits zero edits.
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat/bracket.wat" "wat/service.wat" \
;;            "tests/kernel/probe_arc278_call_site.wat" \
;;            "tests/services/probe_arc278_emitted_from.wat" \
;;            "tests/services/probe_arc278_log_captures_call_line.wat" \
;;            "tests/process/wat_arc170_closure6_label_wall_labeled.wat" \
;;            "tests/macros/probe_arc278_macro_call_site.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/nest-frame-file-line-in-span.wat

;; frame-accessor-field — "file" / "line" for a matched call node; "" otherwise.
(:wat::core::defn :user::frame-accessor-field [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::fix::calls-to? node ":wat::kernel::Frame/file") "file"
    (:wat::core::if (:wat::fix::calls-to? node ":wat::kernel::Frame/line") "line" "")))

;; frame-file-line-call? — a List headed EXACTLY `Frame/file` or `Frame/line`, one argument.
(:wat::core::defn :user::frame-file-line-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::if (:wat::core::not (:wat::core::= (:user::frame-accessor-field node) ""))
      (:wat::core::= (:wat::core::length (:wat::core::ast->children node)) 2)
      false)
    false))

;; wrap-edits-walk — recurse a vector of nodes, concating wrap edits (mirrors
;; :wat::fix::rename-prefix-edits-walk's shape).
(:wat::core::defn :user::wrap-edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::wrap-edits (:wat::core::first items) src lines)
      (:user::wrap-edits-walk (:wat::core::rest items) src lines))))

;; wrap-edits — 0-or-1 whole-node-replace edit at a match; else recurse into structural children.
(:wat::core::defn :user::wrap-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::frame-file-line-call? node)
    (:wat::core::let [ch           (:wat::core::ast->children node)
                      head         (:wat::core::Option/expect (:wat::core::get ch 0) "frame accessor head")
                      ;; arg-text is sliced between the HEAD's end and the whole call's own
                      ;; end (minus the closing paren) — NOT via the arg node's own span.
                      ;; MEASURED: a `~`-unquote-sugar arg (wat/service.wat's `~origin-sym`,
                      ;; inside a quasiquote template) reports ast-kind "list" but an
                      ;; ast-span/ast-end-span covering only the ONE-CHAR `~` sigil, not the
                      ;; symbol it wraps (`wat-scripts/scratch-pad/excursus-003/probe-unquote-
                      ;; span.wat` — a probe kept as the record of this finding). Slicing the
                      ;; ARG's own span silently dropped `origin-sym`, caught by this
                      ;; codemod's own dry-run diff before it ever touched the real corpus.
                      ;; The head keyword and the whole call form are both plain, non-sugar
                      ;; nodes whose spans are reliable; deriving arg-text from THEIR
                      ;; endpoints sidesteps the sugar-node span defect entirely.
                      head-end-off (:wat::fix::node-end-offset head lines)
                      node-end-off (:wat::fix::node-end-offset node lines)
                      arg-text     (:wat::string::trim
                                     (:wat::string::subs src head-end-off (:wat::core::- node-end-off 1)))
                      field    (:user::frame-accessor-field node)
                      off      (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                      old-text (:wat::fix::fix-text-span-text
                                 (:wat::core::ast-span node) (:wat::core::ast-end-span node) lines src)
                      new-text (:wat::string::concat "(:wat::core::Span/"
                                 (:wat::string::concat field
                                   (:wat::string::concat " (:wat::kernel::Frame/span "
                                     (:wat::string::concat arg-text "))"))))]
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
        (:wat::core::Tuple off old-text new-text)))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::wrap-edits-walk (:wat::core::ast->children node) src lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::wrap-edits-walk forms src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[nest-frame-file-line] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
