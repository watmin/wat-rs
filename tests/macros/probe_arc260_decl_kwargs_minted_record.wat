;; tests/macros/probe_arc260_decl_kwargs_minted_record.wat — co-located fixture for
;; probe_arc260_decl_kwargs_minted_record.rs, slurped via startup_beside(file!()).
;;
;; & {port tls} mints :user::connect::Kwargs; the body uses port + tls by name (destructured);
;; the call constructs the record explicitly (no sugar) and passes it. 443 + (tls?1:0) = 444.
(:wat::core::defn :user::connect
  [host <- :wat::core::String
   & [port <- :wat::core::i64  tls <- :wat::core::bool]]
  -> :wat::core::i64
  (:wat::i64::+ port (:wat::core::if tls  1 0)))

;; Arc 294 item 9a: the bundle is built with KWARGS (the bare name is the kwargs macro;
;; the positional prime is generated-code-only). "Constructs the record explicitly (no
;; sugar)" is about passing a PRE-BUILT record rather than using the call sugar — how the
;; record itself is built is orthogonal.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:user::connect "example.com" (:user::connect::Kwargs :port 443 :tls true)))

