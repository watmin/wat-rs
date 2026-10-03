;; Co-located fixtures for probe_excursus003_b1_gb3_wrapper_sites_are_typed.rs.
;; Each fn drives one of the four wrapper sites (excursus 003 strike B1, item 5) and
;; RETURNS the decoded cause `Value` directly — no EDN-text round trip needed, so the
;; Rust harness inspects the real `Value::Aggregate` and its `.class` straight off
;; `call_beside_value`.

;; Site 1 — `check_failed_cause` (src/runtime.rs), via `:wat::eval-with-defs!`'s
;; per-line REPL check path (`:CheckFailed [cause <- Error]`).
(:wat::core::defn :user::check-failed-repl [] -> :wat::core::Value
  (:wat::core::match (:wat::eval-with-defs! (:wat::core::quote (:wat::core::+ "x" 1)) [])
    [:wat::eval::FormOutcome.CheckFailed {:cause __c} __c]
    [_ (:wat::kernel::assertion-failed! :message "expected CheckFailed")]))

;; Site 2 — the second `CheckFailed` producer (`eval_form_against_defs`'s baseline-freeze
;; arm, src/runtime.rs): the ACCUMULATED defs themselves fail to freeze (unresolved
;; reference inside `:user::b1-bad-def`), before the new form is even considered.
(:wat::core::defn :user::check-failed-second-producer [] -> :wat::core::Value
  (:wat::core::match
    (:wat::eval-with-defs!
      (:wat::core::quote (:wat::core::+ 1 1))
      (:wat::core::Vector :- [:wat::WatAST]
        (:wat::core::quote
          (:wat::core::defn :user::b1-bad-def [] -> :wat::core::nil (:user::b1-totally-unknown-fn)))))
    [:wat::eval::FormOutcome.CheckFailed {:cause __c} __c]
    [_ (:wat::kernel::assertion-failed! :message "expected CheckFailed (baseline defs failure)")]))

;; Site 3 — `read_outcome_malformed` (src/edn/render.rs), via `:wat::core::read-string`
;; (`ReadOutcome::Malformed [cause <- Error]`) on genuinely malformed wat source (a real
;; `ParseErrorKind`, declared per S3 — strict decode must succeed).
(:wat::core::defn :user::read-string-malformed [] -> :wat::core::Value
  (:wat::core::match (:wat::core::read-string "(")
    [:wat::core::ReadOutcome.Forms {:forms __f} (:wat::kernel::assertion-failed! :message "expected Malformed")]
    [:wat::core::ReadOutcome.Malformed {:cause __c} __c]))

;; Site 4 — `tagged_read_outcome_malformed` (src/edn/render.rs), via `:wat::edn::read-json`
;; on genuinely malformed JSON text (`ReadJsonOutcome::Malformed [cause <- Error]`). The
;; text never named a tag at all, so there is no diagnostic-specific class behind it (unlike
;; sites 1-3) — but strike B3 item 2 routes it through the declared `:wat::edn::ReadError`
;; catch-all (`EdnReadErrorKind::Other`), not a `:wat::core::Fault` stand-in.
(:wat::core::defn :user::read-json-malformed [] -> :wat::core::Value
  (:wat::core::match (:wat::edn::read-json "{not json")
    [:wat::edn::ReadJsonOutcome.Value {:value __v} (:wat::kernel::assertion-failed! :message "expected Malformed")]
    [:wat::edn::ReadJsonOutcome.Malformed {:cause __c} __c]))
