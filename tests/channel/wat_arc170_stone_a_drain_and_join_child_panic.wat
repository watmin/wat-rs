;; spawn-process child for stone_a T4 — panics intentionally before exit (drain-and-join must still
;; drain stdout+stderr to EOF, then surface the non-zero exit as Err(chain)). A separate subprocess.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::Option/expect :wat::core::None "intentional panic from stone-a process test"))
