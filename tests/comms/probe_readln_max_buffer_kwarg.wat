;; tests/comms/probe_readln_max_buffer_kwarg.wat — co-located fixture slurped via startup_beside(file!()).
;; readln :max-buffer-bytes escape hatch — both the kwarg form and plain form must type-check.

;; Test form 1: the kwarg escape hatch (2 MiB cap). Must type-check — readln is a macro over readln'.
(:wat::core::defn :user::readln-with-max-buffer [] -> :wat::core::nil
  (:wat::core::let
    [_line (:wat::core::match (:wat::kernel::readln :max-buffer-bytes (:wat::i64::* 2 (:wat::i64::* 1024 1024)) ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    nil))

;; Test form 2: backward compat — the existing no-kwarg form must keep working.
(:wat::core::defn :user::readln-plain [] -> :wat::core::nil
  (:wat::core::let
    [_line (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    nil))

