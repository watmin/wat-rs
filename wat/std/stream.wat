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

;; --- spawn-producer ---
;;
;; Accepts a producer function of signature
;; `:fn(Sender<T>) -> :()` — it writes values to the sender until
;; done, then returns. Spawn wires the bounded(1) queue and returns
;; the Stream<T> to the caller. Caller consumes via a combinator or
;; a direct recv loop.
(:wat::core::define
  (:wat::std::stream::spawn-producer<T>
    (producer :fn(rust::crossbeam_channel::Sender<T>)->())
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
