;; A work-fn that waits on `after` then returns. collect-loop Message arm.
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
    [_ (:user::nap 200)]
    (:wat::core::+ x 1)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     got (:wat::bracket::map (:wat::spawn::thread/runner-count 1)
            (:wat::core::Vector :- [:wat::core::i64] 10)
            :user::work)
     elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     n (:wat::core::first got)]
    (:wat::core::format "arm=Message;got={g};elapsed-ms={e}"
      :g n :e elapsed)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
