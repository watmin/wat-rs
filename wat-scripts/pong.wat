;; wat-scripts/pong.wat — the child responder.
;;
;; Reads `:demo::Ping {:n :i64}` lines from stdin, writes
;; `:demo::Pong {:n :i64}` to stdout for each (mirroring the n).
;; Tail-recursive — runs until stdin EOFs (parent closes the
;; pipe), at which point `:user::main` returns and the thread
;; exits cleanly.
;;
;; Usage: spawned by ping-pong.wat via :wat::kernel::spawn-program.
;; Not meant for direct shell invocation.

(:wat::core::struct :demo::Ping
  (n :i64))

(:wat::core::struct :demo::Pong
  (n :i64))


(:wat::core::define
  (:demo::pong::loop
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    (:wat::core::None ())
    ((:wat::core::Some line)
     (:wat::core::let*
       (((ping :demo::Ping) (:wat::edn::read line))
        ((n    :i64)         (:demo::Ping/n ping))
        ((pong :demo::Pong) (:demo::Pong/new n))
        ((_    :())
         (:wat::io::IOWriter/println stdout (:wat::edn::write pong))))
       (:demo::pong::loop stdin stdout)))))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:demo::pong::loop stdin stdout))
