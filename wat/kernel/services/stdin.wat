;; wat/kernel/services/stdin.wat — the `readln` macro + its default cap.
;;
;; Arc 170 Phase 3: the hand-rolled StdInService (the `:wat::kernel::ThreadId` typealias, the
;; `StdInService::Req`/`Rep` structs, and the `StdInService/handle` fn driven by the Rust
;; `spawn_service_peer` loop) is DELETED. stdin is now the primed `:wat::kernel::stdin-svc` defservice
;; (wat/kernel/services/stdio-primes.wat), reached by the flipped `readln'` verb. All that remains here
;; is the user-facing `readln` macro + the default frame cap it injects — both load-order-free.

;; ─── Default frame cap ────────────────────────────────────────────────────
;;
;; Arc 255 escape-hatch — single source of truth for the default readln cap.
;; The `readln` macro injects this as the cap arg when no :max-buffer-bytes
;; kwarg is supplied; `readln'` always takes an explicit max (no Rust default).
;; 512 × 1024 = 524 288 bytes (512 KiB) — mirrors DEFAULT_MAX_FRAME_BYTES in
;; src/edn_shim.rs (kept for the Receiver/from-pipe channel path which has
;; no macro layer).
(:wat::core::def :wat::kernel::MAX-READLN-BYTES
  (:wat::core::i64::* 512 1024))

;; ─── readln macro ─────────────────────────────────────────────────────────
;;
;; Arc 255 escape-hatch. `readln` is the user-facing defmacro; `readln'`
;; (the prime) is the kernel-restricted positional primitive they expand to.
;;
;; Per the kwargs-is-always-a-macro doctrine: the exposed surface is kwargs
;; (readln :max-buffer-bytes N -> :T), the lean prime is positional.
;;
;; Shape:
;;   (readln -> :T)                        → (readln' :wat::kernel::MAX-READLN-BYTES -> :T)
;;   (readln :max-buffer-bytes N -> :T)    → (readln' N -> :T)
;;
;; The `-> :T` annotation is forwarded intact so the checker can infer
;; readln's polymorphic return type from the call-site arrow (see
;; infer_kernel_readln_prime in src/check.rs).
;;
;; Arg parse: if the first element of `args` is the `:max-buffer-bytes`
;; keyword (checked via ast-kind + ast-name), consume it + the next element
;; (N) and emit `(readln' N <rest>)`; otherwise emit `(readln' <args>)`.
;;
;; The program-body path (no leading quasiquote) runs in the fenced macro
;; evaluator; `args` is bound as a Value::Vec of Value::wat__WatAST nodes.
;; `get` returns Option<Value::wat__WatAST>; `Option/expect` unwraps it.
;;
;; `readln'` is a Rust intrinsic (always available at expand time — no
;; load-order dependency on any wat file). This macro therefore has no
;; load-order constraint and lives in stdin.wat as the natural home.
(:wat::core::defmacro :wat::kernel::readln
  [& args <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::let
    [n-args    (:wat::core::length args)
     ;; Check whether the first form is the :max-buffer-bytes keyword.
     ;; Use get (safe on empty vector) and compare by ast-kind + ast-name.
     first-opt (:wat::core::get args 0)]
    (:wat::core::if
      ;; Is there a first arg AND is it a keyword?
      (:wat::core::if
        (:wat::core::= n-args 0)

        false
        (:wat::core::= (:wat::core::ast-kind
                         (:wat::core::Option/expect
                           first-opt
                           "readln macro: internal error — first-opt is None but n-args > 0"))
                       "keyword"))

      ;; First arg is a keyword. Check if it's :max-buffer-bytes.
      (:wat::core::let
        [first-node (:wat::core::Option/expect
                       first-opt
                       "readln macro: internal error — first-node")]
        (:wat::core::if
          (:wat::core::= (:wat::core::ast-name first-node) ":max-buffer-bytes")

          ;; :max-buffer-bytes N  →  (readln' N)  (arc 258 — no `-> :T`; the
          ;; self-describing EDN wire types the value, not the caller)
          (:wat::core::let
            [cap-expr (:wat::core::Option/expect
                          (:wat::core::get args 1)
                          "readln: :max-buffer-bytes requires a value (e.g. :max-buffer-bytes (* 2 1024 1024))")
             rest     (:wat::core::rest (:wat::core::rest args))]
            `(:wat::kernel::readln' ~cap-expr ~@rest))
          ;; Unknown keyword as first arg — pass through to readln' for a clean error.
          `(:wat::kernel::readln' ~@args)))
      ;; First arg is not a keyword (or args is empty) — plain form:
      ;; (readln) → (readln' :wat::kernel::MAX-READLN-BYTES)  (arc 258 — no `-> :T`;
      ;; readln reads what the self-describing EDN wire says, the caller does not attest).
      ;; The macro injects the default cap so readln' always gets an explicit max.
      `(:wat::kernel::readln' :wat::kernel::MAX-READLN-BYTES ~@args))))
