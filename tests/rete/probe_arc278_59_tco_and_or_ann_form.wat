;; tests/rete/probe_arc278_59_tco_and_or_ann_form.wat — co-located fixture for the sibling probe
;; (.rs), slurped via call_beside_value(file!(), entry). Arc 278 #59 — three more tail-context
;; forms (`and`, `or`, `ann-form`) mirroring the `if`/`match`/`let`/`do` shape `eval_tail`
;; already dispatches. Contract: BRIEF-tco-and-or-ann-form.md.

;; ── THE TCO GATE — depth 150000, the measured breaking point (a smaller number proves nothing) ──
;; Same shape as `probe_arc278_55_slice_one_vocabulary.wat`'s `:probe::rete-countdown-if` (row
;; 7/8's TCO gate) — a tail-recursive fn whose LAST form is the form under test. Without
;; `eval_and_tail`/`eval_or_tail`/`eval_ann_form_tail`, each recursive call costs a native Rust
;; stack frame and this SIGSEGVs (SIGABRT under `cargo test`, whose guard page is intact) long
;; before 150000; measured directly against the pinned binary before this arm existed.

(:wat::core::defn :probe::countdown-and [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::core::i64::<= n 0)
    true
    (:wat::core::and true (:probe::countdown-and (:wat::core::i64::- n 1)))))

(:wat::core::defn :user::and-tail-tco-survives-depth [] -> :wat::core::bool
  (:probe::countdown-and 150000))

(:wat::core::defn :probe::countdown-or [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::core::i64::<= n 0)
    false
    (:wat::core::or false (:probe::countdown-or (:wat::core::i64::- n 1)))))

(:wat::core::defn :user::or-tail-tco-survives-depth [] -> :wat::core::bool
  (:probe::countdown-or 150000))

(:wat::core::defn :probe::countdown-ann-form [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::i64::<= n 0)
    0
    (:wat::core::ann-form (:probe::countdown-ann-form (:wat::core::i64::- n 1)) :wat::core::i64)))

(:wat::core::defn :user::ann-form-tail-tco-survives-depth [] -> :wat::core::i64
  (:probe::countdown-ann-form 150000))

;; ── control: `and`/`or` still short-circuit in tail position (STOP-3's gate) ────────────────
;; The identical operand, UNREACHED (first operand already false, so `and` short-circuits before
;; ever evaluating the tail-called second operand), never raises the division.
(:wat::core::defn :user::and-tail-short-circuits [] -> :wat::core::bool
  (:wat::core::and false (:wat::core::i64::> (:wat::core::i64::/ 1 0) 0)))

;; The NON-VACUITY CONTROL: the identical operand, REACHED (first operand true), DOES raise —
;; proving the short-circuit test above isn't passing on a harmless operand.
(:wat::core::defn :user::and-tail-control-raises [] -> :wat::core::bool
  (:wat::core::and true (:wat::core::i64::> (:wat::core::i64::/ 1 0) 0)))

(:wat::core::defn :user::or-tail-short-circuits [] -> :wat::core::bool
  (:wat::core::or true (:wat::core::i64::> (:wat::core::i64::/ 1 0) 0)))

(:wat::core::defn :user::or-tail-control-raises [] -> :wat::core::bool
  (:wat::core::or false (:wat::core::i64::> (:wat::core::i64::/ 1 0) 0)))

;; ── STOP-1 control: TCO must not change any answer ──────────────────────────────────────────
;; A non-tail-recursive `and`/`or`/`ann-form` at a normal, shallow call must still answer exactly
;; as before — TCO is a stack-frame optimization, not a semantic change.
(:wat::core::defn :user::and-tail-shallow-answer [] -> :wat::core::bool
  (:wat::core::and true true false))
(:wat::core::defn :user::or-tail-shallow-answer [] -> :wat::core::bool
  (:wat::core::or false false true))
(:wat::core::defn :user::ann-form-tail-shallow-answer [] -> :wat::core::i64
  (:wat::core::ann-form (:wat::core::i64::+ 2 3) :wat::core::i64))

;; ── the RULED weakening, PINNED (obligation #2 of the #59 brief) ───────────────────────────────
;; `eval_and_tail`/`eval_or_tail` cannot raise the runtime `TypeMismatch` on a non-bool LAST
;; operand — the value is tail-called away before this fn ever inspects it. In ALL statically
;; checked source `infer_boolean_shortcircuit` (check.rs) already forces every operand, including
;; the last, to `:bool`, so this never differs there. The ONLY place the skipped check is
;; observable is a fn body that reaches the runtime WITHOUT the checker ever having seen it: a
;; `:wat::core::fn` literal built INSIDE a `quote` (never type-checked — see `wat_eval_result.wat`
;; test2/test6 for the same "quote never checks its content" property) and invoked via
;; `:wat::eval-ast!` + `:wat::core::apply`. `apply_function` runs `eval_tail` on
;; EVERY fn body on EVERY call — not just self-recursive ones — so this is the ordinary call path,
;; not an exotic one.
;;
;; Without `eval_and_tail`, `eval_tail` would fall through its catch-all to ordinary `eval`,
;; dispatch to the checked `eval_and`, and this would come back `Err` (a located `TypeMismatch`)
;; instead of `Ok(5)`.
(:wat::core::defn :t::and-tail-skips-last-check [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::EvalError])
  (:wat::eval-ast!
    (:wat::core::quote
      (:wat::core::apply (:wat::core::fn [] -> :wat::core::i64 (:wat::core::and true 5)) []))))

(:wat::core::defn :t::or-tail-skips-last-check [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::EvalError])
  (:wat::eval-ast!
    (:wat::core::quote
      (:wat::core::apply (:wat::core::fn [] -> :wat::core::i64 (:wat::core::or false 7)) []))))
