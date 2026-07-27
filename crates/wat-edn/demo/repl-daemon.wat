;; wat REPL daemon — read-eval-print-RECUR over stdin/stdout (the R9 reactor, the R1 river).
;;
;; Protocol: each stdin line is an EDN string whose TEXT is a wat expression's source, e.g.
;;   "(:wat::core::+ 1 2)"
;; Response (one EDN value per line on stdout):
;;   #wat.core.Result/Ok [<value>]                   — the expression evaluated
;;   #wat.core.Result/Err [#wat.core/EvalError {…}]  — it did not (a typed error, as data)
;;
;; The loop is TCO-proper self-invocation — NO `loop`, NO `recur` (TVA RECVRSIO, TVVS REDITVS):
;; block on readln, compute one reply, println it, invoke yourself to listen again.
;; When the client closes stdin, readln ends the session (the process exits) — the honest stop.
;;
;;   run:  target/release/wat crates/wat-edn/demo/repl-daemon.wat   (stdin/stdout are the wire)

(:wat::core::defn :repl::serve [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println
      (:wat::eval-ast!
        (:wat::core::first
          (:wat::core::match (:wat::core::read-string
            (:wat::kernel::readln )) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))
    (:repl::serve)))                                          ; tail-recur → listen again

(:wat::core::defn :user::main [] -> :wat::core::nil (:repl::serve))
