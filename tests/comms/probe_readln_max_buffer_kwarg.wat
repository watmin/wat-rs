;; tests/comms/probe_readln_max_buffer_kwarg.wat — co-located fixture slurped via startup_beside(file!()).
;; readln :max-buffer-bytes escape hatch — both the kwarg form and plain form must type-check.

;; Test form 1: the kwarg escape hatch (2 MiB cap). Must type-check — readln is a macro over readln'.
(:wat::core::defn :user::readln-with-max-buffer [] -> :wat::core::nil
  (:wat::core::let
    [_line (:wat::kernel::readln :max-buffer-bytes (:wat::core::i64::* 2 (:wat::core::i64::* 1024 1024)) -> :wat::core::String)]
    nil))

;; Test form 2: backward compat — the existing no-kwarg form must keep working.
(:wat::core::defn :user::readln-plain [] -> :wat::core::nil
  (:wat::core::let
    [_line (:wat::kernel::readln -> :wat::core::String)]
    nil))

