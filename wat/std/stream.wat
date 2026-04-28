;; :wat::std::stream — composable stage combinators over
;; :wat::kernel::spawn + crossbeam channels. Each combinator spawns
;; one worker program and wires a bounded(1) queue; the combinator
;; returns a :wat::std::stream::Stream<T> — the pair (Receiver,
;; ProgramHandle) the caller composes or terminates.
;;
;; Shape:
;;
;;   (let* (((rx1 h1) (stream::spawn-producer :my::source))
;;          ((rx2 h2) (stream::map rx1 :my::transform))
;;          ((result :()) (stream::for-each rx2 :my::handler))
;;          ((_ :()) (:wat::kernel::join h2))
;;          ((_ :()) (:wat::kernel::join h1)))
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

;; crossbeam_channel is wat substrate (the runtime's channel
;; implementation), not an external Rust crate dependency. `use!`
;; is for declaring intent to consume #[wat_dispatch]'d external
;; libraries; substrate types the runtime already exposes don't
;; need it.

;; Stream<T> — a live channel + the handle to the producer feeding
;; it. Same shape as the Console / Cache stdlib programs return
;; (HandlePool, driver-handle). Alias expansion (2026-04-20) makes
;; the tuple and this name interchangeable at unification.
(:wat::core::typealias
  :wat::std::stream::Stream<T>
  :(rust::crossbeam_channel::Receiver<T>,wat::kernel::ProgramHandle<()>))

;; Producer<T> — the function shape spawn-producer accepts: takes the
;; Sender end of a bounded queue, writes values, returns when done.
(:wat::core::typealias
  :wat::std::stream::Producer<T>
  :fn(rust::crossbeam_channel::Sender<T>)->())

;; --- with-state step shapes ---
;;
;; Buffer-based stream stages (chunks, window, chunks-by) carry an
;; accumulator + an emit list per step. Two recurring shapes:
;;
;;   ChunkStep<T>           — chunks / window      : (buf,             emits)
;;   KeyedChunkStep<K,T>    — chunks-by            : ((Option<K>,buf), emits)
;;
;; Each `:wat::core::tuple` step returns one of these. Naming the
;; shapes keeps lambda return-type annotations from accumulating
;; nested `<>`s at every site.
(:wat::core::typealias
  :wat::std::stream::ChunkStep<T>
  :(Vec<T>,Vec<Vec<T>>))

(:wat::core::typealias
  :wat::std::stream::KeyedChunkStep<K,T>
  :((Option<K>,Vec<T>),Vec<Vec<T>>))

;; --- spawn-producer ---
;;
;; Accepts a producer function of signature `Producer<T>`
;; (i.e., `:fn(Sender<T>) -> :()`) — it writes values to the sender
;; until done, then returns. Spawn wires the bounded(1) queue and
;; returns the Stream<T> to the caller. Caller consumes via a
;; combinator or a direct recv loop.
(:wat::core::define
  (:wat::std::stream::spawn-producer<T>
    (producer :wat::std::stream::Producer<T>)
    -> :wat::std::stream::Stream<T>)
  (:wat::core::let*
    (((pair :(rust::crossbeam_channel::Sender<T>,rust::crossbeam_channel::Receiver<T>))
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :rust::crossbeam_channel::Sender<T>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn producer tx)))
    (:wat::core::tuple rx handle)))

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
  (:wat::std::stream::from-receiver<T>
    (rx :rust::crossbeam_channel::Receiver<T>)
    (handle :wat::kernel::ProgramHandle<()>)
    -> :wat::std::stream::Stream<T>)
  (:wat::core::tuple rx handle))

;; --- map ---
;;
;; 1:1 transform. Spawns a worker that pulls from upstream, applies
;; `f`, sends the result downstream. Returns the new Stream<U>. The
;; worker is the canonical tail-recursive stage shape — on :None
;; from upstream, it exits; on :None from its own send (consumer
;; dropped), it exits; otherwise it recurses.

(:wat::core::define
  (:wat::std::stream::map-worker<T,U>
    (in :rust::crossbeam_channel::Receiver<T>)
    (out :rust::crossbeam_channel::Sender<U>)
    (f :fn(T)->U)
    -> :())
  (:wat::core::match (:wat::kernel::recv in) -> :()
    ((Some v)
      (:wat::core::let*
        (((u :U) (f v))
         ((sent :Option<()>) (:wat::kernel::send out u)))
        (:wat::core::match sent -> :()
          ((Some _) (:wat::std::stream::map-worker in out f))
          (:None ()))))
    (:None ())))

(:wat::core::define
  (:wat::std::stream::map<T,U>
    (upstream :wat::std::stream::Stream<T>)
    (f :fn(T)->U)
    -> :wat::std::stream::Stream<U>)
  (:wat::core::let*
    (((up-rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first upstream))
     ((pair :(rust::crossbeam_channel::Sender<U>,rust::crossbeam_channel::Receiver<U>))
      (:wat::kernel::make-bounded-queue :U 1))
     ((tx :rust::crossbeam_channel::Sender<U>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<U>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::stream::map-worker up-rx tx f)))
    (:wat::core::tuple rx handle)))

;; --- for-each ---
;;
;; Terminal. Pulls from the stream, applies the handler for its
;; side effect, continues until disconnect. Joins the handle and
;; returns :(). Drives the pipeline to completion on the calling
;; thread — no new worker spawned here.

(:wat::core::define
  (:wat::std::stream::for-each-drain<T>
    (rx :rust::crossbeam_channel::Receiver<T>)
    (handler :fn(T)->())
    -> :())
  (:wat::core::match (:wat::kernel::recv rx) -> :()
    ((Some v)
      (:wat::core::let*
        (((_ :()) (handler v)))
        (:wat::std::stream::for-each-drain rx handler)))
    (:None ())))

(:wat::core::define
  (:wat::std::stream::for-each<T>
    (stream :wat::std::stream::Stream<T>)
    (handler :fn(T)->())
    -> :())
  (:wat::core::let*
    (((rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first stream))
     ((handle :wat::kernel::ProgramHandle<()>) (:wat::core::second stream))
     ((_ :()) (:wat::std::stream::for-each-drain rx handler))
     ((_ :()) (:wat::kernel::join handle)))
    ()))

;; --- collect ---
;;
;; Terminal. Accumulates every item into a Vec<T>, joins the handle,
;; returns the Vec. Useful as a test sink and for bounded pipelines
;; whose output fits in memory. For unbounded or large streams, use
;; for-each or a fold-style terminal instead.

(:wat::core::define
  (:wat::std::stream::collect-drain<T>
    (rx :rust::crossbeam_channel::Receiver<T>)
    (acc :Vec<T>)
    -> :Vec<T>)
  (:wat::core::match (:wat::kernel::recv rx) -> :Vec<T>
    ((Some v)
      (:wat::std::stream::collect-drain rx (:wat::core::conj acc v)))
    (:None acc)))

(:wat::core::define
  (:wat::std::stream::collect<T>
    (stream :wat::std::stream::Stream<T>)
    -> :Vec<T>)
  (:wat::core::let*
    (((rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first stream))
     ((handle :wat::kernel::ProgramHandle<()>) (:wat::core::second stream))
     ((items :Vec<T>)
      (:wat::std::stream::collect-drain rx (:wat::core::vec :T)))
     ((_ :()) (:wat::kernel::join handle)))
    items))

;; --- filter ---
;;
;; 1:0..1. Spawns a worker that pulls from upstream; for each item,
;; calls the predicate; forwards only items for which it returned true.
;; Same tail-recursive shape as map. Empty downstream drops.
(:wat::core::define
  (:wat::std::stream::filter-worker<T>
    (in :rust::crossbeam_channel::Receiver<T>)
    (out :rust::crossbeam_channel::Sender<T>)
    (pred :fn(T)->bool)
    -> :())
  (:wat::core::match (:wat::kernel::recv in) -> :()
    ((Some v)
      (:wat::core::if (pred v) -> :()
        (:wat::core::let*
          (((sent :Option<()>) (:wat::kernel::send out v)))
          (:wat::core::match sent -> :()
            ((Some _) (:wat::std::stream::filter-worker in out pred))
            (:None ())))
        (:wat::std::stream::filter-worker in out pred)))
    (:None ())))

(:wat::core::define
  (:wat::std::stream::filter<T>
    (upstream :wat::std::stream::Stream<T>)
    (pred :fn(T)->bool)
    -> :wat::std::stream::Stream<T>)
  (:wat::core::let*
    (((up-rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first upstream))
     ((pair :(rust::crossbeam_channel::Sender<T>,rust::crossbeam_channel::Receiver<T>))
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :rust::crossbeam_channel::Sender<T>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::stream::filter-worker up-rx tx pred)))
    (:wat::core::tuple rx handle)))

;; --- inspect ---
;;
;; 1:1 side-effect pass-through. Spawns a worker that pulls from
;; upstream, calls `f` for its effect (return type :()), and forwards
;; the ORIGINAL value unchanged. Same shape as map except the worker
;; ignores f's return and sends v instead of (f v). Debugging
;; ergonomics: drop an inspect into a pipeline to log / count / trace
;; without perturbing the values.
(:wat::core::define
  (:wat::std::stream::inspect-worker<T>
    (in :rust::crossbeam_channel::Receiver<T>)
    (out :rust::crossbeam_channel::Sender<T>)
    (f :fn(T)->())
    -> :())
  (:wat::core::match (:wat::kernel::recv in) -> :()
    ((Some v)
      (:wat::core::let*
        (((_ :()) (f v))
         ((sent :Option<()>) (:wat::kernel::send out v)))
        (:wat::core::match sent -> :()
          ((Some _) (:wat::std::stream::inspect-worker in out f))
          (:None ()))))
    (:None ())))

(:wat::core::define
  (:wat::std::stream::inspect<T>
    (upstream :wat::std::stream::Stream<T>)
    (f :fn(T)->())
    -> :wat::std::stream::Stream<T>)
  (:wat::core::let*
    (((up-rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first upstream))
     ((pair :(rust::crossbeam_channel::Sender<T>,rust::crossbeam_channel::Receiver<T>))
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :rust::crossbeam_channel::Sender<T>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::stream::inspect-worker up-rx tx f)))
    (:wat::core::tuple rx handle)))

;; --- fold ---
;;
;; Terminal. General reduction: every item folds into an accumulator
;; with the caller's function. Generalizes collect (which is
;; `fold init=[] f=conj`) and gives sum / count / any / all as
;; one-liners. Joins the handle; returns the final accumulator.
(:wat::core::define
  (:wat::std::stream::fold-drain<T,Acc>
    (rx :rust::crossbeam_channel::Receiver<T>)
    (acc :Acc)
    (f :fn(Acc,T)->Acc)
    -> :Acc)
  (:wat::core::match (:wat::kernel::recv rx) -> :Acc
    ((Some v)
      (:wat::std::stream::fold-drain rx (f acc v) f))
    (:None acc)))

(:wat::core::define
  (:wat::std::stream::fold<T,Acc>
    (stream :wat::std::stream::Stream<T>)
    (init :Acc)
    (f :fn(Acc,T)->Acc)
    -> :Acc)
  (:wat::core::let*
    (((rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first stream))
     ((handle :wat::kernel::ProgramHandle<()>) (:wat::core::second stream))
     ((result :Acc) (:wat::std::stream::fold-drain rx init f))
     ((_ :()) (:wat::kernel::join handle)))
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
;;   step  :fn(Acc,T) -> :(Acc, Vec<U>)
;;   flush :fn(Acc)   -> :Vec<U>
;;
;; Worker semantics: each upstream item threads through `step`, which
;; returns the new state and zero-or-more items to emit. On upstream
;; EOS, `flush` is called on the final state and its Vec<U> is
;; drained downstream before the worker exits.
;;
;; Convergence with prior art — Elixir's Stream.transform/3, Rust's
;; scan-with-emit, Haskell's mapAccumL, Mealy's 1955 sequential-
;; circuit state machine. Arc 006 BACKLOG's resolution named this
;; shape as the substrate every stateful-stage combinator wants.

(:wat::core::define
  (:wat::std::stream::drain-items<U>
    (out :rust::crossbeam_channel::Sender<U>)
    (items :Vec<U>)
    -> :Option<()>)
  ;; Tail-recursive helper: send every item in `items`, stop early
  ;; (returning :None) if the consumer dropped. Returns (Some ()) on
  ;; full drain; returns :None if any send failed, signaling the
  ;; caller to exit.
  (:wat::core::if (:wat::core::empty? items) -> :Option<()>
    (Some ())
    ;; Vec is non-empty (just checked); first returns Some<U> via
    ;; arc 047. The :None arm is unreachable but the type checker
    ;; demands totality.
    (:wat::core::match (:wat::core::first items) -> :Option<()>
      ((Some item)
        (:wat::core::let*
          (((rest-items :Vec<U>) (:wat::core::rest items))
           ((sent :Option<()>) (:wat::kernel::send out item)))
          (:wat::core::match sent -> :Option<()>
            ((Some _)
              (:wat::std::stream::drain-items out rest-items))
            (:None :None))))
      (:None :None))))

(:wat::core::define
  (:wat::std::stream::with-state-worker<T,U,Acc>
    (in :rust::crossbeam_channel::Receiver<T>)
    (out :rust::crossbeam_channel::Sender<U>)
    (step :fn(Acc,T)->(Acc,Vec<U>))
    (flush :fn(Acc)->Vec<U>)
    (acc :Acc)
    -> :())
  (:wat::core::match (:wat::kernel::recv in) -> :()
    ((Some item)
      (:wat::core::let*
        (((stepped :(Acc,Vec<U>)) (step acc item))
         ((new-acc :Acc) (:wat::core::first stepped))
         ((to-emit :Vec<U>) (:wat::core::second stepped))
         ((drain-result :Option<()>)
          (:wat::std::stream::drain-items out to-emit)))
        (:wat::core::match drain-result -> :()
          ((Some _)
            (:wat::std::stream::with-state-worker in out step flush new-acc))
          (:None ()))))
    (:None
      ;; Upstream disconnected. Flush final state; drain whatever it
      ;; produced. Consumer-dropped during flush is swallowed silently
      ;; — same behavior chunks had for its final partial buffer.
      (:wat::core::let*
        (((final-emits :Vec<U>) (flush acc))
         ((_ :Option<()>)
          (:wat::std::stream::drain-items out final-emits)))
        ()))))

(:wat::core::define
  (:wat::std::stream::with-state<T,U,Acc>
    (upstream :wat::std::stream::Stream<T>)
    (init :Acc)
    (step :fn(Acc,T)->(Acc,Vec<U>))
    (flush :fn(Acc)->Vec<U>)
    -> :wat::std::stream::Stream<U>)
  (:wat::core::let*
    (((up-rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first upstream))
     ((pair :(rust::crossbeam_channel::Sender<U>,rust::crossbeam_channel::Receiver<U>))
      (:wat::kernel::make-bounded-queue :U 1))
     ((tx :rust::crossbeam_channel::Sender<U>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<U>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::stream::with-state-worker
        up-rx tx step flush init)))
    (:wat::core::tuple rx handle)))

;; --- chunks (rewritten on top of with-state) ---
;;
;; Surface-reduction proof of the with-state decomposition. The N:1
;; batcher's triple:
;;
;;   init  = empty Vec<T>
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
  (:wat::std::stream::chunks-step<T>
    (buffer :Vec<T>)
    (item :T)
    (size :i64)
    -> :wat::std::stream::ChunkStep<T>)
  (:wat::core::let*
    (((new-buffer :Vec<T>) (:wat::core::conj buffer item)))
    (:wat::core::if (:wat::core::>= (:wat::core::length new-buffer) size)
      -> :wat::std::stream::ChunkStep<T>
      (:wat::core::tuple
        (:wat::core::vec :T)
        (:wat::core::vec :Vec<T> new-buffer))
      (:wat::core::tuple
        new-buffer
        (:wat::core::vec :Vec<T>)))))

(:wat::core::define
  (:wat::std::stream::chunks-flush<T>
    (buffer :Vec<T>)
    -> :Vec<Vec<T>>)
  (:wat::core::if (:wat::core::empty? buffer) -> :Vec<Vec<T>>
    (:wat::core::vec :Vec<T>)
    (:wat::core::vec :Vec<T> buffer)))

(:wat::core::define
  (:wat::std::stream::chunks<T>
    (upstream :wat::std::stream::Stream<T>)
    (size :i64)
    -> :wat::std::stream::Stream<Vec<T>>)
  ;; chunks-step takes (buf, item, size) — three args — but with-state
  ;; wants (buf, item). The `size` parameter has to close over the
  ;; chunks caller's argument, so step is genuinely a lambda capturing
  ;; `size`, not a pass-through. chunks-flush takes (buf) exactly, so
  ;; it passes by name directly (arc 009 — names are values).
  (:wat::std::stream::with-state upstream
    (:wat::core::vec :T)
    (:wat::core::lambda ((buf :Vec<T>) (item :T) -> :wat::std::stream::ChunkStep<T>)
      (:wat::std::stream::chunks-step buf item size))
    :wat::std::stream::chunks-flush))

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
  (:wat::std::stream::chunks-by-step<T,K>
    (state :(Option<K>,Vec<T>))
    (item :T)
    (key-fn :fn(T)->K)
    -> :wat::std::stream::KeyedChunkStep<K,T>)
  (:wat::core::let*
    (((last-key :Option<K>) (:wat::core::first state))
     ((buffer :Vec<T>) (:wat::core::second state))
     ((k :K) (key-fn item)))
    (:wat::core::match last-key -> :wat::std::stream::KeyedChunkStep<K,T>
      (:None
        ;; First item — start the run, emit nothing yet.
        (:wat::core::tuple
          (:wat::core::tuple (Some k) (:wat::core::vec :T item))
          (:wat::core::vec :Vec<T>)))
      ((Some prev)
        (:wat::core::if (:wat::core::= prev k)
          -> :wat::std::stream::KeyedChunkStep<K,T>
          ;; Same key — append to current run, emit nothing.
          (:wat::core::tuple
            (:wat::core::tuple (Some k) (:wat::core::conj buffer item))
            (:wat::core::vec :Vec<T>))
          ;; Key change — emit completed run, start new run.
          (:wat::core::tuple
            (:wat::core::tuple (Some k) (:wat::core::vec :T item))
            (:wat::core::vec :Vec<T> buffer)))))))

(:wat::core::define
  (:wat::std::stream::chunks-by-flush<T,K>
    (state :(Option<K>,Vec<T>))
    -> :Vec<Vec<T>>)
  (:wat::core::let*
    (((buffer :Vec<T>) (:wat::core::second state)))
    (:wat::core::if (:wat::core::empty? buffer) -> :Vec<Vec<T>>
      (:wat::core::vec :Vec<T>)
      (:wat::core::vec :Vec<T> buffer))))

(:wat::core::define
  (:wat::std::stream::chunks-by<T,K>
    (upstream :wat::std::stream::Stream<T>)
    (key-fn :fn(T)->K)
    -> :wat::std::stream::Stream<Vec<T>>)
  ;; init = (None, empty) — no key seen yet, no items buffered.
  ;; step closes over key-fn; flush is size-agnostic so passes by name.
  (:wat::std::stream::with-state upstream
    (:wat::core::tuple :None (:wat::core::vec :T))
    (:wat::core::lambda ((state :(Option<K>,Vec<T>)) (item :T)
                         -> :wat::std::stream::KeyedChunkStep<K,T>)
      (:wat::std::stream::chunks-by-step state item key-fn))
    :wat::std::stream::chunks-by-flush))

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
  (:wat::std::stream::window-step<T>
    (buffer :Vec<T>)
    (item :T)
    (size :i64)
    -> :wat::std::stream::ChunkStep<T>)
  (:wat::core::let*
    (((new-buf :Vec<T>) (:wat::core::conj buffer item))
     ((new-len :i64) (:wat::core::length new-buf)))
    (:wat::core::cond -> :wat::std::stream::ChunkStep<T>
      ;; Over-size — slide: drop first, emit trimmed window, keep trimmed.
      ((:wat::core::> new-len size)
        (:wat::core::let*
          (((trimmed :Vec<T>) (:wat::core::rest new-buf)))
          (:wat::core::tuple trimmed (:wat::core::vec :Vec<T> trimmed))))
      ;; Exactly size — first full window. Emit and keep.
      ((:wat::core::= new-len size)
        (:wat::core::tuple new-buf (:wat::core::vec :Vec<T> new-buf)))
      ;; Under-size — still warming up. No emit.
      (:else
        (:wat::core::tuple new-buf (:wat::core::vec :Vec<T>))))))

(:wat::core::define
  (:wat::std::stream::window-flush<T>
    (buffer :Vec<T>)
    (size :i64)
    -> :Vec<Vec<T>>)
  ;; Flush-partial IFF buffer contains items that were never emitted
  ;; as a full window. That's exactly the case len(buf) < size AND
  ;; len(buf) > 0. The len == size case means a full window was
  ;; already emitted on the sliding path — nothing to flush.
  (:wat::core::cond -> :Vec<Vec<T>>
    ((:wat::core::empty? buffer) (:wat::core::vec :Vec<T>))
    ((:wat::core::< (:wat::core::length buffer) size)
      (:wat::core::vec :Vec<T> buffer))
    (:else (:wat::core::vec :Vec<T>))))

(:wat::core::define
  (:wat::std::stream::window<T>
    (upstream :wat::std::stream::Stream<T>)
    (size :i64)
    -> :wat::std::stream::Stream<Vec<T>>)
  ;; Both step and flush close over size — two lambda wrappers.
  (:wat::std::stream::with-state upstream
    (:wat::core::vec :T)
    (:wat::core::lambda ((buf :Vec<T>) (item :T) -> :wat::std::stream::ChunkStep<T>)
      (:wat::std::stream::window-step buf item size))
    (:wat::core::lambda ((buf :Vec<T>) -> :Vec<Vec<T>>)
      (:wat::std::stream::window-flush buf size))))

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
  (:wat::std::stream::take-worker<T>
    (in :rust::crossbeam_channel::Receiver<T>)
    (out :rust::crossbeam_channel::Sender<T>)
    (remaining :i64)
    -> :())
  (:wat::core::if (:wat::core::<= remaining 0) -> :()
    ()
    (:wat::core::match (:wat::kernel::recv in) -> :()
      ((Some v)
        (:wat::core::let*
          (((sent :Option<()>) (:wat::kernel::send out v)))
          (:wat::core::match sent -> :()
            ((Some _)
              (:wat::std::stream::take-worker in out
                (:wat::core::i64::- remaining 1)))
            (:None ()))))
      (:None ()))))

(:wat::core::define
  (:wat::std::stream::take<T>
    (upstream :wat::std::stream::Stream<T>)
    (n :i64)
    -> :wat::std::stream::Stream<T>)
  (:wat::core::let*
    (((up-rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first upstream))
     ((pair :(rust::crossbeam_channel::Sender<T>,rust::crossbeam_channel::Receiver<T>))
      (:wat::kernel::make-bounded-queue :T 1))
     ((tx :rust::crossbeam_channel::Sender<T>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::stream::take-worker up-rx tx n)))
    (:wat::core::tuple rx handle)))

;; --- flat-map ---
;;
;; 1:N expansion. For each upstream item, apply `f` to get a Vec<U>;
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
  (:wat::std::stream::flat-map-worker<T,U>
    (in :rust::crossbeam_channel::Receiver<T>)
    (out :rust::crossbeam_channel::Sender<U>)
    (f :fn(T)->Vec<U>)
    (pending :Vec<U>)
    -> :())
  (:wat::core::if (:wat::core::empty? pending) -> :()
    (:wat::core::match (:wat::kernel::recv in) -> :()
      ((Some v)
        (:wat::std::stream::flat-map-worker in out f (f v)))
      (:None ()))
    ;; pending is non-empty; first returns Some<U> via arc 047.
    ;; :None arm is unreachable but type-required.
    (:wat::core::match (:wat::core::first pending) -> :()
      ((Some item)
        (:wat::core::let*
          (((rest-items :Vec<U>) (:wat::core::rest pending))
           ((sent :Option<()>) (:wat::kernel::send out item)))
          (:wat::core::match sent -> :()
            ((Some _)
              (:wat::std::stream::flat-map-worker in out f rest-items))
            (:None ()))))
      (:None ()))))

(:wat::core::define
  (:wat::std::stream::flat-map<T,U>
    (upstream :wat::std::stream::Stream<T>)
    (f :fn(T)->Vec<U>)
    -> :wat::std::stream::Stream<U>)
  (:wat::core::let*
    (((up-rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first upstream))
     ((pair :(rust::crossbeam_channel::Sender<U>,rust::crossbeam_channel::Receiver<U>))
      (:wat::kernel::make-bounded-queue :U 1))
     ((tx :rust::crossbeam_channel::Sender<U>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<U>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::stream::flat-map-worker
        up-rx tx f (:wat::core::vec :U))))
    (:wat::core::tuple rx handle)))
