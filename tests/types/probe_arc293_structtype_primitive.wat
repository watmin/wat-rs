;; tests/types/probe_arc293_structtype_primitive.wat — co-located fixture
;;
;; Arc 293.2-parity — the :wat::core::structtype primitive.
;; RED at HEAD: :wat::core::structtype is an unknown declaration head.

(:wat::core::structtype :my::Point
  [x <- :wat::core::i64  y <- :wat::core::i64])
