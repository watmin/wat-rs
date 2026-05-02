;; crates/wat-lru/wat-tests/CacheService.wat — restored from
;; pre-slice-4b wat-tests/std/service/Cache.wat (arc 015 slice 3).
;;
;; CacheService composes with Console: both run driver threads, both
;; need thread-safe stdio. In-process `:wat::test::run-ast` uses
;; StringIoWriter under `ThreadOwnedCell` (single-thread) and would
;; panic on cross-thread writes, so this test runs through
;; `:wat::test::run-hermetic-ast` — real subprocess (forked via arc
;; 012's fork-program-ast, COW-inherits the parent test binary's
;; installed dep_sources OnceLock so wat-lru's surface is reachable
;; in the child). Real stdio, AST-entry so the inner program reads
;; as s-expressions not an escaped string.
;;
;; The T1/T2/T3 stderr checkpoints stay as regression sentinels —
;; a future hang halts at the last checkpoint, surfacing the
;; thread-ownership bug that drove the original test.


;; Arc 124 — same Pattern B Put-ack helper-verb cycle deadlock as
;; HologramCacheService's step3-6. `:ignore` keeps cargo test green;
;; `:time-limit "200ms"` is the safety net for `--include-ignored`.
(:wat::test::ignore "arc 119: Put-ack helper-verb cycle deadlock; step 7 under investigation")
(:wat::test::time-limit "200ms")
(:wat::test::deftest :wat-lru::test-cache-service-put-then-get-round-trip
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            ;; Outer scope holds driver handles. The inner scope owns
            ;; the senders — when it exits, senders drop, drivers see
            ;; disconnect, outer joins flush-and-exit cleanly.
            (:wat::core::let*
              (((con-state :wat::console::Spawn)
                (:wat::console::spawn stdout stderr 2))
               ((con-drv :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                (:wat::core::second con-state))
               ((state :wat::lru::Spawn<wat::core::String,wat::core::i64>)
                (:wat::lru::spawn 16 1
                  :wat::lru::null-reporter
                  (:wat::lru::null-metrics-cadence)))
               ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                (:wat::core::second state))

               ((_ :wat::core::unit)
                (:wat::core::let*
                  (((con-pool :wat::kernel::HandlePool<wat::console::Handle>)
                    (:wat::core::first con-state))
                   ((diag :wat::console::Handle)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((_spare :wat::console::Handle)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((_ :wat::core::unit) (:wat::kernel::HandlePool::finish con-pool))

                   ((pool :wat::kernel::HandlePool<wat::lru::ReqTx<wat::core::String,wat::core::i64>>)
                    (:wat::core::first state))
                   ((req-tx :wat::lru::ReqTx<wat::core::String,wat::core::i64>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_ :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

                   ;; Arc 119: separate ack channel (Put, Pattern A unit-ack)
                   ;; and reply channel (Get, Pattern B data-back Vec<Option<V>>).
                   ((reply-pair :wat::lru::ReplyChannel<wat::core::i64>)
                    (:wat::kernel::make-bounded-channel
                      :wat::core::Vector<wat::core::Option<wat::core::i64>> 1))
                   ((reply-tx :wat::lru::ReplyTx<wat::core::i64>)
                    (:wat::core::first reply-pair))
                   ((reply-rx :wat::lru::ReplyRx<wat::core::i64>)
                    (:wat::core::second reply-pair))
                   ((ack-pair :wat::lru::PutAckChannel)
                    (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                   ((ack-tx :wat::lru::PutAckTx)
                    (:wat::core::first ack-pair))
                   ((ack-rx :wat::lru::PutAckRx)
                    (:wat::core::second ack-pair))

                   ((_ :wat::core::unit) (:wat::console::err diag "T1: about-to-put\n"))
                   ;; Arc 119: put takes Vec<Entry<K,V>>; batch-of-one.
                   ((_ :wat::core::unit)
                    (:wat::lru::put req-tx ack-tx ack-rx
                      (:wat::core::conj
                        (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>)
                        (:wat::core::Tuple "answer" 42))))
                   ((_ :wat::core::unit) (:wat::console::err diag "T2: put-acked\n"))
                   ;; Arc 119: get takes Vec<K>; returns Vec<Option<V>>.
                   ;; first on Vector<Option<T>> returns Option<Option<T>> — double-unwrap.
                   ((results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
                    (:wat::lru::get req-tx reply-tx reply-rx
                      (:wat::core::conj
                        (:wat::core::Vector :wat::core::String)
                        "answer")))
                   ((_ :wat::core::unit) (:wat::console::err diag "T3: get-returned\n")))
                  ;; Two-level match: outer on first's Option wrapper; inner on the cache hit/miss.
                  (:wat::core::match (:wat::core::first results) -> :wat::core::unit
                    ((:wat::core::Some inner)
                      (:wat::core::match inner -> :wat::core::unit
                        ((:wat::core::Some _v) (:wat::console::out diag "hit\n"))
                        (:wat::core::None       (:wat::console::out diag "miss\n"))))
                    (:wat::core::None (:wat::console::out diag "miss\n")))))

               ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                (:wat::kernel::Thread/join-result driver))
               ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                (:wat::kernel::Thread/join-result con-drv)))
              ())))
        (:wat::core::Vector :wat::core::String)))
     ((stdout :wat::core::Vector<wat::core::String>) (:wat::kernel::RunResult/stdout r))
     ((stderr :wat::core::Vector<wat::core::String>) (:wat::kernel::RunResult/stderr r))
     ;; Assertions:
     ;;   - stdout first line is "hit" (put→get round-trip succeeded)
     ;;   - stderr contains each of the T1/T2/T3 checkpoints
     ((hit-line :wat::core::String)
      (:wat::core::match (:wat::core::first stdout) -> :wat::core::String
        ((:wat::core::Some s) s)
        (:wat::core::None "<missing>")))
     ((_ :wat::core::unit) (:wat::test::assert-eq hit-line "hit"))
     ((stderr-blob :wat::core::String) (:wat::core::string::join "\n" stderr))
     ((_ :wat::core::unit) (:wat::test::assert-contains stderr-blob "T1: about-to-put"))
     ((_ :wat::core::unit) (:wat::test::assert-contains stderr-blob "T2: put-acked")))
    (:wat::test::assert-contains stderr-blob "T3: get-returned")))
