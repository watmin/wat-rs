;; wat-scripts/fixes/retire-kernel-location-to-core-span.wat — excursus 003, envelope step 1 (D1).
;; SCOPE: wat/core.wat wat/grep.wat wat/kernel/diagnostics.wat wat/rete/compile.wat tests/diagnostics/probe_arc296_error_surface.wat tests/diagnostics/probe_arc296_here.wat
;;
;; Self-hosted, comment-faithful fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; THE CHANGE (DESIGN-the-error-envelope-and-its-frames.md D1, ruled 2026-09-24; BRIEF
;; envelope-step-1): three records meant "a location" before this stone —
;; `:wat::core::Span {file line col end: (Option :- [Pos])}` and the narrower
;; `:wat::kernel::Location {file line col}` (no `end`). D1 retires `:wat::kernel::Location`
;; OUTRIGHT — no alias — in favour of `:wat::core::Span` everywhere a location appears. A
;; Rust-originated location becomes a `Span` with `end` `None`; a wat-originated one carries
;; `Some`. (`:wat::kernel::Frame` is the third shape named in the AUDIT; it changes in envelope
;; step 2, not here — untouched by this codemod.)
;;
;; SPEC: the standalone `(:wat::core::defrecord :wat::kernel::Location …)` declaration is deleted
;; from the file that declares it (its own field shape now lives on `:wat::core::Span`, already
;; declared elsewhere in the same file — keeping both would register the same class name twice
;; with two different shapes). Every remaining occurrence of the keyword `:wat::kernel::Location`
;; — bare (a field type, a param/return type) or as an accessor prefix
;; (`:wat::kernel::Location/line`, `/col`) — is renamed to `:wat::core::Span` (`Span/line`,
;; `Span/col`), via `:wat::fix::rename-keyword-prefix` (boundary-aware, comment-faithful:
;; comments/formatting/non-matching keywords survive byte-identical — prose is swept by hand
;; alongside this codemod, per `wat-rs/CLAUDE.md`'s scratch-`.wat` doctrine that comments are not
;; a rename tool's job).
;;
;; TWO PASSES, composed:
;;   1. `:user::delete-location-decl` — a top-level-form scan (mirrors
;;      `drop-deftest-prelude.wat`'s shape): the ONE form whose head is EXACTLY
;;      `:wat::core::defrecord` and whose declared name is EXACTLY `:wat::kernel::Location` has
;;      its own span deleted whole. Every other top-level form is left untouched by this pass
;;      (not even recursed into — a defrecord's own name slot is a DECLARATION, not a rename
;;      site, the same reasoning `rename-keyword-exact`'s defenum-variant carve-out documents).
;;   2. `:wat::fix::rename-keyword-prefix ":wat::kernel::Location" ":wat::core::Span"` over the
;;      result — the existing arc-283.1 vehicle, unmodified, handles every remaining use site:
;;      field types, param/return types, and the `/line` `/col` accessor forms (boundary-aware:
;;      `/` is a valid right-boundary character, so `:wat::kernel::Location/line` rewrites in
;;      one whole-name edit to `:wat::core::Span/line`, never touching `/line` itself).
;;
;; IDEMPOTENCY: after one run, no top-level form still declares `:wat::kernel::Location` (pass 1
;; matches nothing) and no keyword leaf still spells it (pass 2's `rename-keyword-prefix` already
;; guarantees idempotency for an exact prefix swap — a re-run finds `new-name == name` everywhere
;; and emits zero edits). Composing the two passes is idempotent because each pass alone is.
;;
;; ⛔ NOT RUN OVER `wat-scripts/scratch-pad/probe-arc296-builtin-redeclared-in-wat.wat` — that
;; file's entire SUBJECT is "does re-declaring `:wat::kernel::Location` in wat match the Rust
;; builtin exactly" (arc 296, before this stone). With Location retired there is no builtin left
;; to re-declare; mechanically renaming it would fabricate a THIRD-field `:wat::core::Span`
;; redeclaration that mismatches the real (four-field) one and would misreport a `DuplicateType`
;; the file never meant to probe. Deleted by hand instead (`git rm`) — CLAUDE.md's own scratch
;; rule: "delete it if it's truly dead".
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat/core.wat" "wat/grep.wat" "wat/kernel/diagnostics.wat" "wat/rete/compile.wat" \
;;            "tests/diagnostics/probe_arc296_error_surface.wat" \
;;            "tests/diagnostics/probe_arc296_here.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/retire-kernel-location-to-core-span.wat

;; ── pass 1: delete the standalone Location declaration ──────────────────────────────────────

;; location-decl? — a List headed EXACTLY `:wat::core::defrecord` whose SECOND child (the
;; type-name slot) is the keyword `:wat::kernel::Location` exactly. Field shape is irrelevant to
;; the match (there is exactly one such form in the corpus; matching on head+name, not fields,
;; keeps this codemod correct even if the field list drifted before this ran).
(:wat::core::defn :user::location-decl? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::defrecord")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        false
        (:wat::core::let [name (:wat::core::Option/expect (:wat::core::get ch 1) "defrecord name")]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind name) "keyword")
            (:wat::core::= (:wat::core::ast-name name) ":wat::kernel::Location")
            false))))
    false))

;; decl-form-edits — 0-or-1 deletion edit for one top-level form: the whole node's own span,
;; verbatim old-text (sanctioned by location-decl? already confirming the node's identity).
(:wat::core::defn :user::decl-form-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::location-decl? node)
    (:wat::core::let [off      (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                      old-text (:wat::fix::fix-text-span-text
                                 (:wat::core::ast-span node)
                                 (:wat::core::ast-end-span node)
                                 lines src)]
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
        (:wat::core::Tuple off old-text "")))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))

;; decl-scan — every top-level form, ascending offset order (mirrors drop-deftest-prelude.wat).
(:wat::core::defn :user::decl-scan
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::decl-form-edits (:wat::core::first forms) src lines)
      (:user::decl-scan (:wat::core::rest forms) src lines))))

(:wat::core::defn :user::delete-location-decl [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::decl-scan forms src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

;; ── entry point: pass 1 (delete), then pass 2 (rename-keyword-prefix, unmodified vehicle) ────

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::kernel::Location" ":wat::core::Span"
    (:user::delete-location-decl src)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[retire-location] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
