;; CONTROL — a hand-listed builtin leaf. TRUE now, must stay TRUE.
;; :rust::crossbeam_channel::Sender is NOT in RustDepsRegistry and cannot be use!'d;
;; its builtin_names row is load-bearing and this stone does not remove it.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :rust::crossbeam_channel::Sender)))
