;; NEGATIVE CONTROL — a refusal that ALREADY locates correctly. Without this the detector could
;; pass by flagging every diagnostic, and would still be green after a cure that broke user spans.
;; `(length "abc")` dies at runtime with :location naming THIS file.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::length "abc")))
