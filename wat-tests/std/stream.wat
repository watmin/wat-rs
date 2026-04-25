;; wat-tests/std/stream.wat — tests for wat/std/stream.wat.
;;
;; Focus: `with-state` (arc 006's Mealy-machine stream stage) and the
;; `chunks` rewrite on top of it. With arc 009 (names-are-values) shipped,
;; step/flush are passed by name — no pass-through lambda ceremony.
;;
;; Each deftest's body runs in its own sandboxed frozen world; helper
;; defines written at this file's top level do NOT cross into the
;; sandbox. Everything a test needs (producer logic, step/flush)
;; defines inline via lambda or uses foldl for producer iteration.


;; ─── chunks — rewritten on top of with-state ─────────────────────────
;;
;; Surface-reduction check. The rewrite must preserve the N:1 batcher
;; contract: emit every full chunk of size N; flush the final partial
;; chunk at upstream EOS (if non-empty); no emissions if upstream sent
;; nothing.

(:wat::test::deftest :wat-tests::std::stream::test-chunks-exact-multiple
  ()
  ;; Send 6 items with chunk size 3 → expect two Vec<i64> chunks of 3.
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 2 3 4 5 6))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((chunked :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::chunks stream 3))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect chunked))
     ((num-chunks :i64) (:wat::core::length collected)))
    (:wat::test::assert-eq num-chunks 2)))

(:wat::test::deftest :wat-tests::std::stream::test-chunks-partial-flush
  ()
  ;; Send 5 items with chunk size 3 → expect one full [1 2 3] then a
  ;; flushed partial [4 5] at EOS.
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 2 3 4 5))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((chunked :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::chunks stream 3))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect chunked))
     ((expected :Vec<Vec<i64>>)
      (:wat::core::vec :Vec<i64>
        (:wat::core::vec :i64 1 2 3)
        (:wat::core::vec :i64 4 5))))
    (:wat::test::assert-eq collected expected)))

(:wat::test::deftest :wat-tests::std::stream::test-chunks-empty-upstream
  ()
  ;; No items sent → flush sees empty buffer → no chunks emitted.
  (:wat::core::let*
    (((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :()) ())))
     ((chunked :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::chunks stream 3))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect chunked))
     ((num-chunks :i64) (:wat::core::length collected)))
    (:wat::test::assert-eq num-chunks 0)))

;; ─── with-state — dedupe-adjacent (classic Mealy) ────────────────────
;;
;; Emit each item unless it equals the most recent emitted item. State
;; is :Option<i64> (last emitted). First item always emits; any run of
;; duplicates collapses to one.

(:wat::test::deftest :wat-tests::std::stream::test-with-state-dedupe-adjacent
  ()
  ;; Input: 1 1 2 2 2 3 1 1 → expect 1 2 3 1.
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 1 2 2 2 3 1 1))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((initial :Option<i64>) :None)
     ((step :fn(Option<i64>,i64)->(Option<i64>,Vec<i64>))
      (:wat::core::lambda ((last :Option<i64>) (item :i64) -> :(Option<i64>,Vec<i64>))
        (:wat::core::match last -> :(Option<i64>,Vec<i64>)
          (:None
            (:wat::core::tuple (Some item) (:wat::core::vec :i64 item)))
          ((Some prev)
            (:wat::core::if (:wat::core::= prev item) -> :(Option<i64>,Vec<i64>)
              (:wat::core::tuple last (:wat::core::vec :i64))
              (:wat::core::tuple (Some item) (:wat::core::vec :i64 item)))))))
     ((flush :fn(Option<i64>)->Vec<i64>)
      (:wat::core::lambda ((_last :Option<i64>) -> :Vec<i64>)
        (:wat::core::vec :i64)))
     ((deduped :wat::std::stream::Stream<i64>)
      (:wat::std::stream::with-state stream initial step flush))
     ((collected :Vec<i64>) (:wat::std::stream::collect deduped))
     ((expected :Vec<i64>) (:wat::core::vec :i64 1 2 3 1)))
    (:wat::test::assert-eq collected expected)))

;; ─── with-state — flush path exercised ───────────────────────────────
;;
;; A reducer that buffers everything until EOS, then emits the lot
;; from flush. Proves EOS → flush → drain path fires.

(:wat::test::deftest :wat-tests::std::stream::test-with-state-buffer-all-at-eos
  ()
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 10 20 30))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((step :fn(Vec<i64>,i64)->(Vec<i64>,Vec<i64>))
      (:wat::core::lambda ((buf :Vec<i64>) (item :i64) -> :(Vec<i64>,Vec<i64>))
        (:wat::core::tuple (:wat::core::conj buf item) (:wat::core::vec :i64))))
     ((flush :fn(Vec<i64>)->Vec<i64>)
      (:wat::core::lambda ((buf :Vec<i64>) -> :Vec<i64>) buf))
     ((buffered :wat::std::stream::Stream<i64>)
      (:wat::std::stream::with-state stream
        (:wat::core::vec :i64)
        step flush))
     ((collected :Vec<i64>) (:wat::std::stream::collect buffered)))
    (:wat::test::assert-eq collected source)))

;; ─── arc 009 sanity — map over a vec via a named define ──────────────
;;
;; The canonical shape: `(:wat::core::map (:wat::core::vec :i64 1 2 3) double)`
;; transforms into `(:wat::core::vec :i64 2 4 6)`. Named define `double`
;; passes by bare reference via the let*-bound lambda.

(:wat::test::deftest :wat-tests::std::stream::test-names-are-values-via-let-binding
  ()
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 2 3))
     ((double :fn(i64)->i64)
      (:wat::core::lambda ((n :i64) -> :i64)
        (:wat::core::* n 2)))
     ((doubled :Vec<i64>) (:wat::core::map source double))
     ((expected :Vec<i64>) (:wat::core::vec :i64 2 4 6)))
    (:wat::test::assert-eq doubled expected)))

;; ─── chunks-by — key-boundary N:1 partitioning ────────────────────────

(:wat::test::deftest :wat-tests::std::stream::test-chunks-by-runs-on-identity
  ()
  ;; Stream [1 1 2 3 3 3 1] grouped by identity → [[1 1] [2] [3 3 3] [1]].
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 1 2 3 3 3 1))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((id :fn(i64)->i64)
      (:wat::core::lambda ((x :i64) -> :i64) x))
     ((grouped :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::chunks-by stream id))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect grouped))
     ((expected :Vec<Vec<i64>>)
      (:wat::core::vec :Vec<i64>
        (:wat::core::vec :i64 1 1)
        (:wat::core::vec :i64 2)
        (:wat::core::vec :i64 3 3 3)
        (:wat::core::vec :i64 1))))
    (:wat::test::assert-eq collected expected)))

(:wat::test::deftest :wat-tests::std::stream::test-chunks-by-all-distinct
  ()
  ;; Stream [1 2 3] grouped by identity → [[1] [2] [3]] (each its own run).
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 2 3))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((id :fn(i64)->i64)
      (:wat::core::lambda ((x :i64) -> :i64) x))
     ((grouped :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::chunks-by stream id))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect grouped))
     ((expected :Vec<Vec<i64>>)
      (:wat::core::vec :Vec<i64>
        (:wat::core::vec :i64 1)
        (:wat::core::vec :i64 2)
        (:wat::core::vec :i64 3))))
    (:wat::test::assert-eq collected expected)))

(:wat::test::deftest :wat-tests::std::stream::test-chunks-by-empty-stream
  ()
  ;; Empty stream → no groups emitted.
  (:wat::core::let*
    (((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :()) ())))
     ((id :fn(i64)->i64)
      (:wat::core::lambda ((x :i64) -> :i64) x))
     ((grouped :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::chunks-by stream id))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect grouped))
     ((num :i64) (:wat::core::length collected)))
    (:wat::test::assert-eq num 0)))

;; ─── window — sliding N-length windows, flush-partial-when-short ──────

(:wat::test::deftest :wat-tests::std::stream::test-window-full-windows
  ()
  ;; Stream [1 2 3 4 5], size 3 → [[1 2 3] [2 3 4] [3 4 5]].
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 2 3 4 5))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((windowed :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::window stream 3))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect windowed))
     ((expected :Vec<Vec<i64>>)
      (:wat::core::vec :Vec<i64>
        (:wat::core::vec :i64 1 2 3)
        (:wat::core::vec :i64 2 3 4)
        (:wat::core::vec :i64 3 4 5))))
    (:wat::test::assert-eq collected expected)))

(:wat::test::deftest :wat-tests::std::stream::test-window-short-stream-flushes-partial
  ()
  ;; Stream [1 2], size 3 — never reached size, flush emits [[1 2]].
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 2))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((windowed :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::window stream 3))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect windowed))
     ((expected :Vec<Vec<i64>>)
      (:wat::core::vec :Vec<i64>
        (:wat::core::vec :i64 1 2))))
    (:wat::test::assert-eq collected expected)))

(:wat::test::deftest :wat-tests::std::stream::test-window-exactly-size-no-flush
  ()
  ;; Stream [1 2 3], size 3 — one full window emitted, flush empty.
  (:wat::core::let*
    (((source :Vec<i64>) (:wat::core::vec :i64 1 2 3))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ((windowed :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::window stream 3))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect windowed))
     ((expected :Vec<Vec<i64>>)
      (:wat::core::vec :Vec<i64>
        (:wat::core::vec :i64 1 2 3))))
    (:wat::test::assert-eq collected expected)))

(:wat::test::deftest :wat-tests::std::stream::test-window-empty-stream
  ()
  ;; Empty stream → no windows emitted at all.
  (:wat::core::let*
    (((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :()) ())))
     ((windowed :wat::std::stream::Stream<Vec<i64>>)
      (:wat::std::stream::window stream 3))
     ((collected :Vec<Vec<i64>>) (:wat::std::stream::collect windowed))
     ((num :i64) (:wat::core::length collected)))
    (:wat::test::assert-eq num 0)))
