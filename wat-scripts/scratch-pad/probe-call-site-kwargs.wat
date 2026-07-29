;; probe-call-site-kwargs.wat — does a KWARGS fn's `call-site` see the user, or an adapter?
;;
;; probe-call-site-through-macro.wat falsified both the `let`-pushes-a-frame and the
;; macro-generated-body theories: each reported its own caller's line. The remaining
;; difference in defservice's generated `start` is that it is a KWARGS fn —
;; `[& [locus <- :wat::spawn::Locus  ~@init-param]]` — and kwargs invocation may route
;; through a stdlib adapter, which would push a frame the user never wrote.
;;
;; If the kwargs case reports a wat/*.wat file instead of THIS file's call line, that is the
;; builder's observation reproduced, and it localizes the fix to "kwargs dispatch inserts
;; frames" rather than anything about macros or spawning.

;; positional control — already known-good, repeated here so both run in ONE process
(:wat::core::defn :probe::positional [] -> :wat::kernel::Frame
  (:wat::core::let [origin (:wat::kernel::call-site)] origin))

;; the subject: a kwargs fn, the exact shape defservice's start/resume use
(:wat::core::defn :probe::kw [& [tag <- :wat::core::String]] -> :wat::kernel::Frame
  (:wat::core::let [origin (:wat::kernel::call-site)] origin))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::positional))
    (:wat::kernel::println (:probe::kw :tag "x"))))
