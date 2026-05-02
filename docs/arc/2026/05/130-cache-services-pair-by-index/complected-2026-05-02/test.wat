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
;;
;; Arc 130: substrate reshaped to pair-by-index via HandlePool.
;; No per-call channel allocation; client pops a Handle = (ReqTx,
;; ReplyRx) from the pool. The :should-panic annotation retires —
;; the test PASSES without panic. The inner-let* nesting per
;; SERVICE-PROGRAMS.md § "The lockstep" (arc 131) ensures no
;; ScopeDeadlock fires: pool + handle + work live in INNER scope,
;; which returns the driver Thread; OUTER scope holds only the
;; driver Thread and joins it.


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
            ;; Outer scope holds ONLY the two driver Threads (con-drv,
            ;; lru-drv) returned from the inner scope. The inner scope
            ;; owns all spawn-tuples, pools, handles, and work — it
            ;; returns the two threads so the outer can join them AFTER
            ;; all Senders have dropped. SERVICE-PROGRAMS.md § "The
            ;; lockstep" + arc 117 + arc 131.
            (:wat::core::let*
              (((drvs :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::Thread<wat::core::unit,wat::core::unit>))
                (:wat::core::let*
                  (((con-state :wat::console::Spawn)
                    (:wat::console::spawn stdout stderr 2))
                   ((con-drv :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::second con-state))
                   ((con-pool :wat::kernel::HandlePool<wat::console::Handle>)
                    (:wat::core::first con-state))
                   ((diag :wat::console::Handle)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((_spare :wat::console::Handle)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((_con-finish :wat::core::unit) (:wat::kernel::HandlePool::finish con-pool))

                   ((state :wat::lru::Spawn<wat::core::String,wat::core::i64>)
                    (:wat::lru::spawn 16 1
                      :wat::lru::null-reporter
                      (:wat::lru::null-metrics-cadence)))
                   ((lru-drv :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::second state))
                   ((pool :wat::kernel::HandlePool<wat::lru::Handle<wat::core::String,wat::core::i64>>)
                    (:wat::core::first state))
                   ((handle :wat::lru::Handle<wat::core::String,wat::core::i64>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_pool-finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

                   ((_t1 :wat::core::unit) (:wat::console::err diag "T1: about-to-put\n"))
                   ;; Arc 130: put takes Handle + Vec<Entry<K,V>>; no per-call channel.
                   ((_ :wat::core::unit)
                    (:wat::lru::put handle
                      (:wat::core::conj
                        (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>)
                        (:wat::core::Tuple "answer" 42))))
                   ((_t2 :wat::core::unit) (:wat::console::err diag "T2: put-acked\n"))
                   ;; Arc 130: get takes Handle + Vec<K>; returns Vec<Option<V>>.
                   ((results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
                    (:wat::lru::get handle
                      (:wat::core::conj
                        (:wat::core::Vector :wat::core::String)
                        "answer")))
                   ((_t3 :wat::core::unit) (:wat::console::err diag "T3: get-returned\n"))
                   ;; Two-level match: outer on first's Option wrapper; inner on the cache hit/miss.
                   ((_report :wat::core::unit)
                    (:wat::core::match (:wat::core::first results) -> :wat::core::unit
                      ((:wat::core::Some inner)
                        (:wat::core::match inner -> :wat::core::unit
                          ((:wat::core::Some _v) (:wat::console::out diag "hit\n"))
                          (:wat::core::None       (:wat::console::out diag "miss\n"))))
                      (:wat::core::None (:wat::console::out diag "miss\n")))))
                  ;; Inner returns the two threads; pool + handle drop here.
                  (:wat::core::Tuple con-drv lru-drv)))
               ((con-drv :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first drvs))
               ((lru-drv :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second drvs))
               ;; Outer's only operations: join both now-disconnected drivers.
               ((_join-lru :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                (:wat::kernel::Thread/join-result lru-drv))
               ((_join-con :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
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
