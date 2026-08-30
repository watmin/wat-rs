;; wat-scripts/pong.wat — pipe-stage Ping→Pong responder.
;;
;; Reads `:demo::Ping {:n :i64}` frames from stdin, writes
;; `:demo::Pong {:n :i64}` to stdout for each (mirroring the n).
;; Tail-recursive — runs until stdin EOFs.
;;
;; Usage:
;;   echo '#demo/Ping {:n 3}' | cargo wat ./wat-scripts/pong.wat

(:wat::core::defstruct :demo::Ping
  [n <- :wat::core::i64])

(:wat::core::defstruct :demo::Pong
  [n <- :wat::core::i64])


(:wat::core::defn :demo::pong::loop [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln)
    (:wat::kernel::ReadlnOutcome::Eof nil)
    (:wat::kernel::ReadlnOutcome::Stopped nil)
    ((:wat::kernel::ReadlnOutcome::Datum ping)
      (:wat::core::do
        (:wat::kernel::println (:demo::Pong :n (:demo::Ping/n ping)))
        (:demo::pong::loop)))))


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:demo::pong::loop))
