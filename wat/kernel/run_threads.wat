;; wat/kernel/run_threads.wat — arc 170 Stone D1 (refactored 2026-05-16).
;;
;; `:wat::kernel::run-threads` — the user-facing bracket macro for
;; thread-based concurrency. D1 ships the SINGLE-factory form; D2
;; extends to N factories with heterogeneous types via variadic
;; positional collector; D3 layers panic cascade + ProcessGroupErr on
;; top.
;;
;; D1 call form (4 args, all positional — clean):
;;
;;   (:wat::kernel::run-threads
;;     i-type            ;; :keyword — the I type arg (e.g. :wat::core::String)
;;     o-type            ;; :keyword — the O type arg (e.g. :wat::core::String)
;;     factory           ;; :Fn(ThreadPeer<I, O>) -> :nil — server-side worker
;;     client-fn)        ;; :Fn(ThreadPeer<O, I>) -> R     — parent-side driver
;;
;; D1 returns whatever client-fn returns; D3 will wrap to
;; `Result<R, ProcessGroupErr>`.
;;
;; ─── How the macro constructs `Receiver<I>` / `Sender<O>` keywords ───
;;
;; The wrap-fn the macro generates must bind the spawn-thread raw
;; channel pair (`Receiver<I>`, `Sender<O>`) so the inner
;; `(factory (ThreadPeer/new server-rx server-tx))` call type-checks
;; against the factory's `:Fn(ThreadPeer<I, O>) -> :nil` signature.
;;
;; wat tokenizes parametric type keywords `<...>` atomically — `~`
;; unquote does NOT splice INTO a `<>` bracket pair at expand time.
;; But arc 143 slice 2's COMPUTED UNQUOTE (`~(expr)`) DOES eval an
;; arbitrary substrate expression at expand time and convert the
;; result to a WatAST node via `value_to_watast`. Composing the existing
;; substrate primitives:
;;
;;   :wat::core::keyword/to-string  — keyword → String (strips ':' prefix)
;;   :wat::core::string::concat     — variadic String concat
;;   :wat::core::keyword/from-string — String → keyword (adds ':' prefix)
;;
;; ...lets the macro construct `:rust::crossbeam_channel::Receiver<I>`
;; at expand time from the user's bare `:I` arg. The constructed
;; keyword lands at the binder type position via the same path
;; arc 143's `:wat::runtime::define-alias` macro uses.
;;
;; ─── Why bare factory instead of `(Tuple factory)`? ───────────────────
;;
;; wat has no expand-time AST destructuring — extracting the single
;; child of a Tuple AST in pure wat is not possible at the defmacro
;; layer. D2 extends to N factories via variadic positional collector
;; (`& (factories ...)`), same precedent as `:wat::test::program` at
;; wat/test.wat:228-231. The call form scales naturally from
;; `(run-threads :I :O factory client-fn)` (D1, N=1) to
;; `(run-threads :I :O factory-A factory-B ... client-fn)` (D2).
;;
;; ─── Expansion shape (D1, N=1 factory) ────────────────────────────────
;;
;; (run-threads :wat::core::String :wat::core::String factory client-fn)
;;
;; expands to:
;;
;;   (let [thread       (spawn-thread
;;                        (fn [server-rx <- :rust::crossbeam_channel::Receiver<wat::core::String>
;;                             server-tx <- :rust::crossbeam_channel::Sender<wat::core::String>]
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
;;
;; ─── Production precedent — arc 143 slice 6's define-alias ────────────
;;
;; `:wat::runtime::define-alias` (wat/runtime.wat:22-29) uses the same
;; computed-unquote pattern to construct a fresh `:wat::core::define`
;; form whose head is a renamed callable. In production since arc 143
;; shipped (2026-05). Substrate path proven; arc 199's proposal to
;; add new substrate primitives was REJECTED post-investigation
;; because this composition already covers the surface.

(:wat::core::defmacro
  (:wat::kernel::run-threads
    (i-type    :AST<wat::core::keyword>)
    (o-type    :AST<wat::core::keyword>)
    (factory   :AST<wat::core::nil>)
    (client-fn :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::let
     [thread      (:wat::kernel::spawn-thread
                    (:wat::core::fn
                      [server-rx <- ~(:wat::core::keyword/from-string
                                       (:wat::core::string::concat
                                         "rust::crossbeam_channel::Receiver<"
                                         (:wat::core::keyword/to-string i-type)
                                         ">"))
                       server-tx <- ~(:wat::core::keyword/from-string
                                       (:wat::core::string::concat
                                         "rust::crossbeam_channel::Sender<"
                                         (:wat::core::keyword/to-string o-type)
                                         ">"))]
                      -> :wat::core::nil
                      (~factory (:wat::kernel::ThreadPeer/new server-rx server-tx))))
      client-peer (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread)
                    (:wat::kernel::Thread/input  thread))
      result      (~client-fn client-peer)
      _drained    (:wat::kernel::Thread/drain-and-join thread)]
     result))
