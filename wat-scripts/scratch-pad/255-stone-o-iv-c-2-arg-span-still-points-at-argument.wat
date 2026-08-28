;; wat-scripts/scratch-pad/255-stone-o-iv-c-2-arg-span-still-points-at-argument.wat — arc 255
;; Stone O-iv-c-2, acceptance row 4. `:wat::holon::leaf` is ARG-SPAN — its TypeMismatch
;; locates at `v.span()` (the ARGUMENT's own WatAST span), not the enclosing call's
;; span — which is exactly why it stays SHELL/refused rather than becoming ALGEBRA
;; (`Value` carries no span for `apply`'s value door to hand it).
;;
;; Two call sites below, `span-a` and `span-b`, each pass a non-primitive HolonAST
;; (itself a `leaf` call) to an outer `leaf`, which raises `TypeMismatch` at the INNER
;; form's own span. The outer call and the inner argument sit at different
;; columns/lines in both cases (the inner argument is indented onto its own line,
;; well past the outer call's own opening paren) — if the reported `:location` names
;; the inner argument's line/col (not the outer call's, and not the same line/col in
;; both cases), that is the fidelity refusing this verb protects.
;;
;; UNCAUGHT (not wrapped in `:wat::eval-ast!`/`apply`) so the process crashes and the
;; printed `RuntimeError` carries its full `:location` — the caught `EvalError` a wat
;; program sees via `eval-ast!`/`match` never exposes location (only `:kind`/
;; `:message`), per O-iv-c-1's rider's method (`255-stone-o-iv-c-1-span-still-points-at-caller.wat`).
;;
;; Run with `./target/release/wat <this file> <case>`, case in {span-a, span-b}.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     case (:wat::core::Option/expect (:wat::core::get argv 2) "usage: <this file> <case>")]
    (:wat::core::cond
      ((:wat::core::= case "span-a")
       (:wat::core::do
         (:wat::holon::leaf
                                              (:wat::holon::leaf "boom-a"))
         nil))

      ((:wat::core::= case "span-b")
       (:wat::core::do
         (:wat::holon::leaf
           (:wat::holon::leaf "boom-b"))
         nil))

      (:else
       (:wat::kernel::println (:wat::string::concat "unknown case: " case))))))
