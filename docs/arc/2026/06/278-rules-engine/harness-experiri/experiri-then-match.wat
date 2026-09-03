;; experiri-then-match.wat — the D5 repro (see README.md beside this file, "What it proves" §2).
;;
;; ⛔ ITS red-by-design DECLARATION IS RETIRED, 2026-09-02, by this file's own disposal condition.
;;   The header said: *"If this file ever loads, D5 is cured and the rune must go with it."* D5 is
;;   cured (arc 278 strike-match-arm-is-not-a-call) and this file loads and prints "loaded",
;;   exactly as its `where`-fence twin `experiri-when-match.wat` always did.
;;
;;   ⚠ THE LITERAL MARKER MAY NOT BE QUOTED ANYWHERE IN THIS FILE, not even inside backticks.
;;   `tests/lint/docs_wat_loads_or_declares_why_not.rs`'s reader finds its needle ANYWHERE on a
;;   line, so a quotation of the retired marker would re-exempt the file from the very load check
;;   that retiring it restores — a retirement that silently un-retires itself.
;;
;;   WHAT IT PROVED, recorded here because the declaration's reason was the only record of it: at
;;   HEAD `d10ae67c4` `validate/mod.rs`'s `walk_nested_constructors` could not tell a match ARM from
;;   a CALL. `(:probe::E::A true)` has an enum-variant keyword at `items[0]`, so the arity check
;;   fired the variant's 0 declared fields against the arm's length 1 and startup raised
;;   `RhsArityMismatch` naming a `:then` INSERT of `:probe::E::A` — an insert that appears nowhere
;;   in the source below. The byte-identical expression was accepted unchanged in the `where`
;;   fence, so whether a legal `match` compiled in `:then` depended on which of two equivalent
;;   spellings the author picked.
;;
;;   WHY RETIRING IT IS ITSELF THE REGRESSION GATE: a declaration EXEMPTS a file from that lint's
;;   load check — that is its whole function. Without one, this file is back under the check, so a
;;   walker that starts refusing arm patterns again reddens the lint suite.
;;   `tests/rete/probe_arc278_match_arm_is_not_a_call.rs` drives the pair as well, and names which
;;   file regressed and why.
;;
(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :probe::E])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String  ok <- :wat::core::bool])

;; IDENTICAL match expression, three positions. Uncomment one rule at a time.
(:wat::rete::defrule :probe::in-then
  :when  [(:probe::In (?k <- :k) (?v <- :v))]
  :then  [(:probe::Out :k ?k :ok (:wat::rete::core::match ?v (:probe::E::A true) (:probe::E::B false)))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "loaded"))
