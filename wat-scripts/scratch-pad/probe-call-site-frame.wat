;; probe-call-site-frame.wat — WHICH frame does `(:wat::kernel::call-site)` return?
;;
;; The #6 label lift rests entirely on this: `map-worker` / a generated `start` must be
;; able to name THEIR CALLER's source position from inside their own body, with no param
;; threaded and no macro plumbing. runtime.rs:20755 says `snapshot_call_stack().first()`
;; is "the innermost user call that invoked call-site" and FrameInfo is
;; `{callee_path, call_span}` — so the expectation is:
;;
;;   file/line = WHERE THE CALL TO `spawner` WAS WRITTEN (this file, the `(:probe::spawner)`
;;               line in main), NOT where call-site itself is written (inside spawner).
;;
;; If that is inverted — if it reports spawner's OWN line — then the lift needs the caller
;; to pass its position in, and the "no plumbing" claim is false. Measure, do not assume.

(:wat::core::defn :probe::spawner [] -> :wat::kernel::Frame
  (:wat::kernel::call-site))

;; A second caller at a DIFFERENT line: the two must disagree, or call-site is reporting
;; the callee's fixed position and is useless for labelling.
(:wat::core::defn :probe::other-caller [] -> :wat::kernel::Frame
  (:probe::spawner))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::spawner))
    (:wat::kernel::println (:probe::other-caller))))
