;; tests/types/probe_arc293_k4_extend_type_own_aggregate.wat — co-located fixture (arc 293 K4)
;;
;; LOCK-IN regression: `extend-type` is the GENERAL per-type satisfaction door — it binds method
;; impls for ANY type, your OWN aggregates included, not just foreign builtins. The "foreign-only
;; adapter / monkeypatch" framing was DOCTRINAL; 293.4c built the registration generically (it
;; registers `:T/method` for any T, never gated to foreign), and 293.4e-pre.i gave it the one
;; canonical ArgSpec. This guards the capability K5 structurally depends on: `extend-surface`
;; expands to `(extend-type S$record S …)`, and `S$record` is an OWN aggregate (K2/K3-emitted).
;;
;; GREEN at HEAD (no RED phase — proves + guards an existing capability).

(:wat::core::defrecord :k4::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defsurface :k4::Located :nature :wat::core::Record
  :features [(mag2 [self <- :k4::Located] -> :wat::core::i64)])   ; a METHOD feature (behavior)

;; extend-type on :k4::Pt — a type I OWN — to satisfy :k4::Located (impl reads Pt's own fields):
(:wat::core::extend-type :k4::Pt :k4::Located
  (mag2 [self] -> :wat::core::i64
    (:wat::i64::+
      (:wat::i64::* (:k4::Pt/x self) (:k4::Pt/x self))
      (:wat::i64::* (:k4::Pt/y self) (:k4::Pt/y self)))))

;; consume through the surface — dispatch routes :k4::Located/mag2 → :k4::Pt/mag2 by runtime type:
(:wat::core::defn :k4::report [s <- :k4::Located] -> :wat::core::i64
  (:k4::Located/mag2 s))

(:wat::core::defn :k4::demo [] -> :wat::core::i64
  (:k4::report (:k4::Pt :x 3 :y 4)))    ; 3*3 + 4*4 = 9 + 16 = 25
