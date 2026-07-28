;; wat-scripts/read-flat.wat — read ONE value-frame from stdin, write it FLAT.
;;
;; The consumer half of the value-framing end-to-end test: pair with
;; intrinsic-metadata.wat (which pprintln's a multi-line map). This reads the
;; whole multi-line frame as ONE value via readln (value-framed), then println's
;; it compact (one line). Proves a multi-line message crosses an OS pipe intact.
;;
;; Usage:
;;   cargo wat ./wat-scripts/intrinsic-metadata.wat \
;;     | cargo wat ./wat-scripts/read-flat.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
