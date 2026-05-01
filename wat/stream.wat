;; :wat::std::stream — composable stage combinators over
;; :wat::kernel::spawn-thread + crossbeam channels. Each combinator
;; spawns one worker thread and wires a bounded(1) queue; the
;; combinator returns a :wat::stream::Stream<T> — the pair
;; (Receiver, Thread<(),()>) the caller composes or terminates.
;;
;; Shape:
;;
;;   (let* (((rx1 h1) (stream::spawn-producer :my::source))
;;          ((rx2 h2) (stream::map rx1 :my::transform))
;;          ((result :()) (stream::for-each rx2 :my::handler))
;;          ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
;;           (:wat::kernel::Thread/join-result h2))
;;          ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
;;           (:wat::kernel::Thread/join-result h1)))
;;     result)
;;
;; The drop cascade (FOUNDATION § Pipeline Discipline) handles
;; shutdown: when the terminal combinator exits, its Receiver drops,
;; the upstream's Sender sees :None on the next send, the upstream
;; exits, its receiver drops, the next-upstream's sender sees :None,
;; etc. Joins confirm each stage exited cleanly.
;;
;; Terminal combinators (:for-each, :collect) drive the pipeline by
;; pulling until the receiver disconnects. They join their OWN stage
;; handle before returning; upstream handles the caller joins.
;;
;; Arc 114: workers fit `:Fn(:Receiver<()>, :Sender<()>) -> :()` and
;; close over caller-allocated bounded(1) channels (mini-TCP via
;; paired channels — docs/ZERO-MUTEX.md). The substrate-allocated
;; `_in` / `_out` stay unused; data flows through the closed-over
;; channels.

;; crossbeam_channel is wat substrate (the runtime's channel
;; implementation), not an external Rust crate dependency. `use!`
;; is for declaring intent to consume #[wat_dispatch]'d external
;; libraries; substrate types the runtime already exposes don't
;; need it.

;; Stream<T> — a live channel + the handle to the producer feeding
;; it. Same shape as the Console / Cache stdlib programs return
;; (HandlePool, driver-handle). Alias expansion (2026-04-20) makes
;; the tuple and this name interchangeable at unification.
;;
;; Arc 114: the handle is a Thread<(),()> — the worker fits the
;; spawn-thread body shape (closes over caller-held channels; the
;; substrate's `_in` / `_out` are unit-typed and unused).
(:wat::core::typealias
  :wat::stream::Stream<T>
  :(wat::kernel::QueueReceiver<T>,wat::kernel::Thread<wat::core::unit,wat::core::unit>))

;; Producer<T> — the function shape spawn-producer accepts: takes the
;; Sender end of a bounded queue, writes values, returns when done.
(:wat::core::typealias
  :wat::stream::Producer<T>
  :fn(wat::kernel::QueueSender<T>)->wat::core::unit)

;; --- with-state step shapes ---
;;
;; Buffer-based stream stages (chunks, window, chunks-by) carry an
;; accumulator + an emit list per step. Two recurring shapes:
;;
;;   ChunkStep<T>           — chunks / window      : (buf,             emits)
;;   KeyedChunkStep<K,T>    — chunks-by            : ((wat::core::Option<K>,buf), emits)
;;
;; Each `:wat::core::Tuple` step returns one of these. Naming the
;; shapes keeps lambda return-type annotations from accumulating
;; nested `<>`s at every site.
(:wat::core::typealias
  :wat::stream::ChunkStep<T>
  :(wat::core::Vector<T>,wat::core::Vector<wat::core::Vector<T>>))

(:wat::core::typealias
  :wat::stream::KeyedChunkStep<K,T>
  :((wat::core::Option<K>,wat::core::Vector<T>),wat::core::Vector<wat::core::Vector<T>>))

;; --- spawn-producer ---
;;
;; Accepts a producer function of signature `Producer<T>`
;; (i.e., `:fn(Sender<T>) -> :()`) — it writes values to the sender
;; until done, then returns. Spawn wires the bounded(1) queue and
;; returns the Stream<T> to the caller. Caller consumes via a
;; combinator or a direct recv loop.
(:wat::core::define
  (:wat::stream::spawn-producer<T>
    (producer :wat::stream::Producer<T>)
    -> :wat::stream::Stream<T>)
  (:wat::core::let*
    (((pair :wat::kernel::QueuePair<T>)
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :wat::kernel::QueueSender<T>) (:wat::core::first pair))
     ((rx :wat::kernel::QueueReceiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (producer tx)))))
    (:wat::core::Tuple rx handle)))

;; --- from-receiver ---
;;
;; Wrap an externally-obtained `Receiver<T>` and its producer's
;; `ProgramHandle<()>` into a `Stream<T>`. Trivial tuple-wrap; no
;; worker spawned. Used when the caller already owns a spawned
;; producer (or an equivalent thread) and wants to plug its output
;; into the stream-stdlib combinators.
;;
;; Both arguments are required. Stream<T>'s typealias includes the
;; handle so downstream `for-each` / `collect` / `fold` can join it.
;; If the caller doesn't have a handle, they don't have a stream —
;; they have a bare Receiver, and some other thread will never be
;; joined, which is a broken shutdown story. Don't hide that.
(:wat::core::define
  (:wat::stream::from-receiver<T>
    (rx :wat::kernel::QueueReceiver<T>)
    (handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
    -> :wat::stream::Stream<T>)
  (:wat::core::Tuple rx handle))

;; --- map ---
;;
;; 1:1 transform. Spawns a worker that pulls from upstream, applies
;; `f`, sends the result downstream. Returns the new Stream<U>. The
;; worker is the canonical tail-recursive stage shape — on :None
;; from upstream, it exits; on :None from its own send (consumer
;; dropped), it exits; otherwise it recurses.

(:wat::core::define
  (:wat::stream::map-worker<T,U>
    (in :wat::kernel::QueueReceiver<T>)
    (out :wat::kernel::QueueSender<U>)
    (f :fn(T)->U)
    -> :wat::core::unit)
  (:wat::core::match (:wat::kernel::recv in) -> :wat::core::unit
    ((:wat::core::Ok (:wat::core::Some v))
      (:wat::core::let*
        (((u :U) (f v)))
        (:wat::core::match (:wat::kernel::send out u) -> :wat::core::unit
          ((:wat::core::Ok _) (:wat::stream::map-worker in out f))
          ((:wat::core::Err _) ()))))
    ((:wat::core::Ok :wat::core::None) ())
    ((:wat::core::Err _died) ())))

(:wat::core::define
  (:wat::stream::map<T,U>
    (upstream :wat::stream::Stream<T>)
    (f :fn(T)->U)
    -> :wat::stream::Stream<U>)
  (:wat::core::let*
    (((up-rx :wat::kernel::QueueReceiver<T>) (:wat::core::first upstream))
     ((pair :wat::kernel::QueuePair<U>)
      (:wat::kernel::make-bounded-queue :U 1))
     ((tx :wat::kernel::QueueSender<U>) (:wat::core::first pair))
     ((rx :wat::kernel::QueueReceiver<U>) (:wat::core::second pair))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::stream::map-worker up-rx tx f)))))
    (:wat::core::Tuple rx handle)))

;; --- for-each ---
;;
;; Terminal. Pulls from the stream, applies the handler for its
;; side effect, continues until disconnect. Joins the handle and
;; returns :(). Drives the pipeline to completion on the calling
;; thread — no new worker spawned here.

(:wat::core::define
  (:wat::stream::for-each-drain<T>
    (rx :wat::kernel::QueueReceiver<T>)
    (handler :fn(T)->wat::core::unit)
    -> :wat::core::unit)
  (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::unit
    ((:wat::core::Ok (:wat::core::Some v))
      (:wat::core::let*
        (((_ :wat::core::unit) (handler v)))
        (:wat::stream::for-each-drain rx handler)))
    ((:wat::core::Ok :wat::core::None) ())
    ((:wat::core::Err _died) ())))

(:wat::core::define
  (:wat::stream::for-each<T>
    (stream :wat::stream::Stream<T>)
    (handler :fn(T)->wat::core::unit)
    -> :wat::core::unit)
  (:wat::core::let*
    (((rx :wat::kernel::QueueReceiver<T>) (:wat::core::first stream))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second stream))
     ((_ :wat::core::unit) (:wat::stream::for-each-drain rx handler))
     ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result handle)))
    ()))

;; --- collect ---
;;
;; Terminal. Accumulates every item into a wat::core::Vector<T>, joins the handle,
;; returns the Vec. Useful as a test sink and for bounded pipelines
;; whose output fits in memory. For unbounded or large streams, use
;; for-each or a fold-style terminal instead.

(:wat::core::define
  (:wat::stream::collect-drain<T>
    (rx :wat::kernel::QueueReceiver<T>)
    (acc :wat::core::Vector<T>)
    -> :wat::core::Vector<T>)
  (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::Vector<T>
    ((:wat::core::Ok (:wat::core::Some v))
      (:wat::stream::collect-drain rx (:wat::core::conj acc v)))
    ((:wat::core::Ok :wat::core::None) acc)
    ((:wat::core::Err _died) acc)))

(:wat::core::define
  (:wat::stream::collect<T>
    (stream :wat::stream::Stream<T>)
    -> :wat::core::Vector<T>)
  (:wat::core::let*
    (((rx :wat::kernel::QueueReceiver<T>) (:wat::core::first stream))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second stream))
     ((items :wat::core::Vector<T>)
      (:wat::stream::collect-drain rx (:wat::core::Vector :T)))
     ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result handle)))
    items))

;; --- filter ---
;;
;; 1:0..1. Spawns a worker that pulls from upstream; for each item,
;; calls the predicate; forwards only items for which it returned true.
;; Same tail-recursive shape as map. Empty downstream drops.
(:wat::core::define
  (:wat::stream::filter-worker<T>
    (in :wat::kernel::QueueReceiver<T>)
    (out :wat::kernel::QueueSender<T>)
    (pred :fn(T)->wat::core::bool)
    -> :wat::core::unit)
  (:wat::core::match (:wat::kernel::recv in) -> :wat::core::unit
    ((:wat::core::Ok (:wat::core::Some v))
      (:wat::core::if (pred v) -> :wat::core::unit
        (:wat::core::match (:wat::kernel::send out v) -> :wat::core::unit
          ((:wat::core::Ok _) (:wat::stream::filter-worker in out pred))
          ((:wat::core::Err _) ()))
        (:wat::stream::filter-worker in out pred)))
    ((:wat::core::Ok :wat::core::None) ())
    ((:wat::core::Err _died) ())))

(:wat::core::define
  (:wat::stream::filter<T>
    (upstream :wat::stream::Stream<T>)
    (pred :fn(T)->wat::core::bool)
    -> :wat::stream::Stream<T>)
  (:wat::core::let*
    (((up-rx :wat::kernel::QueueReceiver<T>) (:wat::core::first upstream))
     ((pair :wat::kernel::QueuePair<T>)
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :wat::kernel::QueueSender<T>) (:wat::core::first pair))
     ((rx :wat::kernel::QueueReceiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::stream::filter-worker up-rx tx pred)))))
    (:wat::core::Tuple rx handle)))

;; --- inspect ---
;;
;; 1:1 side-effect pass-through. Spawns a worker that pulls from
;; upstream, calls `f` for its effect (return type :()), and forwards
;; the ORIGINAL value unchanged. Same shape as map except the worker
;; ignores f's return and sends v instead of (f v). Debugging
;; ergonomics: drop an inspect into a pipeline to log / count / trace
;; without perturbing the values.
(:wat::core::define
  (:wat::stream::inspect-worker<T>
    (in :wat::kernel::QueueReceiver<T>)
    (out :wat::kernel::QueueSender<T>)
    (f :fn(T)->wat::core::unit)
    -> :wat::core::unit)
  (:wat::core::match (:wat::kernel::recv in) -> :wat::core::unit
    ((:wat::core::Ok (:wat::core::Some v))
      (:wat::core::let*
        (((_ :wat::core::unit) (f v)))
        (:wat::core::match (:wat::kernel::send out v) -> :wat::core::unit
          ((:wat::core::Ok _) (:wat::stream::inspect-worker in out f))
          ((:wat::core::Err _) ()))))
    ((:wat::core::Ok :wat::core::None) ())
    ((:wat::core::Err _died) ())))

(:wat::core::define
  (:wat::stream::inspect<T>
    (upstream :wat::stream::Stream<T>)
    (f :fn(T)->wat::core::unit)
    -> :wat::stream::Stream<T>)
  (:wat::core::let*
    (((up-rx :wat::kernel::QueueReceiver<T>) (:wat::core::first upstream))
     ((pair :wat::kernel::QueuePair<T>)
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :wat::kernel::QueueSender<T>) (:wat::core::first pair))
     ((rx :wat::kernel::QueueReceiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::stream::inspect-worker up-rx tx f)))))
    (:wat::core::Tuple rx handle)))

;; --- fold ---
;;
;; Terminal. General reduction: every item folds into an accumulator
;; with the caller's function. Generalizes collect (which is
;; `fold init=[] f=conj`) and gives sum / count / any / all as
;; one-liners. Joins the handle; returns the final accumulator.
(:wat::core::define
  (:wat::stream::fold-drain<T,Acc>
    (rx :wat::kernel::QueueReceiver<T>)
    (acc :Acc)
    (f :fn(Acc,T)->Acc)
    -> :Acc)
  (:wat::core::match (:wat::kernel::recv rx) -> :Acc
    ((:wat::core::Ok (:wat::core::Some v))
      (:wat::stream::fold-drain rx (f acc v) f))
    ((:wat::core::Ok :wat::core::None) acc)
    ((:wat::core::Err _died) acc)))

(:wat::core::define
  (:wat::stream::fold<T,Acc>
    (stream :wat::stream::Stream<T>)
    (init :Acc)
    (f :fn(Acc,T)->Acc)
    -> :Acc)
  (:wat::core::let*
    (((rx :wat::kernel::QueueReceiver<T>) (:wat::core::first stream))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second stream))
     ((result :Acc) (:wat::stream::fold-drain rx init f))
     ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result handle)))
    result))

;; --- chunks ---
;;
;; N:1 batcher. Accumulates items into a Vec until it holds `size`
;; entries, then emits the Vec as one downstream item and starts a
;; fresh accumulator. At end-of-stream (upstream :None), flushes
;; the partial accumulator if non-empty. This is the canonical
;; stateful-stage pattern: state threads through the tail-recursive
;; worker as a parameter (no mutation; the recursion carries it).
;; --- with-state ---
;;
;; The Mealy-machine stream stage. Every stateful stage reducer
;; (chunks, chunks-by, chunks-while, window, dedupe, distinct-until-
;; changed, sessionize, rate-limit, running-stats, ...) is a triple
;; (init, step, flush) over with-state.
;;
;;   init  :Acc
;;   step  :fn(Acc,T) -> :(Acc, wat::core::Vector<U>)
;;   flush :fn(Acc)   -> :wat::core::Vector<U>
;;
;; Worker semantics: each upstream item threads through `step`, which
;; returns the new state and zero-or-more items to emit. On upstream
;; EOS, `flush` is called on the final state and its wat::core::Vector<U> is
;; drained downstream before the worker exits.
;;
;; Convergence with prior art — Elixir's Stream.transform/3, Rust's
;; scan-with-emit, Haskell's mapAccumL, Mealy's 1955 sequential-
;; circuit state machine. Arc 006 BACKLOG's resolution named this
;; shape as the substrate every stateful-stage combinator wants.

(:wat::core::define
  (:wat::stream::drain-items<U>
    (out :wat::kernel::QueueSender<U>)
    (items :wat::core::Vector<U>)
    -> :wat::core::Option<wat::core::unit>)
  ;; Tail-recursive helper: send every item in `items`, stop early
  ;; (returning :None) if the consumer dropped. Returns (Some ()) on
  ;; full drain; returns :None if any send failed, signaling the
  ;; caller to exit.
  (:wat::core::if (:wat::core::empty? items) -> :wat::core::Option<wat::core::unit>
    (:wat::core::Some ())
    ;; Vec is non-empty (just checked); first returns Some<U> via
    ;; arc 047. The :None arm is unreachable but the type checker
    ;; demands totality.
    (:wat::core::match (:wat::core::first items) -> :wat::core::Option<wat::core::unit>
      ((:wat::core::Some item)
        (:wat::core::let*
          (((rest-items :wat::core::Vector<U>) (:wat::core::rest items)))
          (:wat::core::match (:wat::kernel::send out item) -> :wat::core::Option<wat::core::unit>
            ((:wat::core::Ok _)
              (:wat::stream::drain-items out rest-items))
            ((:wat::core::Err _) :wat::core::None))))
      (:wat::core::None :wat::core::None))))

(:wat::core::define
  (:wat::stream::with-state-worker<T,U,Acc>
    (in :wat::kernel::QueueReceiver<T>)
    (out :wat::kernel::QueueSender<U>)
    (step :fn(Acc,T)->(Acc,wat::core::Vector<U>))
    (flush :fn(Acc)->wat::core::Vector<U>)
    (acc :Acc)
    -> :wat::core::unit)
  (:wat::core::match (:wat::kernel::recv in) -> :wat::core::unit
    ((:wat::core::Ok (:wat::core::Some item))
      (:wat::core::let*
        (((stepped :(Acc,wat::core::Vector<U>)) (step acc item))
         ((new-acc :Acc) (:wat::core::first stepped))
         ((to-emit :wat::core::Vector<U>) (:wat::core::second stepped))
         ((drain-result :wat::core::Option<wat::core::unit>)
          (:wat::stream::drain-items out to-emit)))
        (:wat::core::match drain-result -> :wat::core::unit
          ((:wat::core::Some _)
            (:wat::stream::with-state-worker in out step flush new-acc))
          (:wat::core::None ()))))
    ((:wat::core::Ok :wat::core::None)
      ;; Upstream disconnected. Flush final state; drain whatever it
      ;; produced. Consumer-dropped during flush is swallowed silently
      ;; — same behavior chunks had for its final partial buffer.
      (:wat::core::let*
        (((final-emits :wat::core::Vector<U>) (flush acc))
         ((_ :wat::core::Option<wat::core::unit>)
          (:wat::stream::drain-items out final-emits)))
        ()))
    ((:wat::core::Err _died) ())))

(:wat::core::define
  (:wat::stream::with-state<T,U,Acc>
    (upstream :wat::stream::Stream<T>)
    (init :Acc)
    (step :fn(Acc,T)->(Acc,wat::core::Vector<U>))
    (flush :fn(Acc)->wat::core::Vector<U>)
    -> :wat::stream::Stream<U>)
  (:wat::core::let*
    (((up-rx :wat::kernel::QueueReceiver<T>) (:wat::core::first upstream))
     ((pair :wat::kernel::QueuePair<U>)
      (:wat::kernel::make-bounded-queue :U 1))
     ((tx :wat::kernel::QueueSender<U>) (:wat::core::first pair))
     ((rx :wat::kernel::QueueReceiver<U>) (:wat::core::second pair))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::stream::with-state-worker up-rx tx step flush init)))))
    (:wat::core::Tuple rx handle)))

;; --- chunks (rewritten on top of with-state) ---
;;
;; Surface-reduction proof of the with-state decomposition. The N:1
;; batcher's triple:
;;
;;   init  = empty wat::core::Vector<T>
;;   step  = (buf, item) ->
;;             if len(buf)+1 == size: (empty, [buf++[item]])
;;             else:                  (buf++[item], [])
;;   flush = buf -> if empty: [] else: [buf]
;;
;; Arc 006 inscription note: the pre-with-state chunks-worker was a
;; standalone tail-recursive state machine. Same semantics, different
;; factoring — the state transitions now live in step/flush lambdas
;; instead of in-worker branches.

(:wat::core::define
  (:wat::stream::chunks-step<T>
    (buffer :wat::core::Vector<T>)
    (item :T)
    (size :wat::core::i64)
    -> :wat::stream::ChunkStep<T>)
  (:wat::core::let*
    (((new-buffer :wat::core::Vector<T>) (:wat::core::conj buffer item)))
    (:wat::core::if (:wat::core::>= (:wat::core::length new-buffer) size)
      -> :wat::stream::ChunkStep<T>
      (:wat::core::Tuple
        (:wat::core::Vector :T)
        (:wat::core::Vector :wat::core::Vector<T> new-buffer))
      (:wat::core::Tuple
        new-buffer
        (:wat::core::Vector :wat::core::Vector<T>)))))

(:wat::core::define
  (:wat::stream::chunks-flush<T>
    (buffer :wat::core::Vector<T>)
    -> :wat::core::Vector<wat::core::Vector<T>>)
  (:wat::core::if (:wat::core::empty? buffer) -> :wat::core::Vector<wat::core::Vector<T>>
    (:wat::core::Vector :wat::core::Vector<T>)
    (:wat::core::Vector :wat::core::Vector<T> buffer)))

(:wat::core::define
  (:wat::stream::chunks<T>
    (upstream :wat::stream::Stream<T>)
    (size :wat::core::i64)
    -> :wat::stream::Stream<wat::core::Vector<T>>)
  ;; chunks-step takes (buf, item, size) — three args — but with-state
  ;; wants (buf, item). The `size` parameter has to close over the
  ;; chunks caller's argument, so step is genuinely a lambda capturing
  ;; `size`, not a pass-through. chunks-flush takes (buf) exactly, so
  ;; it passes by name directly (arc 009 — names are values).
  (:wat::stream::with-state upstream
    (:wat::core::Vector :T)
    (:wat::core::lambda ((buf :wat::core::Vector<T>) (item :T) -> :wat::stream::ChunkStep<T>)
      (:wat::stream::chunks-step buf item size))
    :wat::stream::chunks-flush))

;; --- chunks-by ---
;;
;; N:1 with key-fn boundary. Groups consecutive items sharing the
;; same key into one Vec; emits when the key changes; flushes the
;; final group at EOS. Clojure's `partition-by` shape; named in
;; arc 006's INSCRIPTION as `init = (None, [])` over with-state.
;;
;; Equality on K uses :wat::core::= (polymorphic, structural over
;; primitives and composite values).

(:wat::core::define
  (:wat::stream::chunks-by-step<T,K>
    (state :(wat::core::Option<K>,wat::core::Vector<T>))
    (item :T)
    (key-fn :fn(T)->K)
    -> :wat::stream::KeyedChunkStep<K,T>)
  (:wat::core::let*
    (((last-key :wat::core::Option<K>) (:wat::core::first state))
     ((buffer :wat::core::Vector<T>) (:wat::core::second state))
     ((k :K) (key-fn item)))
    (:wat::core::match last-key -> :wat::stream::KeyedChunkStep<K,T>
      (:wat::core::None
        ;; First item — start the run, emit nothing yet.
        (:wat::core::Tuple
          (:wat::core::Tuple (:wat::core::Some k) (:wat::core::Vector :T item))
          (:wat::core::Vector :wat::core::Vector<T>)))
      ((:wat::core::Some prev)
        (:wat::core::if (:wat::core::= prev k)
          -> :wat::stream::KeyedChunkStep<K,T>
          ;; Same key — append to current run, emit nothing.
          (:wat::core::Tuple
            (:wat::core::Tuple (:wat::core::Some k) (:wat::core::conj buffer item))
            (:wat::core::Vector :wat::core::Vector<T>))
          ;; Key change — emit completed run, start new run.
          (:wat::core::Tuple
            (:wat::core::Tuple (:wat::core::Some k) (:wat::core::Vector :T item))
            (:wat::core::Vector :wat::core::Vector<T> buffer)))))))

(:wat::core::define
  (:wat::stream::chunks-by-flush<T,K>
    (state :(wat::core::Option<K>,wat::core::Vector<T>))
    -> :wat::core::Vector<wat::core::Vector<T>>)
  (:wat::core::let*
    (((buffer :wat::core::Vector<T>) (:wat::core::second state)))
    (:wat::core::if (:wat::core::empty? buffer) -> :wat::core::Vector<wat::core::Vector<T>>
      (:wat::core::Vector :wat::core::Vector<T>)
      (:wat::core::Vector :wat::core::Vector<T> buffer))))

(:wat::core::define
  (:wat::stream::chunks-by<T,K>
    (upstream :wat::stream::Stream<T>)
    (key-fn :fn(T)->K)
    -> :wat::stream::Stream<wat::core::Vector<T>>)
  ;; init = (None, empty) — no key seen yet, no items buffered.
  ;; step closes over key-fn; flush is size-agnostic so passes by name.
  (:wat::stream::with-state upstream
    (:wat::core::Tuple :wat::core::None (:wat::core::Vector :T))
    (:wat::core::lambda ((state :(wat::core::Option<K>,wat::core::Vector<T>)) (item :T)
                         -> :wat::stream::KeyedChunkStep<K,T>)
      (:wat::stream::chunks-by-step state item key-fn))
    :wat::stream::chunks-by-flush))

;; --- window ---
;;
;; Sliding window, step=1. Emits every full-size window as items
;; arrive. Matching arc 006's INSCRIPTION and the book's Ruby-example
;; discipline (*don't silently drop data at EOS*), the flush rule is:
;; emit the partial buffer ONLY if the stream was shorter than `size`
;; (buffer was never emitted as a full window). In every other case
;; the last full window was already emitted on the sliding path and
;; flush stays empty.
;;
;; For step>1 or other sliding behaviors, callers author their own
;; with-state directly. Same stdlib-as-blueprint discipline — the
;; named combinator ships one honest choice; richer shapes earn their
;; slots when real callers demand them.

(:wat::core::define
  (:wat::stream::window-step<T>
    (buffer :wat::core::Vector<T>)
    (item :T)
    (size :wat::core::i64)
    -> :wat::stream::ChunkStep<T>)
  (:wat::core::let*
    (((new-buf :wat::core::Vector<T>) (:wat::core::conj buffer item))
     ((new-len :wat::core::i64) (:wat::core::length new-buf)))
    (:wat::core::cond -> :wat::stream::ChunkStep<T>
      ;; Over-size — slide: drop first, emit trimmed window, keep trimmed.
      ((:wat::core::> new-len size)
        (:wat::core::let*
          (((trimmed :wat::core::Vector<T>) (:wat::core::rest new-buf)))
          (:wat::core::Tuple trimmed (:wat::core::Vector :wat::core::Vector<T> trimmed))))
      ;; Exactly size — first full window. Emit and keep.
      ((:wat::core::= new-len size)
        (:wat::core::Tuple new-buf (:wat::core::Vector :wat::core::Vector<T> new-buf)))
      ;; Under-size — still warming up. No emit.
      (:else
        (:wat::core::Tuple new-buf (:wat::core::Vector :wat::core::Vector<T>))))))

(:wat::core::define
  (:wat::stream::window-flush<T>
    (buffer :wat::core::Vector<T>)
    (size :wat::core::i64)
    -> :wat::core::Vector<wat::core::Vector<T>>)
  ;; Flush-partial IFF buffer contains items that were never emitted
  ;; as a full window. That's exactly the case len(buf) < size AND
  ;; len(buf) > 0. The len == size case means a full window was
  ;; already emitted on the sliding path — nothing to flush.
  (:wat::core::cond -> :wat::core::Vector<wat::core::Vector<T>>
    ((:wat::core::empty? buffer) (:wat::core::Vector :wat::core::Vector<T>))
    ((:wat::core::< (:wat::core::length buffer) size)
      (:wat::core::Vector :wat::core::Vector<T> buffer))
    (:else (:wat::core::Vector :wat::core::Vector<T>))))

(:wat::core::define
  (:wat::stream::window<T>
    (upstream :wat::stream::Stream<T>)
    (size :wat::core::i64)
    -> :wat::stream::Stream<wat::core::Vector<T>>)
  ;; Both step and flush close over size — two lambda wrappers.
  (:wat::stream::with-state upstream
    (:wat::core::Vector :T)
    (:wat::core::lambda ((buf :wat::core::Vector<T>) (item :T) -> :wat::stream::ChunkStep<T>)
      (:wat::stream::window-step buf item size))
    (:wat::core::lambda ((buf :wat::core::Vector<T>) -> :wat::core::Vector<wat::core::Vector<T>>)
      (:wat::stream::window-flush buf size))))

;; --- take ---
;;
;; Stage, not terminal. Forwards the first `n` items from upstream,
;; then exits. The worker's exit drops its Sender (downstream sees
;; :None) and its Receiver (upstream's next send returns :None,
;; upstream exits). Drop cascade fires naturally via spawn scope
;; exit; no kernel-level force-drop needed. See arc 006 BACKLOG for
;; the reasoning that forced the stage (vs terminal) framing.
;;
;; `n <= 0` emits nothing (worker exits immediately). Upstream
;; ending before `n` is reached is fine — worker sees :None on recv,
;; exits, downstream gets :None naturally.
(:wat::core::define
  (:wat::stream::take-worker<T>
    (in :wat::kernel::QueueReceiver<T>)
    (out :wat::kernel::QueueSender<T>)
    (remaining :wat::core::i64)
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::<= remaining 0) -> :wat::core::unit
    ()
    (:wat::core::match (:wat::kernel::recv in) -> :wat::core::unit
      ((:wat::core::Ok (:wat::core::Some v))
        (:wat::core::match (:wat::kernel::send out v) -> :wat::core::unit
          ((:wat::core::Ok _)
            (:wat::stream::take-worker in out
              (:wat::core::i64::- remaining 1)))
          ((:wat::core::Err _) ())))
      ((:wat::core::Ok :wat::core::None) ())
      ((:wat::core::Err _died) ()))))

(:wat::core::define
  (:wat::stream::take<T>
    (upstream :wat::stream::Stream<T>)
    (n :wat::core::i64)
    -> :wat::stream::Stream<T>)
  (:wat::core::let*
    (((up-rx :wat::kernel::QueueReceiver<T>) (:wat::core::first upstream))
     ((pair :wat::kernel::QueuePair<T>)
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :wat::kernel::QueueSender<T>) (:wat::core::first pair))
     ((rx :wat::kernel::QueueReceiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::stream::take-worker up-rx tx n)))))
    (:wat::core::Tuple rx handle)))

;; --- flat-map ---
;;
;; 1:N expansion. For each upstream item, apply `f` to get a wat::core::Vector<U>;
;; emit each element downstream. Empty result emits nothing for that
;; input (0:1 sub-case). Symmetric with chunks (N:1).
;;
;; State machine: the worker carries a `pending` buffer of items
;; produced by the most recent (f v) expansion that haven't been
;; sent yet. When pending is empty, pull the next upstream item and
;; expand. When pending has items, send the first and recurse with
;; the rest. One function, state threaded through the parameter —
;; the same pattern chunks uses for its accumulator.
(:wat::core::define
  (:wat::stream::flat-map-worker<T,U>
    (in :wat::kernel::QueueReceiver<T>)
    (out :wat::kernel::QueueSender<U>)
    (f :fn(T)->wat::core::Vector<U>)
    (pending :wat::core::Vector<U>)
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::empty? pending) -> :wat::core::unit
    (:wat::core::match (:wat::kernel::recv in) -> :wat::core::unit
      ((:wat::core::Ok (:wat::core::Some v))
        (:wat::stream::flat-map-worker in out f (f v)))
      ((:wat::core::Ok :wat::core::None) ())
      ((:wat::core::Err _died) ()))
    ;; pending is non-empty; first returns Some<U> via arc 047.
    ;; :None arm is unreachable but type-required.
    (:wat::core::match (:wat::core::first pending) -> :wat::core::unit
      ((:wat::core::Some item)
        (:wat::core::let*
          (((rest-items :wat::core::Vector<U>) (:wat::core::rest pending)))
          (:wat::core::match (:wat::kernel::send out item) -> :wat::core::unit
            ((:wat::core::Ok _)
              (:wat::stream::flat-map-worker in out f rest-items))
            ((:wat::core::Err _) ()))))
      (:wat::core::None ()))))

(:wat::core::define
  (:wat::stream::flat-map<T,U>
    (upstream :wat::stream::Stream<T>)
    (f :fn(T)->wat::core::Vector<U>)
    -> :wat::stream::Stream<U>)
  (:wat::core::let*
    (((up-rx :wat::kernel::QueueReceiver<T>) (:wat::core::first upstream))
     ((pair :wat::kernel::QueuePair<U>)
      (:wat::kernel::make-bounded-queue :U 1))
     ((tx :wat::kernel::QueueSender<U>) (:wat::core::first pair))
     ((rx :wat::kernel::QueueReceiver<U>) (:wat::core::second pair))
     ((handle :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::kernel::spawn-thread
        (:wat::core::lambda
          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
           -> :wat::core::unit)
          (:wat::stream::flat-map-worker up-rx tx f (:wat::core::Vector :U))))))
    (:wat::core::Tuple rx handle)))
