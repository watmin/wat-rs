;; wat-tests/std/stream.wat — tests for wat/std/stream.wat.
;;
;; Focus: `with-state` (arc 006's Mealy-machine stream stage) and the
;; `chunks` rewrite on top of it. With arc 009 (names-are-values) shipped,
;; step/flush are passed by name — no pass-through lambda ceremony.
;;
;; Each deftest's body runs in its own sandboxed frozen world; helper
;; defines written at this file's top level do NOT cross into the
;; sandbox. So everything a test needs (producer logic, step/flush
;; if with-state is invoked directly) defines inline via lambda or
;; via a top-level define inside the deftest body's scope.

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

;; ─── chunks — rewritten on top of with-state ─────────────────────────
;;
;; Surface-reduction check. The rewrite must preserve the N:1 batcher
;; contract: emit every full chunk of size N; flush the final partial
;; chunk at upstream EOS (if non-empty); no emissions if upstream sent
;; nothing.

(:wat::test::deftest :wat-tests::std::stream::test-chunks-exact-multiple 1024 :error
  ;; Send 6 items with chunk size 3 → expect two Vec<i64> chunks of 3.
  (:wat::core::let*
    (((source :Vec<i64>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::conj
            (:wat::core::conj
              (:wat::core::conj
                (:wat::core::conj (:wat::core::vec :i64) 1)
                2)
              3)
            4)
          5)
        6))
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

(:wat::test::deftest :wat-tests::std::stream::test-chunks-partial-flush 1024 :error
  ;; Send 5 items with chunk size 3 → expect one full [1 2 3] then a
  ;; flushed partial [4 5] at EOS. Two chunks total.
  (:wat::core::let*
    (((source :Vec<i64>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::conj
            (:wat::core::conj
              (:wat::core::conj (:wat::core::vec :i64) 1)
              2)
            3)
          4)
        5))
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
     ((num-chunks :i64) (:wat::core::length collected))
     ((last-chunk :Vec<i64>) (:wat::core::first (:wat::core::rest collected)))
     ((last-len :i64) (:wat::core::length last-chunk))
     ((_ :()) (:wat::test::assert-eq num-chunks 2)))
    (:wat::test::assert-eq last-len 2)))

(:wat::test::deftest :wat-tests::std::stream::test-chunks-empty-upstream 1024 :error
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
;; duplicates collapses to one. Step/flush defined at the deftest's
;; scope (wat's top level after the deftest expands inside the sandbox).

(:wat::test::deftest :wat-tests::std::stream::test-with-state-dedupe-adjacent 1024 :error
  ;; Input: 1 1 2 2 2 3 1 1. Expect emitted: 1 2 3 1.
  ;; Uses lambdas for step/flush (local to the test body).
  (:wat::core::let*
    (((source :Vec<i64>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::conj
            (:wat::core::conj
              (:wat::core::conj
                (:wat::core::conj
                  (:wat::core::conj
                    (:wat::core::conj (:wat::core::vec :i64) 1)
                    1)
                  2)
                2)
              2)
            3)
          1)
        1))
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
            (:wat::core::tuple
              (Some item)
              (:wat::core::conj (:wat::core::vec :i64) item)))
          ((Some prev)
            (:wat::core::if (:wat::core::= prev item) -> :(Option<i64>,Vec<i64>)
              (:wat::core::tuple last (:wat::core::vec :i64))
              (:wat::core::tuple
                (Some item)
                (:wat::core::conj (:wat::core::vec :i64) item)))))))
     ((flush :fn(Option<i64>)->Vec<i64>)
      (:wat::core::lambda ((_last :Option<i64>) -> :Vec<i64>)
        (:wat::core::vec :i64)))
     ((deduped :wat::std::stream::Stream<i64>)
      (:wat::std::stream::with-state stream initial step flush))
     ((collected :Vec<i64>) (:wat::std::stream::collect deduped))
     ((expected :Vec<i64>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::conj
            (:wat::core::conj (:wat::core::vec :i64) 1)
            2)
          3)
        1)))
    (:wat::test::assert-eq collected expected)))

;; ─── with-state — flush path exercised ───────────────────────────────
;;
;; A reducer that buffers everything until EOS, then emits the lot
;; from flush. Proves EOS → flush → drain path fires.

(:wat::test::deftest :wat-tests::std::stream::test-with-state-buffer-all-at-eos 1024 :error
  (:wat::core::let*
    (((source :Vec<i64>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::conj (:wat::core::vec :i64) 10)
          20)
        30))
     ((stream :wat::std::stream::Stream<i64>)
      (:wat::std::stream::spawn-producer
        (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
          (:wat::core::foldl source ()
            (:wat::core::lambda ((_ :()) (item :i64) -> :())
              (:wat::core::match (:wat::kernel::send tx item) -> :()
                ((Some _) ())
                (:None ())))))))
     ;; Step accumulates; never emits during step.
     ((step :fn(Vec<i64>,i64)->(Vec<i64>,Vec<i64>))
      (:wat::core::lambda ((buf :Vec<i64>) (item :i64) -> :(Vec<i64>,Vec<i64>))
        (:wat::core::tuple (:wat::core::conj buf item) (:wat::core::vec :i64))))
     ;; Flush emits everything collected, in order — names are values,
     ;; :wat::core::identity would work if we had one; here we just
     ;; build a lambda that returns its arg.
     ((flush :fn(Vec<i64>)->Vec<i64>)
      (:wat::core::lambda ((buf :Vec<i64>) -> :Vec<i64>) buf))
     ((buffered :wat::std::stream::Stream<i64>)
      (:wat::std::stream::with-state stream
        (:wat::core::vec :i64)
        step flush))
     ((collected :Vec<i64>) (:wat::std::stream::collect buffered)))
    (:wat::test::assert-eq collected source)))

;; ─── arc 009 sanity — pass a named stdlib define by reference ────────
;;
;; Define a helper at the deftest body's local scope via let*-bound
;; lambda, pass it to :wat::core::map. With arc 009 shipped, we could
;; also pass the lambda via a let*-bound name — this test proves the
;; fn-typed parameter happily accepts the bare symbol binding (the
;; existing path) AND that the substrate treats it uniformly.

(:wat::test::deftest :wat-tests::std::stream::test-names-are-values-via-let-binding 1024 :error
  (:wat::core::let*
    (((source :Vec<i64>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::conj (:wat::core::vec :i64) 1)
          2)
        3))
     ((double :fn(i64)->i64)
      (:wat::core::lambda ((n :i64) -> :i64)
        (:wat::core::i64::* n 2)))
     ((doubled :Vec<i64>) (:wat::core::map source double))
     ((expected :Vec<i64>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::conj (:wat::core::vec :i64) 2)
          4)
        6)))
    (:wat::test::assert-eq doubled expected)))
