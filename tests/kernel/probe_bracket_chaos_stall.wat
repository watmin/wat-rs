;; A work-fn that stalls past collect-deadline-ms. 1 runner so collect-wait-one
;; (recv-by-deadline) can fire. Inject WAT_COLLECT_DEADLINE_MS=200. Nap 2000.
(:wat::core::defn :user::nap
  [ms <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread
        (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)
    (:wat::kernel::RecvOutcome::TimedOut nil)
    ((:wat::kernel::RecvOutcome::Malformed _c) nil)))

(:wat::core::defn :user::work
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [_ (:user::nap 2000)]
    (:wat::core::+ x 1)))

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::bracket::map (:wat::spawn::thread/runner-count 1)
    (:wat::core::Vector :- [:wat::core::i64] 0)
    :user::work))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
