;; PROBE — STONE: `StepValue` faces `WatAST`; the holon round-trip is lossy.
;;
;; Part 1 reproduces (BEFORE the fix) `try_recognize_holon_value`'s two SURPRISE arms
;; (`src/holon/ast.rs:928`) corrupting a RationalLit / BigIntLit into a StringLit on the
;; way through `eval-step!`'s AlreadyTerminal path (an ALREADY-a-value quoted literal).
;;
;; Part 2 covers the sibling defect the orchestrator's correction identified: the FIRED-
;; redex path (`step_descend_then_fire` / `step_holon_descend_then_fire`) routed its
;; result through `value_to_holon` (Value -> HolonAST) before lifting back to WatAST via
;; `holon_to_watast`. `value_to_holon` has no arm for `Value::wat__core__Rational` /
;; `Value::wat__core__BigInt` (HolonAST has no such leaf, by design) — so a REDEX whose
;; value is a rational or bigint refused with TypeMismatch instead of corrupting. Fixed
;; by routing `step_descend_then_fire`/`step_holon_descend_then_fire` through
;; `value_to_watast` (Value -> WatAST) directly, which now has exact Rational/BigInt arms.
;;
;; Part 2's redex only reaches `step_descend_then_fire`'s fire branch at all because
;; `is_step_canonical` (runtime.rs) was widened to admit RationalLit/BigIntLit as
;; canonical arguments — a separate, adjacent, pre-existing gap this stone's brief never
;; named. Before that widening, `(:wat::core::+ 1/2 1/3)` through eval-step! returned an
;; unchanged `StepNext` forever (the descend loop kept "stepping" the already-terminal
;; rational operand, got the same value back, and made no progress) — it never reached
;; `eval_inner`/`value_to_watast` at all. Flagged, not hidden: this widening was NOT in
;; the orchestrator's explicit brief for this stone.
;;
;; Part 3 covers the `:wat::core::fn` arm: it used to lower a bare `(fn ...)` form into
;; `HolonAST::Atom` (the VSA algebra's QUOTE, not a generic box) and back, so `eval-step!`
;; returned `(:wat::holon::Atom (fn ...))` — a form it never received. A bare `fn` form is
;; already its own canonical value; it is now returned unchanged.
;;
;; i64 is the control throughout — it must render identically before and after every fix.
;;
;; Run: `target/release/wat wat-scripts/scratch-pad/255-stepvalue-watast-roundtrip-probe.wat`

(:wat::core::defn :user::show [label <- :wat::core::String form <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::match (:wat::eval-step! form)
    ((:wat::core::Ok step)
      (:wat::core::match step
        ((:wat::eval::StepResult::AlreadyTerminal v)
          (:wat::core::do (:wat::kernel::println label) (:wat::kernel::println "  AlreadyTerminal ->") (:wat::kernel::println v)))
        ((:wat::eval::StepResult::StepTerminal v)
          (:wat::core::do (:wat::kernel::println label) (:wat::kernel::println "  StepTerminal ->") (:wat::kernel::println v)))
        ((:wat::eval::StepResult::StepNext v)
          (:wat::core::do (:wat::kernel::println label) (:wat::kernel::println "  StepNext ->") (:wat::kernel::println v)))))
    ((:wat::core::Err e)
      (:wat::core::do (:wat::kernel::println label) (:wat::kernel::println "  ERR ->") (:wat::kernel::println e)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "== PART 1: AlreadyTerminal (input already a value-shape literal) ==")
    (:wat::kernel::println "quote rational renders:")
    (:wat::kernel::println (:wat::core::quote 1/2))
    (:wat::kernel::println "quote bigint renders:")
    (:wat::kernel::println (:wat::core::quote 123456789012345678901234567890N))
    (:user::show "rational (AlreadyTerminal)" (:wat::core::quote 1/2))
    (:user::show "bigint   (AlreadyTerminal)" (:wat::core::quote 123456789012345678901234567890N))
    (:user::show "i64 CTL  (AlreadyTerminal)" (:wat::core::quote 42))

    (:wat::kernel::println "")
    (:wat::kernel::println "== PART 2: StepTerminal (a FIRED redex whose value is rational/bigint) ==")
    (:user::show "rational redex (1/2 + 1/3)" (:wat::core::quote (:wat::core::+ 1/2 1/3)))
    (:user::show "bigint redex   (2N * 3N)  " (:wat::core::quote (:wat::core::* 2N 3N)))
    (:user::show "i64 CTL redex  (1 + 2)    " (:wat::core::quote (:wat::core::+ 1 2)))

    (:wat::kernel::println "")
    (:wat::kernel::println "== PART 3: a bare fn form is its own canonical value, unchanged ==")
    (:user::show "fn literal" (:wat::core::quote (:wat::core::fn [x] x)))))
