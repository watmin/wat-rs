;; tests/function/wat_spawn_fn_not_callable.wat — NEGATIVE fixture: non-callable body.
;; 42 is neither a keyword path nor a fn value; spawn-thread expects Fn(Receiver<I>,Sender<O>)->().
;; startup MUST fail with TypeMismatch naming :wat::kernel::spawn-thread.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [not-fn 42
               thr
                (:wat::kernel::spawn-thread not-fn)]
              ()))
