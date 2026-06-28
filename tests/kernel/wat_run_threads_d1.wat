;; Co-located fixture for wat_run_threads_d1.rs — slurped via startup_beside(file!()).

;; Server-side echo worker: reads one String, writes it back.
(:wat::core::defn :my::echo-factory
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
  -> :wat::core::nil
  (:wat::core::let
    [line (:wat::kernel::Thread/readln peer)
     _    (:wat::kernel::Thread/println peer line)]
    nil))

;; Named echo-client fn: the actual coordinator logic.
(:wat::core::defn :my::echo-client
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
  -> :wat::core::String
  (:wat::core::let
    [_     (:wat::kernel::Thread/println peer "hello")
     reply (:wat::kernel::Thread/readln peer)]
    reply))

;; Entry point: coordinator-fn form (N=1).
(:wat::core::defn :my::test::run-d1
  [] -> :wat::core::String
  (:wat::kernel::run-threads
    (:wat::core::fn
      [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
      -> :wat::core::String
      (:my::echo-client peer))
    :my::echo-factory))

