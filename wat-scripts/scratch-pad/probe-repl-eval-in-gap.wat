;; probe-repl-eval-in-gap.wat — the RED gate for the REPL's one missing primitive.
;;
;; THE GAP, in one line: a running wat program can BUILD a definition (a form is just data,
;; and `WatAST` is pure by nature) but it cannot EVALUATE anything against a world built from
;; that definition. `:wat::eval-ast!` evaluates against the CALLER'S ambient symbol table
;; (`runtime.rs:22688` → `run_constrained(ast, env, sym)`), and `sym` is `&SymbolTable` —
;; immutable, and not a parameter the caller may supply. So an accumulated definition set is
;; inert: you can hold it, ship it, print it, and never run in it.
;;
;; This is the whole of what stands between the substrate and a REPL. Everything else the
;; REPL needs is already on the disk:
;;   - the world-from-forms builder — `startup_from_forms_with_inherit` (`freeze.rs:931`),
;;     deliberately main-free, already called from inside a running runtime by
;;     `run_forms_as_server_child` (`process/verbs.rs:354`)
;;   - the two-part state — `run_constrained(ast, env, sym)` already takes the live
;;     `Environment` SEPARATELY from the symbol table, which is exactly `:ephemeral` vs `:durable`
;;   - the classifier — the freeze's own partition, measured in
;;     `tests/program/probe_arc170_repl_freeze_partition.rs`
;;
;; STEP 1 and STEP 2 below are GREEN at HEAD and prove the surrounding ground is clean:
;; forms can be read and held. STEP 3 is the RED — it is the only line that fails, and it
;; fails on exactly the gap.
;;
;; ★ THE STONE LANDED (2026-07-28). `:wat::eval-with-defs!` closes exactly this gap, and
;; STEP 3 below now demonstrates it: the same expression, given the definition set, returns
;; `#wat.eval.FormOutcome/Evaluated [7]`. The line kept beside it — the ORIGINAL `eval-ast!`
;; call — still fails, and is kept ON PURPOSE: it is the before/after in one run, and it
;; keeps honest the claim that the ambient-eval could never have done this.
;;
;; RUN: target/release/wat wat-scripts/scratch-pad/probe-repl-eval-in-gap.wat
;;   STEP 3a (eval-ast!, the ambient world)  → Err `unknown function: :usr::f`
;;   STEP 3b (eval-with-defs!, the supplied world) → Evaluated [7]

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; STEP 1 — a REPL turn's worth of input: one definition line, held as pure data.
     ;; `read-string` returns the ONE wrapping form; `ast->children` is the def vector.
     defs (:wat::core::ast->children
            (:wat::core::match (:wat::core::read-string "(:wat::core::defn :usr::f [] -> :wat::core::i64 7)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))

     ;; STEP 2 — and a second line that CALLS it. Also just data.
     expr (:wat::core::first (:wat::core::match (:wat::core::read-string "(:usr::f)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))]

    (:wat::core::do
      ;; GREEN — the definition set is real, held, and countable.
      (:wat::kernel::println "STEP-1-defs-held")
      (:wat::kernel::println (:wat::core::length defs))

      ;; GREEN — the expression is a form.
      (:wat::kernel::println "STEP-2-expr-held")
      (:wat::kernel::println (:wat::core::ast-kind expr))

      ;; STEP 3a — the AMBIENT eval still cannot see the definition, and never could:
      ;; `eval-ast!` reaches `run_constrained(ast, env, sym)` with the call site's own
      ;; `sym`, and `defs` is a vector of forms, not a world.
      (:wat::kernel::println "STEP-3a-ambient-eval")
      (:wat::kernel::println (:wat::eval-ast! expr))

      ;; STEP 3b — the same expression, with the world supplied. This is the stone.
      (:wat::kernel::println "STEP-3b-eval-with-defs")
      (:wat::kernel::println (:wat::eval-with-defs! expr defs)))))
