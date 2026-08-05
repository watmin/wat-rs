;; Arc 278 #57 — `:wat::rete::core::enum::{=,not=}`, the ACCEPT path, on a real user `defenum`.
;;
;; The corpus compares a user enum in two `where` forms
;; (`(= (:arena::Route/method ?route) :arena::Method::POST)` — sift-rules-arena.wat:114 and
;; probe-arena-rich-graph.wat:54). The ten minted equality rows cover bool/f64/i64/keyword/string;
;; a user enum is none of them, and never can be — the row table is closed, user enums are not.
;;
;; ── WHY THE ACCEPT PATH IS PROVEN HERE AND NOT IN A RUST UNIT TEST ────────────────────────────
;;
;; A first attempt asserted it in `check.rs`'s test module using `:wat::core::None`. It failed,
;; and the arm named the reason: `:wat::core::Option` is a BUILTIN — it is not registered in
;; `env.types()` as a `TypeDef::Enum` at all (the working idiom, `types.rs:5451`, uses a USER enum
;; `:my::Option`). That test was measuring the harness, not the gate. The subject is a user
;; `defenum`, so the subject is where the proof belongs.
;;
;; The REFUSAL half stays in Rust (`check::tests::rete_enum_equality_refuses_non_enum_operands`)
;; because it needs no enum: two i64 literals are exactly what an ungated `Var("E")`/`Var("E")`
;; row would happily unify, which is the hole this row's `Form` class + gate exist to prevent.
;;
;; ⚠ AND THE NEGATIVE CONTROL CANNOT LIVE IN THIS DIRECTORY: `every_wat_scripts_file_loads`
;; requires every `wat-scripts/` file to type-check, so a deliberately-refused form cannot sit
;; here. Measured instead, and recorded — `(:wat::rete::core::enum::= 1 1)` is refused at
;; `--check` with: "malformed :wat::core::= form: the rete enum-equality surface admits ENUM
;; operands only — got :wat::core::i64 and :wat::core::i64."

(:wat::core::defenum :eq::Method :wat::enum::Pure :GET :POST :PUT :DELETE)
(:wat::core::defenum :eq::Status :wat::enum::Pure :OPEN :CLOSED)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::PersistentMap
      ;; ACCEPT — same user enum, both operands. The row's whole job.
      :same-variant      (:wat::rete::core::enum::= :eq::Method::POST :eq::Method::POST)
      ;; NON-VACUITY: a row hard-wired to `true` passes the line above and fails this one.
      :different-variant (:wat::rete::core::enum::= :eq::Method::GET  :eq::Method::POST)
      ;; `not=` is a distinct row with its own core_name — exercised, not assumed.
      :not-eq-differing  (:wat::rete::core::enum::not= :eq::Method::GET :eq::Method::POST)
      :not-eq-same       (:wat::rete::core::enum::not= :eq::Method::PUT :eq::Method::PUT)
      ;; A SECOND enum proves the gate is about enum-ness, not about one hard-coded type.
      :second-enum       (:wat::rete::core::enum::= :eq::Status::OPEN :eq::Status::OPEN))))
