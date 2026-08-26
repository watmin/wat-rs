;; probe-s5-tail-position-is-load-bearing.wat — arc 278 #56 (S5) reconnaissance.
;;
;; THE QUESTION THIS ANSWERS, and it decides the S5 brief's shape:
;;
;; `eval_tail` (runtime.rs:3807) intercepts exactly `:wat::core::{if, match, let, do}` (plus
;; `serve-dispatch-op`) and hands them to their `*_tail` TCO variants. It has NO rete gate — the
;; only rete gate in the runtime is at `dispatch_keyword_head_value` (runtime.rs:4486). THREE of
;; the four forms S5 mints (`if`, `let`, `match`) are exactly the forms `eval_tail` intercepts.
;;
;; So a `:wat::rete::core::if` in TAIL POSITION would miss `eval_if_tail` and fall through to the
;; ordinary evaluator. Before deciding whether that matters, MEASURE whether the TCO path is
;; load-bearing at all — i.e. whether a tail-recursive fn whose tail is `if` actually depends on
;; `eval_if_tail` to survive depth. If it does not, S5 needs no `eval_tail` change and the four
;; forms are cheaper than feared. If it does, S5 must gate `eval_tail` too, or a rete `if` in a
;; user's tail-recursive fn silently trades TCO for a stack overflow.
;;
;; `where` predicates do not self-recurse, so this is NOT about the fence. It is about the
;; one-directional wall: rete ops are ordinary wat functions, legal (if odd) outside a rule, so a
;; user CAN put one in a tail position — and the substrate must not blow up when they do.

;; Tail-recursive countdown whose tail form is `:wat::core::if`. If `eval_if_tail`'s TCO is real,
;; this survives any depth; without it, each level costs a Rust frame.
(:wat::core::defn :probe::countdown-if [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= n 0)
    0
    (:probe::countdown-if (:wat::i64::- n 1))))

;; The CONTROL for depth: the same recursion at a depth nothing could need a trampoline for.
;; If the deep case below fails while this passes, depth is the variable — not a typo in the fn.
(:wat::core::defn :probe::shallow [] -> :wat::core::i64
  (:probe::countdown-if 10))

;; THE DISCONFIRMING CONTROL. The identical recursion, but the last form is `and` — a FORM (lazy,
;; short-circuiting) that `eval_tail` does NOT intercept, so the recursive call under it takes the
;; ordinary evaluator. `and` is chosen deliberately: it is the exact class S5 mints, and
;; `:wat::rete::core::and` already SHIPPED in #55 — so whatever this measures is a property the
;; vocabulary already has, not a new risk S5 introduces.
;;
;; NOT `cond`: that is a defmacro expanding to `if`, so it would inherit `eval_if_tail` and prove
;; nothing (measured — it also rejects a non-`:else` terminal arm).
(:wat::core::defn :probe::countdown-and [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::i64::<= n 0)
    true
    (:wat::core::and true (:probe::countdown-and (:wat::i64::- n 1)))))

;; ── THE MEASUREMENT, taken 2026-08-02, `./target/release/wat` on this file ──────────────────
;;
;;   (:probe::shallow)              -> 0     ;; control: the recursion itself is sound
;;   (:probe::countdown-if 200000)  -> 0     ;; TCO via eval_if_tail — survives
;;   (:probe::countdown-and 200000) -> SIGSEGV, exit 139   ;; a FORM in the last position: no TCO
;;
;; VERDICT: `eval_tail`'s dispatch is load-bearing, and a Form in a recursive last position
;; overflows the native stack — as a SIGSEGV, not a located error. Two consequences:
;;
;;   1. FOR S5 — a `:wat::rete::core::if` minted without gating `eval_tail` would be a strictly
;;      WORSE `if` than its core twin: identical semantics, silently no TCO. `if`/`let`/`match`
;;      are exactly the forms `eval_tail` intercepts, so all three are affected. The gate is
;;      therefore part of the stone, not a nicety.
;;   2. NOT S5's, and tracked separately — this is a PRE-EXISTING substrate property, not
;;      something the rete surface introduces: plain `:wat::core::and` does it today, and
;;      `:wat::rete::core::and` (shipped #55) inherits it. A recursion that outruns the stack
;;      should raise a located error, not SIGSEGV. Do NOT fold that into S5.
;;
;; The deep `and` case is deliberately NOT driven from main: this file is loader-gated
;; (`every_wat_scripts_file_loads`), and a scratch probe that segfaults anyone who runs it is
;; hostile. Re-add the call below to reproduce.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::shallow))
    (:wat::kernel::println (:probe::countdown-if 200000))
    (:wat::kernel::println (:probe::countdown-and 10))))
