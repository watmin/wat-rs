;; wat-scripts/sink.wat — pipeline stage 3.
;;
;; Reads `:demo::Partial {:sum :i64}` frames from stdin. Tracks the
;; last seen value across the stream. On EOF, emits one final
;; `:demo::Total {:total :i64}` frame carrying the last partial sum
;; — the pipeline's terminal value.

(:wat::core::defstruct :demo::Partial
  [sum <- :wat::core::i64])

(:wat::core::defstruct :demo::Total
  [total <- :wat::core::i64])


(:wat::core::defn :demo::sink::loop
  [last <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::match (:wat::kernel::readln)
    (:wat::kernel::ReadlnOutcome::Eof last)
    (:wat::kernel::ReadlnOutcome::Stopped last)
    ((:wat::kernel::ReadlnOutcome::Datum partial)
      (:demo::sink::loop (:demo::Partial/sum partial)))))


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:demo::Total :total (:demo::sink::loop 0))))
