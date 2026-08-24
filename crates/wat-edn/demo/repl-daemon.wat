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
;;
;; THE THREE EDGES ARE THE LOOP'S WHOLE CONTROL FLOW, and they are at the read, where the
;; read happens (arc 170 #24). `readln` returns `(:wat::kernel::ReadlnOutcome :- [T])`, so the
;; session's two exits are VALUES this loop faces, not raises that flee past it:
;;   Datum   — evaluate, reply, recur (the only arm that continues)
;;   Eof     — the client closed stdin; return nil, which ends the process. The honest stop.
;;   Stopped — a process-wide stop was requested; the same clean end, named distinctly so a
;;             reader can tell "the client hung up" from "we were told to stop".
;; Before #24 both of those RAISED: this file's header claimed "the honest stop" while the
;; run produced an `AssertionFailure` and exit 2. The claim is now true.
;;
;;   run:  target/release/wat crates/wat-edn/demo/repl-daemon.wat   (stdin/stdout are the wire)

(:wat::core::defn :repl::serve [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln )

    ;; a datum arrived — evaluate it, print the reply, listen again
    ((:wat::kernel::ReadlnOutcome::Datum src)
      (:wat::core::do
        (:wat::kernel::println
          (:wat::eval-ast!
            (:wat::core::first
              (:wat::core::match (:wat::core::read-string src)
                ((:wat::core::ReadOutcome::Forms __forms) __forms)
                ((:wat::core::ReadOutcome::Malformed __cause)
                  (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))
        (:repl::serve)))                                      ; tail-recur → listen again

    ;; the client closed stdin — end the session; returning nil ends the process
    (:wat::kernel::ReadlnOutcome::Eof     nil)

    ;; a stop was requested — the same clean end, distinctly named
    (:wat::kernel::ReadlnOutcome::Stopped nil)))

(:wat::core::defn :user::main [] -> :wat::core::nil (:repl::serve))
