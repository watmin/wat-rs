;; Co-located fixture for wat_run_threads_d2.rs — slurped via startup_beside(file!()).

;; Factory A: echo — reads one String, writes it back.
(:wat::core::defn :my::worker-a
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
  -> :wat::core::nil
  (:wat::core::let
    [line (:wat::kernel::Thread/readln peer)
     _    (:wat::kernel::Thread/println peer line)]
    nil))

;; Factory B: reads any String, writes "world".
(:wat::core::defn :my::worker-b
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::Thread/readln peer)
     _ (:wat::kernel::Thread/println peer "world")]
    nil))

;; Factory C: reads any String, writes "pong".
(:wat::core::defn :my::worker-c
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::Thread/readln peer)
     _ (:wat::kernel::Thread/println peer "pong")]
    nil))

;; Named coordinator fn.
(:wat::core::defn :my::three-fac-coordinator
  [a <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
   b <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
   c <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
  -> :wat::core::Vector<wat::core::String>
  (:wat::core::let
    [_       (:wat::kernel::Thread/println a "hello")
     reply-a (:wat::kernel::Thread/readln a)
     _       (:wat::kernel::Thread/println b "hello")
     reply-b (:wat::kernel::Thread/readln b)
     _       (:wat::kernel::Thread/println c "ping")
     reply-c (:wat::kernel::Thread/readln c)]
    (:wat::core::Vector :wat::core::String reply-a reply-b reply-c)))

;; Entry point: three-factory coordinator-fn form (N=3).
(:wat::core::defn :my::test::run-d2
  [] -> :wat::core::Vector<wat::core::String>
  (:wat::kernel::run-threads
    (:wat::core::fn
      [a <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
       b <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
       c <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
      -> :wat::core::Vector<wat::core::String>
      (:my::three-fac-coordinator a b c))
    :my::worker-a
    :my::worker-b
    :my::worker-c))

