;; Scratch probe — arc 255 Stone "the option/result siblings get homes".
;;
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-option-result-siblings.md
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-option-result-siblings.md
;;
;; Homes the last three of the family: `:wat::core::Option/try`, `:wat::core::Result/expect`,
;; `:wat::core::Result/try`. The RULING (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-
;; partial.md`) splits them from their own bodies, not by family symmetry: `Result/expect`
;; `expect_panic`s on `Err` (a raise, `Partial`); both `try` verbs return a propagate SIGNAL
;; (`EvalSignal::TryPropagate`/`OptionPropagate`) that `apply_function` wraps into the enclosing
;; function's own matchable `Err`/`:None` return (`Total`) — two siblings sharing a namespace and
;; a naming convention with `expect`, ruled the OPPOSITE way because they do the opposite thing.
;;
;;   section 1 — behavior unchanged: both `try` verbs unwrap the happy path and short-circuit the
;;               unhappy one; `Result/expect` unwraps `Ok` (its panic-on-`Err` path is NOT
;;               exercised live here — the body did not move (STOP-4), so the raise mechanics are
;;               unchanged by construction; see `eval_result_expect`'s doc in `runtime.rs` and
;;               `Option/expect`'s existing probe, `255-probe-the-accessor-classifies-pure.wat`,
;;               which shows the same restraint for its own `expect` sibling)
;;   section 2 — `metadata-of :totality` shows the split: `Result/expect` => `Partial`, both
;;               `try` verbs => `Total`
;;
;; ⚠ Run against the PRE-EXISTING `target/release/wat` (predates this stone's Rust changes, per
;; the rider's brief) — expect section 1 to behave exactly as before (the dispatch arms this
;; stone deletes are still literal match arms in that binary) and section 2's `metadata-of` to
;; answer `:None` for all three (`registry().lookup_entry` finds nothing pre-rebuild; only a
;; rebuilt binary registers them and can answer `Some hm`). See the rider's report for what this
;; binary actually printed.

(:wat::core::defn :user::roundtrip-option [o <- (:wat::core::Option :- [:wat::core::i64])] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::Some (:wat::core::Option/try o)))

(:wat::core::defn :user::roundtrip-result [r <- (:wat::core::Result :- [:wat::core::i64 :wat::core::String])] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Ok (:wat::core::Result/try r)))

(:wat::core::defn :user::totality-of [name <- :wat::core::keyword] -> :wat::core::String
  (:wat::core::match (:wat::runtime::metadata-of name)
    ((:wat::core::Some hm)
     (:wat::core::match (:wat::hashmap::get hm :totality)
       ((:wat::core::Some t) (:wat::edn::write t))
       (:None "registered, but no :totality key (unexpected)")))
    (:None "None (not registered in this binary)")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── section 1 — behavior unchanged ──")
    (:wat::kernel::println (:wat::string::concat "Option/try (Some 3)  => " (:wat::edn::write (:user::roundtrip-option (:wat::core::Some 3)))))
    (:wat::kernel::println (:wat::string::concat "Option/try :None     => " (:wat::edn::write (:user::roundtrip-option :wat::core::None))))
    (:wat::kernel::println (:wat::string::concat "Result/try (Ok 3)    => " (:wat::edn::write (:user::roundtrip-result (:wat::core::Ok 3)))))
    (:wat::kernel::println (:wat::string::concat "Result/try (Err msg) => " (:wat::edn::write (:user::roundtrip-result (:wat::core::Err "boom")))))
    (:wat::kernel::println (:wat::string::concat "Result/expect (Ok 3) => " (:wat::edn::write (:wat::core::Result/expect (:wat::core::Ok 3) "unreachable"))))
    (:wat::kernel::println "── section 2 — metadata-of :totality (expect Partial vs. both try Total) ──")
    (:wat::kernel::println (:wat::string::concat "Result/expect :totality => " (:user::totality-of :wat::core::Result/expect)))
    (:wat::kernel::println (:wat::string::concat "Option/try    :totality => " (:user::totality-of :wat::core::Option/try)))
    (:wat::kernel::println (:wat::string::concat "Result/try    :totality => " (:user::totality-of :wat::core::Result/try)))
    nil))
