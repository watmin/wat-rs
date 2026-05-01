;; wat-scripts/dispatch.wat — EDN-stdin RPC dispatcher (arc 103c).
;;
;; Reads one `:demo::Job` EDN line from stdin, reads the named
;; query program's source via `:wat::io::read-file`, spawns it via
;; `:wat::kernel::spawn-program` with the db-path written as the
;; inner's single stdin line, forwards the inner's stdout to the
;; dispatcher's own stdout, joins.
;;
;; Usage:
;;   # First, seed a fixture database (one-time setup):
;;   echo /tmp/demo.db | ./target/release/wat ./wat-scripts/seed-fixture.wat
;;
;;   # Then dispatch — pick db-path + query-program at the EDN line:
;;   echo '#demo/Job {:db-path "/tmp/demo.db" :query-program "./wat-scripts/count-logs.wat"}' \
;;     | ./target/release/wat ./wat-scripts/dispatch.wat
;;
;; Demonstrates the hologram-nesting pattern from arc 103a:
;;   - dispatcher.wat (outer hologram) reads the job, picks the
;;     inner program, mediates IO
;;   - count-logs.wat (inner hologram) runs in its own frozen world
;;     with the dispatcher's db-path piped in as stdin
;;   - bytes flow only through three OS pipes; neither side can
;;     reach into the other's bindings
;;   - same EDN+newline protocol the shell uses to talk to wat
;;
;; See docs/arc/2026/04/103-kernel-spawn/HOLOGRAM.md for the framing.

(:wat::core::struct :demo::Job
  (db-path       :String)
  (query-program :String))


;; Tail-recursive byte pump. Reads each line from the inner's
;; stdout, forwards to our stdout, recurses. EOF (`:None`)
;; terminates the loop. Constant Rust stack regardless of message
;; count (arc 003 TCO).
(:wat::core::define
  (:demo::dispatch::pump
    (in  :wat::io::IOReader)
    (out :wat::io::IOWriter)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line in) -> :()
    (:wat::core::None ())
    ((:wat::core::Some line)
     (:wat::core::let*
       (((_ :()) (:wat::io::IOWriter/println out line)))
       (:demo::dispatch::pump in out)))))


;; Run one job. Reads the program source, spawns it, writes the
;; db-path to its stdin, closes that pipe (so the inner's first
;; read-line returns the path then `:None`), forwards the inner's
;; stdout, joins the inner's thread.
;; Returns Result so spawn-program failures (arc 105a) propagate
;; through `:wat::core::try` to :user::main, which writes the
;; StartupError message to stderr instead of panicking.
(:wat::core::define
  (:demo::dispatch::run
    (job    :demo::Job)
    (stdout :wat::io::IOWriter)
    -> :Result<(),wat::kernel::StartupError>)
  (:wat::core::let*
    (((db-path :String) (:demo::Job/db-path job))
     ((qp      :String) (:demo::Job/query-program job))
     ((src     :String) (:wat::io::read-file qp))
     ((proc    :wat::kernel::Process<(),()>)
      (:wat::core::try (:wat::kernel::spawn-program src :wat::core::None)))
     ((in-w    :wat::io::IOWriter)             (:wat::kernel::Process/stdin proc))
     ((_w      :i64)                           (:wat::io::IOWriter/write-string in-w db-path))
     ((_close  :())                            (:wat::io::IOWriter/close in-w))
     ((out-r   :wat::io::IOReader)             (:wat::kernel::Process/stdout proc))
     ((_pump   :())                            (:demo::dispatch::pump out-r stdout))
     ((_join   :())
      (:wat::core::match (:wat::kernel::Process/join-result proc) -> :()
        ((Ok _) ())
        ((Err _died)
         (:wat::core::panic! "dispatch: child died unexpectedly")))))
    (Ok ())))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    (:wat::core::None
      (:wat::io::IOWriter/println stderr
        "dispatch: expected a #demo/Job EDN line on stdin"))
    ((:wat::core::Some line)
     (:wat::core::let*
       (((job    :demo::Job)                            (:wat::edn::read line))
        ((result :Result<(),wat::kernel::StartupError>) (:demo::dispatch::run job stdout)))
       (:wat::core::match result -> :()
         ((Ok _) ())
         ((Err err)
          (:wat::io::IOWriter/println stderr
            (:wat::core::string::concat
              "dispatch: spawn failed: "
              (:wat::kernel::StartupError/message err)))))))))
