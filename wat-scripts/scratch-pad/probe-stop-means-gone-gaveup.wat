;; Row 4: ack arrived, lineage does not close → GaveUp, not a false Stopped.
;; A process child emits one message (the "ack") then parks on readln. First
;; owner-recv-loop sees Message. owner-wait-gone then times out: last=TimedOut.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-stop-means-gone-gaveup.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::do
                (:wat::kernel::println "acked")
                (:wat::core::match (:wat::kernel::readln)
                  ((:wat::kernel::ReadlnOutcome::Datum _) nil)
                  (:wat::kernel::ReadlnOutcome::Eof nil)
                  (:wat::kernel::ReadlnOutcome::Stopped nil))))))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     first (:wat::service::owner-recv-loop p t0 2000 "recv")
     second (:wat::service::owner-wait-gone p t0 500 "close")]
    (:wat::core::do
      (:wat::core::match first
        ((:wat::service::StopOutcome::Stopped _)
          (:wat::kernel::println "first=Stopped"))
        ((:wat::service::StopOutcome::Gone _)
          (:wat::kernel::println "first=Gone"))
        ((:wat::service::StopOutcome::GaveUp _w _l)
          (:wat::kernel::println "first=GaveUp")))
      (:wat::core::match second
        ((:wat::service::StopOutcome::Stopped _)
          (:wat::kernel::println "second=Stopped"))
        ((:wat::service::StopOutcome::Gone _)
          (:wat::kernel::println "second=Gone"))
        ((:wat::service::StopOutcome::GaveUp w l)
          (:wat::kernel::println
            (:wat::core::format "second=GaveUp waited={w} last={l}" :w w :l l)))))))
