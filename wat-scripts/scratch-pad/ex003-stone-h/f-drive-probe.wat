;; Excursus 003 stone H — drives the probe fixture's fns directly (no cargo), printing each face.
(:wat::load-file! "/home/john/work/holon/wat-rs/tests/comms/probe_ex003_stone_h_every_wire_encoder_is_strict.wat")
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:h::probe-after-pure))
     _ (:wat::kernel::println (:h::probe-thread-try-send-holon))
     _ (:wat::kernel::println (:h::probe-child-println-pure))
     _ (:wat::kernel::println (:h::probe-child-println-handle))
     _ (:wat::kernel::println (:h::probe-child-pprintln-handle))
     _ (:wat::kernel::println (:h::probe-after-handle))]
    nil))
