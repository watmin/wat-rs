;; wat-scripts/fixes/eval-error-to-failure.wat — excursus 003 strike E.
;; SCOPE: tests/value/wat_eval_result.wat tests/value/wat_eval_result_wrong_arity.wat tests/types/probe_arc293_struct_to_form_roundtrip.wat tests/types/probe_stone_binder_is_universal.wat tests/types/probe_stone_guard_the_peel_point.wat tests/rete/probe_arc278_59_tco_and_or_ann_form.wat tests/types/probe_stone_guard_the_peel_point_row1_too_many.wat.bad tests/types/probe_stone_guard_the_peel_point_row2_too_many.wat.bad
;;
;; Self-hosted, comment-faithful fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; THE CHANGE (AUDIT-the-shape-of-an-error.md F4, D2; RULING 2026-09-27 item 3;
;; BRIEF-shape-strike-E-eval-carries-the-real-error.md): the eval family's `Err` moves from
;; the flattened `:wat::core::EvalError {kind message}` to the real `:wat::kernel::Failure`
;; the dynamic eval raised (the declared record class, its fields, and its captured frames —
;; see `runtime_error_failure`, src/runtime.rs). Three rewrites, each a whole-token or
;; whole-call replace:
;;
;;   1. `:wat::core::EvalError` (a bare type-path annotation, e.g. inside
;;      `(:wat::core::Result :- [T :wat::core::EvalError])`) -> `:wat::kernel::Failure`.
;;      Pure keyword-leaf RENAME — reuses `:wat::fix::rename-keyword-exact` directly, same
;;      shape as the sibling `frame-symbol-span-to-fn-at.wat`.
;;
;;   2. `(:wat::core::EvalError/message X)` -> `(:wat::kernel::Failure/message X)`. Same
;;      accessor shape (`Failure/message` is a DERIVED accessor reading `error.message`,
;;      wat/kernel/diagnostics.wat) — pure keyword-leaf RENAME of the head, reusing
;;      `rename-keyword-exact` again with a second old/new pair.
;;
;;   3. `(:wat::core::EvalError/kind X)` -> `(:wat::core::type (:wat::kernel::Failure/error X))`.
;;      A FORM FLIP, not a rename — item 4's class-branching decision: wat has no enum-variant
;;      tag to match on here (the 40 RuntimeErrorKind records are 40 distinct declared record
;;      TYPES, not variants of one shared enum), so the honest existing mechanism is
;;      `:wat::core::type` (arc 234/255, Total, Pure — extracts ANY value's declared record-type
;;      FQDN as a String), applied to the Failure's own `error` field. Same wrap-edits shape as
;;      the sibling `nest-frame-file-line-in-span.wat`: the whole matched call's text is
;;      replaced; the argument text is sliced between the HEAD keyword's end and the whole
;;      call's own end (not the arg node's own span — the sibling's measured defect with
;;      `~`-unquote-sugar arguments). SCOPE's one call site (`tests/value/wat_eval_result.wat`,
;;      both uses) passes a plain bound symbol, not unquote sugar, but the codemod uses the
;;      safe endpoint-slicing shape regardless, for the same reason the sibling does: it is
;;      correct for both cases and no simpler for only one.
;;
;; ORDER: the three passes run in sequence against the LATEST text each time, re-parsing
;; between passes. Pass 1 (the `/kind` flip) runs FIRST — it inserts a brand-new
;; `:wat::kernel::Failure/error` keyword, which pass 2's `/message` rename and pass 3's bare
;; type-path rename must not re-touch (they are each a DIFFERENT exact keyword string, so
;; there is no overlap either way, but running the flip first keeps each pass's precondition —
;; "no other matching text yet" — simple to state).
;;
;; IDEMPOTENCY: after one run, no node's full keyword name is `:wat::core::EvalError`,
;; `:wat::core::EvalError/kind` or `:wat::core::EvalError/message` anywhere in SCOPE — a
;; re-run's three scans each find nothing and emit zero edits.
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["tests/value/wat_eval_result.wat" \
;;            "tests/value/wat_eval_result_wrong_arity.wat" \
;;            "tests/types/probe_arc293_struct_to_form_roundtrip.wat" \
;;            "tests/types/probe_stone_binder_is_universal.wat" \
;;            "tests/types/probe_stone_guard_the_peel_point.wat" \
;;            "tests/rete/probe_arc278_59_tco_and_or_ann_form.wat" \
;;            "tests/types/probe_stone_guard_the_peel_point_row1_too_many.wat.bad" \
;;            "tests/types/probe_stone_guard_the_peel_point_row2_too_many.wat.bad"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/eval-error-to-failure.wat

;; ── pass 1: the `/kind` form flip ───────────────────────────────────────────────────────────

(:wat::core::defn :user::kind-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::EvalError/kind")
    (:wat::core::= (:wat::core::length (:wat::core::ast->children node)) 2)
    false))

(:wat::core::defn :user::kind-wrap-edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::kind-wrap-edits (:wat::core::first items) src lines)
      (:user::kind-wrap-edits-walk (:wat::core::rest items) src lines))))

(:wat::core::defn :user::kind-wrap-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::kind-call? node)
    (:wat::core::let [ch           (:wat::core::ast->children node)
                      head         (:wat::core::Option/expect (:wat::core::get ch 0) "EvalError/kind head")
                      ;; arg-text sliced between the HEAD's end and the whole call's own end
                      ;; (minus the closing paren) — not via the arg node's own span (the
                      ;; measured `~`-unquote-sugar defect `nest-frame-file-line-in-span.wat`
                      ;; records; irrelevant to this corpus's plain-symbol args but the safe
                      ;; shape regardless).
                      head-end-off (:wat::fix::node-end-offset head lines)
                      node-end-off (:wat::fix::node-end-offset node lines)
                      arg-text     (:wat::string::trim
                                     (:wat::string::subs src head-end-off (:wat::core::- node-end-off 1)))
                      off      (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                      old-text (:wat::fix::fix-text-span-text
                                 (:wat::core::ast-span node) (:wat::core::ast-end-span node) lines src)
                      new-text (:wat::string::concat "(:wat::core::type (:wat::kernel::Failure/error "
                                 (:wat::string::concat arg-text "))"))]
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
        (:wat::core::Tuple off old-text new-text)))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::kind-wrap-edits-walk (:wat::core::ast->children node) src lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::flip-kind [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::kind-wrap-edits-walk forms src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

;; ── passes 2 and 3: the two whole-keyword renames ───────────────────────────────────────────

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [after-kind    (:user::flip-kind src)
                    after-message (:wat::fix::rename-keyword-exact
                                     ":wat::core::EvalError/message" ":wat::kernel::Failure/message" after-kind)
                    after-type    (:wat::fix::rename-keyword-exact
                                     ":wat::core::EvalError" ":wat::kernel::Failure" after-message)]
    after-type))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[eval-error-to-failure] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
