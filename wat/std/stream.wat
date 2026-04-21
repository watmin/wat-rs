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

(:wat::core::use! :rust::crossbeam_channel::Sender)
(:wat::core::use! :rust::crossbeam_channel::Receiver)

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
(:wat::core::define
  (:wat::std::stream::chunks-worker<T>
    (in :rust::crossbeam_channel::Receiver<T>)
    (out :rust::crossbeam_channel::Sender<Vec<T>>)
    (size :i64)
    (buffer :Vec<T>)
    -> :())
  (:wat::core::match (:wat::kernel::recv in) -> :()
    ((Some item)
      (:wat::core::let*
        (((new-buffer :Vec<T>) (:wat::core::conj buffer item)))
        (:wat::core::if (:wat::core::>= (:wat::core::length new-buffer) size) -> :()
          (:wat::core::let*
            (((sent :Option<()>) (:wat::kernel::send out new-buffer)))
            (:wat::core::match sent -> :()
              ((Some _)
                (:wat::std::stream::chunks-worker in out size
                  (:wat::core::vec :T)))
              (:None ())))
          (:wat::std::stream::chunks-worker in out size new-buffer))))
    (:None
      ;; Upstream disconnected. Flush the partial accumulator if
      ;; non-empty; consumer-dropped is swallowed.
      (:wat::core::if (:wat::core::empty? buffer) -> :()
        ()
        (:wat::core::match (:wat::kernel::send out buffer) -> :()
          ((Some _) ())
          (:None ()))))))

(:wat::core::define
  (:wat::std::stream::chunks<T>
    (upstream :wat::std::stream::Stream<T>)
    (size :i64)
    -> :wat::std::stream::Stream<Vec<T>>)
  (:wat::core::let*
    (((up-rx :rust::crossbeam_channel::Receiver<T>) (:wat::core::first upstream))
     ((pair :(rust::crossbeam_channel::Sender<Vec<T>>,rust::crossbeam_channel::Receiver<Vec<T>>))
      (:wat::kernel::make-bounded-queue :Vec<T> 1))
     ((tx :rust::crossbeam_channel::Sender<Vec<T>>) (:wat::core::first pair))
     ((rx :rust::crossbeam_channel::Receiver<Vec<T>>) (:wat::core::second pair))
     ((handle :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :wat::std::stream::chunks-worker
        up-rx tx size (:wat::core::vec :T))))
    (:wat::core::tuple rx handle)))

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
    (:wat::core::let*
      (((item :U) (:wat::core::first pending))
       ((rest-items :Vec<U>) (:wat::core::rest pending))
       ((sent :Option<()>) (:wat::kernel::send out item)))
      (:wat::core::match sent -> :()
        ((Some _)
          (:wat::std::stream::flat-map-worker in out f rest-items))
        (:None ())))))

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
