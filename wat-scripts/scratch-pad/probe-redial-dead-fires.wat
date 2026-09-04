;; probe-redial-dead-fires.wat — EXPECTATIONS row 2.
;;
;; Point a redial at a genuinely dead service. The wall must still fire:
;; "peer is dead, not a broken pipe". A recovery path that cannot be made to
;; fail has not been demonstrated.

(:wat::config::set-redef! true)
(:wat::load-file! "probe-frame-cap-severs-one-conn.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h    (:fc::echo/start :locus (:wat::spawn::process) :record (:fc::echo::Record))
     addr (:fc::echo::Handle/addr h)
     _    (:fc::echo/stop h)]
    (:wat::core::match (:wat::kernel::connect addr)
      ((:wat::kernel::ConnectOutcome::Connected _p)
        (:wat::kernel::assertion-failed! "row2: dead echo still accepted a connect" :wat::core::None :wat::core::None))
      (_ (:wat::kernel::assertion-failed! "queue: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))))
