;; wat/kernel/run_threads.wat — arc 170 Stone D2 (coordinator-fn form, 2026-05-16).
;;
;; `:wat::kernel::run-threads` — the user-facing bracket macro for
;; thread-based concurrency. D2 ships the coordinator-fn form that
;; scales from N=1 to N=3 via the arc 201 reflection chain. D3 adds
;; panic cascade + ProcessGroupErr on top.
;;
;; ─── Required call form (coordinator-fn) ──────────────────────────────
;;
;;   (:wat::kernel::run-threads
;;     (:wat::core::fn
;;       [peer-a <- :wat::kernel::ThreadPeer<A-in, A-out>
;;        peer-b <- :wat::kernel::ThreadPeer<B-in, B-out>
;;        ...]
;;       -> :Result-type
;;       (:user::actual-coordinator-fn peer-a peer-b ...))
;;     (:app::factory-a)
;;     (:app::factory-b)
;;     ...)
;;
;; Coordinator-fn structural rule: the inline `(:wat::core::fn ...)` body
;; is ALWAYS a single delegating call to a named fn. The inline fn carries
;; the binder declarations (names + types) for reflection. The real work
;; lives in a named fn that is independently testable, reflectable, and
;; reusable. Same pattern as `:wat::runtime::define-alias`.
;;
;; ─── Macro algorithm (arc 201 reflection chain) ────────────────────────
;;
;; 1. Macro receives the coordinator AST + N variadic factory call form ASTs
;; 2. At expand time — count N from the coordinator's arg count (via
;;    signature-of-fn + extract-arg-names)
;; 3. Dispatch to `:wat::kernel::run-threads-n1` (N=1) or
;;    `:wat::kernel::run-threads-n3` (N=3) helper macros
;; 4. Each helper macro uses the arc 201 reflection chain to extract
;;    ThreadPeer<I,O> type args from the coordinator signature:
;;    signature-of-fn → extract-arg-types → Bundle/children → atom-value
;;    → keyword/to-string + string::concat + keyword/from-string
;;    This constructs :rust::crossbeam_channel::Receiver<I> / Sender<O>
;;    at expand time (same pattern arc 143 slice 2 proved with define-alias)
;; 5. Peer binding names come from coordinator arg names (extract-arg-names
;;    + to-watast → WatAST::Symbol as valid let binder)
;; 6. Coordinator is called as (~coordinator peer-a peer-b ...) where the
;;    peer symbols are spliced via ~@(extract-arg-names sig)
;;
;; ─── Fresh-name strategy ───────────────────────────────────────────────
;;
;; STOP-trigger-1 discovery: the BRIEF described constructing fresh
;; thread/drain binding names via keyword/from-string (e.g. "thread-logger"
;; from ":logger"). This is BLOCKED because parse_let_binding only accepts
;; WatAST::Symbol (not WatAST::Keyword) as a binder, and keyword/from-string
;; produces WatAST::Keyword via value_to_watast. Resolution:
;;
;; - Thread binding names: LITERAL fixed index-based symbols (thread-0,
;;   thread-1, thread-2) embedded in each N-specific template
;; - Peer binding names: coordinator's own binder names (extract-arg-names
;;   + to-watast → WatAST::Symbol("logger")) — valid let binders
;; - Drain binding names: LITERAL fixed index-based symbols (_drained-0,
;;   _drained-1, _drained-2)
;;
;; Hygiene note: the macro-generated names `thread-0`/`thread-1`/`thread-2`,
;; `_drained-0`/`_drained-1`/`_drained-2`, and `result` shadow any
;; same-named user bindings in scope. Per BRIEF STOP-trigger 2: this is
;; documented. Users should avoid these names in coordinator-fn bodies.
;; The peer names MATCH coordinator binders by design — name collision
;; between peer-binder and user binders would be caught by the type checker.
;;
;; ─── Dispatch mechanism ────────────────────────────────────────────────
;;
;; The variadic `run-threads` macro body uses a computed-unquote `~(let[...]...)`
;; at expand time to:
;; 1. Count N from the coordinator's signature via substrate primitives only
;; 2. Extract factory 0/1/2 ASTs by quoting the variadic rest-list and
;;    converting through HolonAST (quasiquote → from-watast → Bundle/children
;;    → to-watast round-trip)
;; 3. Return a Value::wat__WatAST containing the call to the N-specific macro
;;    (run-threads-n1 or run-threads-n3) with coordinator and factory args
;; 4. The macro expansion pipeline re-expands the result, firing the
;;    N-specific macro to produce the final let form
;;
;; All dispatch uses substrate primitives only (no user defns at expand time).
;;
;; ─── N=1 expansion shape ───────────────────────────────────────────────
;;
;; (run-threads (fn [peer <- :ThreadPeer<S,S>] -> :S (:my::run peer)) factory)
;;
;; expands to:
;;
;;   (let [thread-0    (spawn-thread
;;                       (fn [server-rx <- :Receiver<S> server-tx <- :Sender<S>]
;;                         -> :nil
;;                         (factory (ThreadPeer/new server-rx server-tx))))
;;         peer        (ThreadPeer/new (Thread/output thread-0) (Thread/input thread-0))
;;         result      ((fn [peer <- :ThreadPeer<S,S>] -> :S (:my::run peer)) peer)
;;         _drained-0  (Thread/drain-and-join thread-0)]
;;     result)
;;
;; ─── N=3 expansion shape ───────────────────────────────────────────────
;;
;; Similar but with thread-0/peer-name-0, thread-1/peer-name-1,
;; thread-2/peer-name-2, and three drain bindings.
;;
;; ─── Production precedent ──────────────────────────────────────────────
;;
;; Arc 143 slice 6's `:wat::runtime::define-alias` uses the same
;; computed-unquote pattern for reflect-driven expansion. Arc 201's
;; signature-of-fn + extract-arg-types + Bundle/children chain was
;; designed for this consumer. Both in production since their respective
;; arcs shipped.

;; ─── N=1 coordinator-fn helper macro ──────────────────────────────────
;;
;; Called from `run-threads` dispatch when coordinator has 1 binder.
;; Receives: coordinator fn-form AST + factory-0 call-form AST.
;;
;; Type extraction per slot 0:
;;   i-type-0 = I from ThreadPeer<I,O> at coordinator arg 0 (children[1])
;;   o-type-0 = O from ThreadPeer<I,O> at coordinator arg 0 (children[2])
;;   receiver-0 = :rust::crossbeam_channel::Receiver<i-type-0>
;;   sender-0   = :rust::crossbeam_channel::Sender<o-type-0>
;;
;; Peer binder name: coordinator arg name 0 (e.g. "logger" → WatAST::Symbol).
;; Coordinator call: (~coordinator peer-name-0).

(:wat::core::defmacro
  (:wat::kernel::run-threads-n1
    (coordinator :AST<wat::core::nil>)
    (factory-0   :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::let
     [thread-0
        (:wat::kernel::spawn-thread
          (:wat::core::fn
            [server-rx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty0 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 0)
                                     "run-threads-n1: missing type arg 0")
                               ch0 (:wat::holon::Bundle/children ty0)
                               i0h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch0 1)
                                     "run-threads-n1: missing I-type child at slot 1")
                               i0  (:wat::core::atom-value i0h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Receiver<"
                                  (:wat::core::keyword/to-string i0)
                                  ">")))
             server-tx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty0 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 0)
                                     "run-threads-n1: missing type arg 0")
                               ch0 (:wat::holon::Bundle/children ty0)
                               o0h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch0 2)
                                     "run-threads-n1: missing O-type child at slot 2")
                               o0  (:wat::core::atom-value o0h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Sender<"
                                  (:wat::core::keyword/to-string o0)
                                  ">")))]
            -> :wat::core::nil
            (~factory-0 (:wat::kernel::ThreadPeer/new server-rx server-tx))))
      ~(:wat::holon::to-watast
          (:wat::core::Option/expect -> :wat::holon::HolonAST
            (:wat::core::get
              (:wat::runtime::extract-arg-names
                (:wat::runtime::signature-of-fn coordinator))
              0)
            "run-threads-n1: coordinator has no binder at position 0"))
        (:wat::kernel::ThreadPeer/new
          (:wat::kernel::Thread/output thread-0)
          (:wat::kernel::Thread/input  thread-0))
      result
        (~coordinator
          ~@(:wat::runtime::extract-arg-names
               (:wat::runtime::signature-of-fn coordinator)))
      _drained-0
        (:wat::kernel::Thread/drain-and-join thread-0)]
     result))

;; ─── N=3 coordinator-fn helper macro ──────────────────────────────────
;;
;; Called from `run-threads` dispatch when coordinator has 3 binders.
;; Receives: coordinator fn-form AST + factory-0/1/2 call-form ASTs.
;;
;; Each slot k extracts:
;;   i-type-k = children[1] of type-AST at arg k
;;   o-type-k = children[2] of type-AST at arg k
;;   receiver-k = :rust::crossbeam_channel::Receiver<i-type-k>
;;   sender-k   = :rust::crossbeam_channel::Sender<o-type-k>
;;
;; Peer binder names: coordinator arg names 0/1/2 (WatAST::Symbol).
;; Coordinator call: (~coordinator peer-0-name peer-1-name peer-2-name).

(:wat::core::defmacro
  (:wat::kernel::run-threads-n3
    (coordinator :AST<wat::core::nil>)
    (factory-0   :AST<wat::core::nil>)
    (factory-1   :AST<wat::core::nil>)
    (factory-2   :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::let
     [thread-0
        (:wat::kernel::spawn-thread
          (:wat::core::fn
            [server-rx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty0 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 0)
                                     "run-threads-n3: missing type arg 0")
                               ch0 (:wat::holon::Bundle/children ty0)
                               i0h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch0 1)
                                     "run-threads-n3: missing I-type child at slot 0:1")
                               i0  (:wat::core::atom-value i0h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Receiver<"
                                  (:wat::core::keyword/to-string i0)
                                  ">")))
             server-tx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty0 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 0)
                                     "run-threads-n3: missing type arg 0")
                               ch0 (:wat::holon::Bundle/children ty0)
                               o0h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch0 2)
                                     "run-threads-n3: missing O-type child at slot 0:2")
                               o0  (:wat::core::atom-value o0h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Sender<"
                                  (:wat::core::keyword/to-string o0)
                                  ">")))]
            -> :wat::core::nil
            (~factory-0 (:wat::kernel::ThreadPeer/new server-rx server-tx))))
      ~(:wat::holon::to-watast
          (:wat::core::Option/expect -> :wat::holon::HolonAST
            (:wat::core::get
              (:wat::runtime::extract-arg-names
                (:wat::runtime::signature-of-fn coordinator))
              0)
            "run-threads-n3: coordinator has no binder at position 0"))
        (:wat::kernel::ThreadPeer/new
          (:wat::kernel::Thread/output thread-0)
          (:wat::kernel::Thread/input  thread-0))
      thread-1
        (:wat::kernel::spawn-thread
          (:wat::core::fn
            [server-rx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty1 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 1)
                                     "run-threads-n3: missing type arg 1")
                               ch1 (:wat::holon::Bundle/children ty1)
                               i1h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch1 1)
                                     "run-threads-n3: missing I-type child at slot 1:1")
                               i1  (:wat::core::atom-value i1h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Receiver<"
                                  (:wat::core::keyword/to-string i1)
                                  ">")))
             server-tx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty1 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 1)
                                     "run-threads-n3: missing type arg 1")
                               ch1 (:wat::holon::Bundle/children ty1)
                               o1h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch1 2)
                                     "run-threads-n3: missing O-type child at slot 1:2")
                               o1  (:wat::core::atom-value o1h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Sender<"
                                  (:wat::core::keyword/to-string o1)
                                  ">")))]
            -> :wat::core::nil
            (~factory-1 (:wat::kernel::ThreadPeer/new server-rx server-tx))))
      ~(:wat::holon::to-watast
          (:wat::core::Option/expect -> :wat::holon::HolonAST
            (:wat::core::get
              (:wat::runtime::extract-arg-names
                (:wat::runtime::signature-of-fn coordinator))
              1)
            "run-threads-n3: coordinator has no binder at position 1"))
        (:wat::kernel::ThreadPeer/new
          (:wat::kernel::Thread/output thread-1)
          (:wat::kernel::Thread/input  thread-1))
      thread-2
        (:wat::kernel::spawn-thread
          (:wat::core::fn
            [server-rx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty2 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 2)
                                     "run-threads-n3: missing type arg 2")
                               ch2 (:wat::holon::Bundle/children ty2)
                               i2h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch2 1)
                                     "run-threads-n3: missing I-type child at slot 2:1")
                               i2  (:wat::core::atom-value i2h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Receiver<"
                                  (:wat::core::keyword/to-string i2)
                                  ">")))
             server-tx <- ~(:wat::core::let
                              [sig (:wat::runtime::signature-of-fn coordinator)
                               tys (:wat::runtime::extract-arg-types sig)
                               ty2 (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get tys 2)
                                     "run-threads-n3: missing type arg 2")
                               ch2 (:wat::holon::Bundle/children ty2)
                               o2h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                     (:wat::core::get ch2 2)
                                     "run-threads-n3: missing O-type child at slot 2:2")
                               o2  (:wat::core::atom-value o2h)]
                              (:wat::core::keyword/from-string
                                (:wat::core::string::concat
                                  "rust::crossbeam_channel::Sender<"
                                  (:wat::core::keyword/to-string o2)
                                  ">")))]
            -> :wat::core::nil
            (~factory-2 (:wat::kernel::ThreadPeer/new server-rx server-tx))))
      ~(:wat::holon::to-watast
          (:wat::core::Option/expect -> :wat::holon::HolonAST
            (:wat::core::get
              (:wat::runtime::extract-arg-names
                (:wat::runtime::signature-of-fn coordinator))
              2)
            "run-threads-n3: coordinator has no binder at position 2"))
        (:wat::kernel::ThreadPeer/new
          (:wat::kernel::Thread/output thread-2)
          (:wat::kernel::Thread/input  thread-2))
      result
        (~coordinator
          ~@(:wat::runtime::extract-arg-names
               (:wat::runtime::signature-of-fn coordinator)))
      _drained-0
        (:wat::kernel::Thread/drain-and-join thread-0)
      _drained-1
        (:wat::kernel::Thread/drain-and-join thread-1)
      _drained-2
        (:wat::kernel::Thread/drain-and-join thread-2)]
     result))

;; ─── Top-level variadic coordinator-fn macro ───────────────────────────
;;
;; Public surface: accepts coordinator fn-form + N factory call-forms
;; (N=1 or N=3). Dispatches to run-threads-n1 or run-threads-n3 via
;; a computed-unquote that:
;;   1. Evals coordinator form → fn value → signature-of-fn → counts binders
;;   2. Quotes the variadic factories rest-list → from-watast → Bundle/children
;;      → extracts factory 0 (and 1/2 for N=3) as Value::wat__WatAST
;;   3. Returns Value::wat__WatAST of the n1/n3 macro call
;;   4. Re-expansion fires the n1/n3 macro to produce the final let form
;;
;; Only substrate primitives are used at expand time (no user defns needed).
;;
;; STOP-trigger-1 honest disclosure: the BRIEF described naming binders
;; thread-{name}/peer-{name}/_drained-{name}. This requires producing
;; WatAST::Symbol("thread-logger") dynamically — blocked by the substrate
;; (keyword/from-string produces WatAST::Keyword which parse_let_binding
;; rejects; no primitive produces WatAST::Symbol from a computed string).
;; Resolution: thread/drain names are literal (thread-0 etc.); peer names
;; come from the coordinator's binder names directly (which ARE valid
;; WatAST::Symbol via extract-arg-names + to-watast).

(:wat::core::defmacro
  (:wat::kernel::run-threads
    (coordinator :AST<wat::core::nil>)
    & (factories :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `~(:wat::core::let
      [;; Count N from coordinator's binder count via reflection chain.
       ;; signature-of-fn evals coordinator inline → fn value, no user env needed.
       rt-sig    (:wat::runtime::signature-of-fn coordinator)
       rt-names  (:wat::runtime::extract-arg-names rt-sig)
       rt-n      (:wat::core::length rt-names)
       ;; Quote the variadic factories rest-list to get its AST, then
       ;; convert through HolonAST to access individual factory forms.
       ;; quasiquote preserves the List([fA,fB,...]) as Value::wat__WatAST.
       ;; from-watast lifts it to HolonAST::Bundle([holonA,holonB,...]).
       ;; Bundle/children gives Vec<HolonAST> for indexed access.
       rt-facs-ast  (:wat::core::quasiquote factories)
       rt-facs-h    (:wat::holon::from-watast rt-facs-ast)
       rt-facs-ch   (:wat::holon::Bundle/children rt-facs-h)
       rt-fac0-h    (:wat::core::Option/expect -> :wat::holon::HolonAST
                      (:wat::core::get rt-facs-ch 0)
                      "run-threads: no factory at position 0")
       ;; to-watast on the HolonAST::Bundle restores the factory call form
       ;; as Value::wat__WatAST for embedding in the inner macro call.
       rt-fac0      (:wat::holon::to-watast rt-fac0-h)]
      ;; Dispatch: N=1 → run-threads-n1; else → run-threads-n3.
      ;; The if is evaluated at expand time; only the matching branch runs.
      ;; The branch returns Value::wat__WatAST of the n-specific macro call.
      ;; Re-expansion then fires that macro to produce the final let form.
      (:wat::core::if
        (:wat::core::= rt-n 1)
        -> :wat::WatAST
        (:wat::core::quasiquote
          (:wat::kernel::run-threads-n1
            coordinator
            (:wat::core::unquote rt-fac0)))
        (:wat::core::let
          [rt-fac1-h   (:wat::core::Option/expect -> :wat::holon::HolonAST
                          (:wat::core::get rt-facs-ch 1)
                          "run-threads: no factory at position 1")
           rt-fac1     (:wat::holon::to-watast rt-fac1-h)
           rt-fac2-h   (:wat::core::Option/expect -> :wat::holon::HolonAST
                          (:wat::core::get rt-facs-ch 2)
                          "run-threads: no factory at position 2")
           rt-fac2     (:wat::holon::to-watast rt-fac2-h)]
          (:wat::core::quasiquote
            (:wat::kernel::run-threads-n3
              coordinator
              (:wat::core::unquote rt-fac0)
              (:wat::core::unquote rt-fac1)
              (:wat::core::unquote rt-fac2)))))))
