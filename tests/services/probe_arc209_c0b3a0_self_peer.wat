;; Parent spawns a process echo service; the child gets its self-peer and echoes owner→child + 100.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [self (:wat::program::self-peer :wat::core::i64 :wat::core::i64)
                  x    (:wat::kernel::recv' self)
                  _    (:wat::kernel::send' self (:wat::core::+ x 100))]
                 nil))))
     _   (:wat::kernel::send' svc 5)
     got (:wat::kernel::recv' svc)]
    got))
