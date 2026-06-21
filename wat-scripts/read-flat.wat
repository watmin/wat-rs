;; wat-scripts/read-flat.wat — read ONE value-frame from stdin, write it FLAT.
;;
;; The consumer half of the value-framing end-to-end test: pair with
;; intrinsic-metadata.wat (which pprintln's a multi-line map). This reads the
;; whole multi-line frame as ONE value via readln (value-framed), then println's
;; it compact (one line). Proves a multi-line message crosses an OS pipe intact.
;;
;; Usage:
;;   ./target/release/wat ./wat-scripts/intrinsic-metadata.wat \
;;     | ./target/release/wat ./wat-scripts/read-flat.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::kernel::readln -> :wat::core::Value)))
