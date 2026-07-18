;; Co-located fixture for probe_arc278_process_crash_reason_carried.rs — arc 278 no-hidden-failures,
;; transport-tier twin, RED gate 2 (the crash half).
;;
;; SUBJECT: a PROCESS-locus service (NOT thread — thread-tier crash-reason is already GREEN via
;; probe_arc259_thread_crash_reason) with one op whose handler GENUINELY panics (assertion-failed!
;; raises inside the serve loop — a real runtime panic mid-handler, NOT a decode rejection; decode
;; rejection is Mechanism A's path, covered by probe_arc278_dead_child_speaks). Modeled directly on
;; probe_arc272_rs2_crash_surfaces_to_client.wat, which only asserts is_err (a mute raise would pass
;; that); this fixture is for the .rs harness that also asserts the raised reason CARRIES the sentinel.
(:wat::core::defsurface :my::CrashSvc :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::CrashSvc::BoomRequest  [])
   (:wat::core::defrecord :my::CrashSvc::BoomResponse [ok <- :wat::core::bool])]
  :features
  [(boom [self <- :my::CrashSvc  req <- :my::CrashSvc::BoomRequest] -> :my::CrashSvc::BoomResponse)])

(:wat::service::defservice :my::crashsvc
  :satisfies :my::CrashSvc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(boom [s req]
     (:wat::kernel::assertion-failed!
       "BOOM-SENTINEL-PROCESS-4471 — the process handler crashed on purpose"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::crashsvc/start :locus (:wat::spawn::process) :record (:my::crashsvc::Record :count 0))
     c (:wat::kernel::connect' (:my::crashsvc::Handle/addr h))
     _ (:my::crashsvc/boom c (:my::CrashSvc::BoomRequest))]
    true))
