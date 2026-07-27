;; wat-scripts/demos/repl/repl.wat — the wat REPL, correct-but-slow (arc 170).
;;
;; THE SHAPE. A REPL is a loop over ONE state — the user's definition set — and one verb:
;;
;;     read   →  a line of stdin, `read-string`d into a form
;;     eval   →  `(:wat::eval-with-defs! form defs)` — the form, in a world built from defs
;;     print  →  the outcome, as EDN (it already IS data; nothing formats it)
;;     loop   →  tail-invoke with defs GROWN if the line was a declaration, unchanged if not
;;
;; The state split the whole design rests on:
;;   :durable    `defs` — a `Vector<WatAST>`. Forms are PURE by nature (a tree of keywords
;;               and literals holds no fd and no peer), so the definition set is data that
;;               ships, hibernates, and replays.
;;   :ephemeral  the live `Environment` — never rebuilt, and it does not need to be, because
;;               `run_constrained(ast, env, sym)` has always taken `env` separately from the
;;               symbol table. A bound service survives every re-freeze for free.
;;
;; WHY IT IS SLOW ON PURPOSE. Every turn re-derives the entire world from `defs`. That is
;; the R1/R9 dual-impl discipline: this is the correct-but-slow ORACLE. Its correctness is
;; not argued, it is structural — the turn runs the ORDINARY program pipeline, so this REPL
;; is exactly as strongly typed as a compiled program. The fast incremental data plane gets
;; built later, behind a differential against this. Never delete the oracle.
;;
;; WHY THERE IS NO `declaration?` PREDICATE. There cannot be an honest one at this layer.
;; `defn` and `defrecord` fail eval with `unknown-function` — byte-identical to a TYPO —
;; because both are macros with no runtime verb to find (measured:
;; wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat). The substrate classifies
;; instead, on the POST-EXPANSION residue, and hands back a named outcome. A consequence
;; worth having: a user's OWN macro that expands to a `def` is classified correctly, and
;; this file never learns it exists.
;;
;;   run:  target/release/wat wat-scripts/demos/repl/repl.wat
;;   then: type a form per line —
;;           (:wat::core::defn :usr::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* n 2))
;;           (:usr::double 21)
;;
;; KNOWN, and NOT papered over: EOF on stdin raises a `LociDiedError/Panic` cascade rather
;; than stopping cleanly. That is a real substrate gap (`readln` has no matchable EOF), it
;; is filed, and it is NOT worked around here — a `;; the honest stop` comment over a crash
;; is exactly the lie this arc exists to kill. Ctrl-D ends the session loudly, for now.

(:wat::core::defn :repl::turn
  [defs <- :wat::core::Vector<wat::WatAST>]
  -> :wat::core::nil
  (:wat::core::let
    [form (:wat::core::first (:wat::core::read-string (:wat::kernel::readln )))]
    (:wat::core::match (:wat::eval-with-defs! form defs)

      ;; A DECLARATION joined the world. Nothing to show — but the definition set grows,
      ;; and THIS is the only arm that grows it.
      ;; (a UNIT variant matches BARE — the inner parens are for tagged variants only)
      (:wat::eval::FormOutcome::Declared
        (:repl::turn (:wat::core::conj defs form)))

      ;; An EXPRESSION produced a value. The world is unchanged.
      ((:wat::eval::FormOutcome::Evaluated v)
        (:wat::core::do
          (:wat::kernel::println v)
          (:repl::turn defs)))

      ;; It did not type-check in this world. Nothing ran; the session is untouched.
      ;; `cause` is a navigable error TREE, not prose — `:causes` down to a real `:span`.
      ((:wat::eval::FormOutcome::CheckFailed cause)
        (:wat::core::do
          (:wat::kernel::println cause)
          (:repl::turn defs)))

      ;; It type-checked, ran, and unwound. Also non-fatal: one bad line does not end a
      ;; session, which is the whole reason a REPL's failures must be VALUES.
      ((:wat::eval::FormOutcome::Raised cause)
        (:wat::core::do
          (:wat::kernel::println cause)
          (:repl::turn defs))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:repl::turn (:wat::core::Vector :wat::WatAST)))
