;; wat-scripts/fixes/frame-symbol-span-to-fn-at.wat — excursus 003 strike D.
;; SCOPE: wat/service.wat wat/bracket.wat tests/kernel/probe_arc278_call_site.wat tests/macros/probe_arc278_macro_call_site.wat tests/process/wat_arc170_closure6_label_wall_labeled.wat tests/services/probe_arc278_emitted_from.wat tests/services/probe_arc278_log_captures_call_line.wat
;;
;; Self-hosted, comment-faithful fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; THE CHANGE (AUDIT-the-shape-of-an-error.md F6, RULING 2026-09-27 item 2): `:wat::kernel::Frame`
;; is reshaped from `{symbol span kind}` (pairing callee/call-site) to `{fn at tail-elided}`
;; (pairing function/where-inside-it, the Clojure/Java convention). `kind` is retired outright
;; (no replacement accessor — it was derivable from `span.end`); every prior
;; `(:wat::kernel::Frame/symbol X)` accessor call becomes `(:wat::kernel::Frame/fn X)`, and every
;; prior `(:wat::kernel::Frame/span X)` accessor call becomes `(:wat::kernel::Frame/at X)`. A pure
;; keyword-leaf RENAME (not a form flip, unlike the sibling `nest-frame-file-line-in-span.wat` —
;; no wrapping call is introduced), so this reuses `:wat::fix::rename-keyword-exact` directly
;; rather than a hand-rolled walk: measured, every occurrence in SCOPE is the call-form HEAD
;; (`ast-kind` "keyword"), never a string literal or a different keyword sharing the tail
;; (`:wat::kernel::Frame/symbol` / `/span` appear nowhere else in these 7 files — confirmed by a
;; full-corpus grep before writing this codemod — so the exact-name leaf match cannot misfire).
;;
;; IDEMPOTENCY: after one run, no keyword leaf's `ast-name` equals `:wat::kernel::Frame/symbol` or
;; `:wat::kernel::Frame/span` in SCOPE (every one now reads `/fn` or `/at`) — a re-run's scan
;; finds nothing and emits zero edits.
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat/service.wat" "wat/bracket.wat" \
;;            "tests/kernel/probe_arc278_call_site.wat" \
;;            "tests/macros/probe_arc278_macro_call_site.wat" \
;;            "tests/process/wat_arc170_closure6_label_wall_labeled.wat" \
;;            "tests/services/probe_arc278_emitted_from.wat" \
;;            "tests/services/probe_arc278_log_captures_call_line.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/frame-symbol-span-to-fn-at.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [after-symbol (:wat::fix::rename-keyword-exact
                                    ":wat::kernel::Frame/symbol" ":wat::kernel::Frame/fn" src)
                     after-span   (:wat::fix::rename-keyword-exact
                                    ":wat::kernel::Frame/span" ":wat::kernel::Frame/at" after-symbol)]
    after-span))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[frame-symbol-span-to-fn-at] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
