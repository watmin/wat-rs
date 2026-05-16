;; wat/kernel/run_threads.wat — arc 170 Stone D1.
;;
;; `:wat::kernel::run-threads` — the user-facing bracket macro for
;; thread-based concurrency. D1 ships the SINGLE-factory form; D2
;; extends to N factories with heterogeneous types via variadic
;; positional collector; D3 layers panic cascade + ProcessGroupErr on
;; top.
;;
;; D1 call form (4 args, all positional):
;;
;;   (:wat::kernel::run-threads
;;     server-rx-type    ;; :keyword — full `Receiver<I>` type for the wrap-fn binder
;;     server-tx-type    ;; :keyword — full `Sender<O>`   type for the wrap-fn binder
;;     factory           ;; :Fn(ThreadPeer<I, O>) -> :nil — server-side worker
;;     client-fn)        ;; :Fn(ThreadPeer<O, I>) -> R     — parent-side driver
;;
;; D1 returns whatever client-fn returns; D3 will wrap to
;; `Result<R, ProcessGroupErr>`.
;;
;; ─── Why pre-baked Receiver<I> / Sender<O> keywords? ─────────────────
;;
;; The wrap-fn the macro generates must bind the spawn-thread raw
;; channel pair (`Receiver<I>`, `Sender<O>`) so that the inner
;; `(factory (ThreadPeer/new server-rx server-tx))` call type-checks
;; against the factory's `:Fn(ThreadPeer<I, O>) -> :nil` signature.
;; Honest delta from D1's BRIEF target: the wat parser tokenizes
;; parametric type keywords (`Receiver<T>`) atomically — `~` unquote
;; does NOT splice INTO a `<>` bracket pair at expand time, because
;; the angle-brackets are part of the keyword token, not a list-
;; structured AST shape. Same constraint `:wat::test::run-hermetic-
;; with-io` documented at its definition site (wat/test.wat:800-815):
;; no `keyword::from-string` runs at macro-expand time in a way that
;; reaches the binder type position. The honest path is to take
;; pre-baked full type keywords as macro args. Compose-on-the-fly
;; would require either a substrate AST-keyword constructor verb
;; usable at expand time (out of scope) or AST-introspection on the
;; factory's declared signature (also out of scope).
;;
;; ─── Why bare factory instead of `(Tuple factory)`? ───────────────────
;;
;; The BRIEF's D1 design pass authorized either (i) Tuple-wrapped
;; `(Tuple factory)` with first-child extraction, or (ii) bare factory
;; positional. Picked (ii) because wat has no expand-time AST
;; destructuring — extracting the single child of a Tuple AST in pure
;; wat is not possible at the defmacro layer. D2 extends to N
;; factories via variadic positional `& (factories ...)` (same
;; precedent as `:wat::test::program` at wat/test.wat:228-231), not
;; via Tuple-AST iteration. The call form scales naturally from
;; `(run-threads rx-type tx-type factory client-fn)` (D1, N=1) to
;; `(run-threads rx-type tx-type factory-A factory-B ... client-fn)` (D2).
;;
;; ─── Expansion shape (D1, N=1 factory) ────────────────────────────────
;;
;; (run-threads :Receiver<I> :Sender<O> factory client-fn)
;;
;; expands to:
;;
;;   (let [thread       (spawn-thread
;;                        (fn [server-rx <- :Receiver<I>
;;                             server-tx <- :Sender<O>]
;;                          -> :nil
;;                          (factory (ThreadPeer/new server-rx server-tx))))
;;         client-peer  (ThreadPeer/new
;;                        (Thread/output thread)   ;; Receiver<O> — parent reads what server wrote
;;                        (Thread/input  thread))  ;; Sender<I>   — parent writes what server reads
;;         result       (client-fn client-peer)
;;         _drained     (Thread/drain-and-join thread)]
;;     result)
;;
;; Type-parameter mirror: server peer is `ThreadPeer<I, O>` (server
;; reads I, writes O); client peer is `ThreadPeer<O, I>` (client reads
;; O, writes I). Same struct, opposite type-param binding per Stone C1
;; design.

(:wat::core::defmacro
  (:wat::kernel::run-threads
    (server-rx-type :AST<wat::core::nil>)
    (server-tx-type :AST<wat::core::nil>)
    (factory        :AST<wat::core::nil>)
    (client-fn      :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::let
     [thread      (:wat::kernel::spawn-thread
                    (:wat::core::fn
                      [server-rx <- ~server-rx-type
                       server-tx <- ~server-tx-type]
                      -> :wat::core::nil
                      (~factory (:wat::kernel::ThreadPeer/new server-rx server-tx))))
      client-peer (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread)
                    (:wat::kernel::Thread/input  thread))
      result      (~client-fn client-peer)
      _drained    (:wat::kernel::Thread/drain-and-join thread)]
     result))
