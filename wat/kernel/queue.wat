;; wat/kernel/queue.wat — kernel-namespace channel aliases.
;;
;; Three names, one shape. The runtime exposes channel endpoints as
;; `:rust::crossbeam_channel::Sender<T>` / `Receiver<T>` (the actual
;; Rust types). These aliases give the kernel-namespace short names
;; that every let* binding, function signature, and Vec carrier
;; reaches for — without forcing each caller to spell out the long
;; rust:: paths every time.
;;
;;   QueueSender<T>    — single sender end of a substrate channel
;;   QueueReceiver<T>  — single receiver end of a substrate channel
;;   QueuePair<T>      — what `make-bounded/unbounded-queue` returns
;;   Chosen<T>         — what `:wat::kernel::select` returns
;;                       (idx, wat::core::Result<wat::core::Option<T>, ThreadDiedError>) per arc 111
;;                       — which receiver fired, and what it produced.
;;   CommResult<T>     — what `recv` / `try-recv` return
;;                       (and the inner shape of `send`'s :CommResult<()>)
;;                       wat::core::Result<wat::core::Option<T>, ThreadDiedError> per arc 111:
;;                       Ok(Some v) — value flowed; Ok(:None) — clean
;;                       shutdown (every sender dropped via scope);
;;                       Err(ThreadDied) — sender thread panicked.
;;                       Replaces arc-110-era `:wat::kernel::Sent`.
;;
;; Sister to `:wat::std::stream::Stream<T>` (a tuple alias in
;; stream.wat for `(Receiver<T>, ProgramHandle<()>)`). These three
;; live in the `:wat::kernel::` namespace because they name kernel
;; substrate concepts — bare channel ends and the pair, before any
;; program / producer is wired around them.
;;
;; Registered via the stdlib-types path (src/stdlib.rs +
;; types::register_stdlib_types), which bypasses the reserved-prefix
;; gate that otherwise blocks user code from declaring under :wat::*.

(:wat::core::typealias :wat::kernel::QueueSender<T>
  :rust::crossbeam_channel::Sender<T>)

(:wat::core::typealias :wat::kernel::QueueReceiver<T>
  :rust::crossbeam_channel::Receiver<T>)

(:wat::core::typealias :wat::kernel::QueuePair<T>
  :(wat::kernel::QueueSender<T>,wat::kernel::QueueReceiver<T>))

;; Arc 113 — Err arm widened to a Vec<ThreadDiedError> so cascades
;; carry the chain. Head = the immediate peer that died; tail =
;; whatever killed it, transitively. (:wat::core::first chain)
;; recovers the head when consumers don't care about the trail.
;;
;; The named-chain typealiases below let consumers spell the
;; cascade type without re-typing `Vec<wat::kernel::*DiedError>`
;; at every binding site. `ProcessPanics` is the cross-fork
;; shape (the element type ProcessDiedError matches what
;; fork-program-ast's substrate emits in its
;; `#wat.kernel/ProcessPanics` stderr marker per arc 113 slice 3);
;; `ThreadPanics` is the in-process cousin (chain produced by
;; spawn-thread cascade). Arc 109 § J will introduce a shared
;; supertype `ProgramPanics` satisfied by both — same shape from
;; the caller's vantage; the per-program-kind concrete name is
;; what surfaces today.
(:wat::core::typealias :wat::kernel::ProcessPanics
  :Vec<wat::kernel::ProcessDiedError>)

(:wat::core::typealias :wat::kernel::ThreadPanics
  :Vec<wat::kernel::ThreadDiedError>)

(:wat::core::typealias :wat::kernel::CommResult<T>
  :wat::core::Result<wat::core::Option<T>,wat::kernel::ThreadPanics>)

(:wat::core::typealias :wat::kernel::Chosen<T>
  :(wat::core::i64,wat::kernel::CommResult<T>))
