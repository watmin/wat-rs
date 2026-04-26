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
;;                       (idx, Option<T>) — which receiver fired,
;;                       and what it produced (Some v) or disconnected (:None)
;;   Sent              — what `:wat::kernel::send` returns
;;                       Some(()) on placed; None on disconnect.
;;                       Mirrors the recv side's :Option<T> shape.
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

(:wat::core::typealias :wat::kernel::Chosen<T>
  :(i64,Option<T>))

(:wat::core::typealias :wat::kernel::Sent
  :Option<()>)
