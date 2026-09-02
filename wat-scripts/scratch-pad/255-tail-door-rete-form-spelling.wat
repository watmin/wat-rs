;; 255-tail-door-rete-form-spelling.wat — arc 255 Stone the-tail-door placement probe.
;;
;; THE QUESTION THIS ANSWERS: with the `eval_tail` registry guard in place (`src/runtime.rs`,
;; right after the rete `Form` re-mapping and right before `match head {`), does a rete
;; `Form`-class SPELLING of a form whose literal `eval_tail` arm this stone deleted (`if`) still
;; reach its tail impl (`eval_if_tail`, via `IntrinsicEntry::tail_handler`) and keep TCO?
;;
;; THE TRAP THE STONE NAMES: a guard placed ABOVE the re-mapping would consult the registry with
;; the UNMAPPED head (`:wat::rete::core::if`, not `:wat::core::if`), find no entry, and silently
;; fall through to `eval_inner` — which is STILL CORRECT, just not tail-optimized. That failure
;; is invisible to every ordinary test: the answer is right, only the stack cost is wrong, and it
;; only shows up at a depth nobody hand-writes into a unit test. This probe supplies that depth.
;;
;; SPELLING SOURCE: `src/rete/vocabulary.rs`'s `RETE_OPS` —
;;   rete_name: ":wat::rete::core::if", core_name: ":wat::core::if", class: OpClass::Form
;; — an actual `OpClass::Form` row, not an invented spelling.
;;
;; `if` was chosen (over `let`/`match`, the other two arms this stone deletes) only because it is
;; the shortest to write as a countdown; the guard's placement is a single gate shared by all
;; three, so this one spelling exercises the same code path all three depend on.

;; Tail-recursive countdown whose tail form is the RETE spelling of `if`. If the guard sits where
;; this stone requires, `head` has already been rewritten to `:wat::core::if` by the time the
;; registry is consulted, `eval_if_tail` is reached, and this survives any depth. If the guard
;; were mis-placed (above the re-mapping), the unmapped `:wat::rete::core::if` would miss the
;; registry lookup and fall through to `eval_inner` — still correct, but recursing on the native
;; Rust stack, which a shallow run cannot distinguish from success.
(:wat::core::defn :probe::countdown-rete-if [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::rete::core::if (:wat::i64::<= n 0)
    0
    (:probe::countdown-rete-if (:wat::i64::- n 1))))

;; The CONTROL for depth: the same recursion at a depth nothing could need a trampoline for. If
;; the deep case below fails while this passes, depth is the variable being measured, not a typo.
(:wat::core::defn :probe::shallow [] -> :wat::core::i64
  (:probe::countdown-rete-if 10))

;; THE DEEP CASE — chosen well past any native stack frame budget. With the guard correctly
;; placed (this stone's shipped state), this returns 0 without growing the Rust call stack. A
;; mis-placed guard (STOP-1) would SIGSEGV here instead of erroring, exactly as the pre-tail-door
;; `and`/`or` case did in `probe-s5-tail-position-is-load-bearing.wat`.
(:wat::core::defn :probe::deep [] -> :wat::core::i64
  (:probe::countdown-rete-if 200000))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::shallow))
    (:wat::kernel::println (:probe::deep))))
