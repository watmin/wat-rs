;; Arc 170 migration: canonical [] -> :nil signatures;
;; IOWriter/println → (:wat::kernel::println ...);
;; demo::loop no longer needs a stdout param — println
;; routes through the ambient StdOutService.
;;
;; See tests/cli/wat_cli.rs::sigterm_to_cli_cascades_via_polling_contract:
;; prints READY once about to enter the polling loop (the test's lock-step
;; marker — it reads stdout until READY, THEN sends SIGTERM; no sleep, the
;; wire IS the synchronization), then polls (:wat::kernel::stopped?) and
;; returns cleanly once the signal cascade sets it.
(:wat::core::defn :demo::loop [] -> :wat::core::nil
  (:wat::core::if (:wat::kernel::stopped?)
    ()                                       ; observed stop → return clean
    (:demo::loop)))                          ; tight poll loop

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "READY")
    (:demo::loop)))
