;; PROBE — the identity arm, in isolation. Companion to
;; probe-arc278-watast-on-the-wire-decomposed.wat (which measures the two ORIGINAL
;; ★ load-bearing rows). This file targets the two rows the brief added on top:
;;
;;   GATE ROW 3 — a BARE `:wat::WatAST` field (not only `(Vector :- [WatAST])`) crosses.
;;   GATE ROW 4 — THE NEGATIVE ROW: a genuinely wrong field type is still refused.
;;     The identity arm applies to `:wat::WatAST` ALONE — widening the hole instead
;;     of closing the edge is exactly what STOP-3 forbids.
;;
;; `:wat::edn::validate` is called DIRECTLY (no service, no locus) — the walker
;; under test (`edn_shim::edn_to_typed_value`) is exercised the same way whether a
;; caller reaches it via a service's shape-guarded wall or directly, so this is the
;; cheapest fixture that isolates the walker from any locus/transport question.
;;
;; ⛔ SEPARATE FINDING (not this probe's subject, but measured while building it):
;; `probe-arc278-watast-on-the-wire-decomposed.wat`'s "count THREAD" arm now reads
;; `Ok n=3` — the identity arm fixes decode/validate for BOTH loci equally (they
;; share this exact walker). But the PROCESS-locus arms (`count`, and the bare
;; case exercised at the bottom of this file) still read `LOST disconnected`,
;; UNCHANGED from before the fix. Isolated (see report): a process-locus request
;; carrying ANY `:wat::WatAST`-declared field disconnects even when the op handler
;; never touches the field (so it is not a decode-into-the-handler issue) and even
;; though `:wat::edn::validate` on the identical value/type pair (below, and via
;; the THREAD arm) returns `Valid`. A NESTED RECORD field (non-primitive, non-
;; WatAST) round-trips fine over the same locus — so it is not "any non-primitive
;; field," specifically a `:wat::WatAST`-declared field over `:wat::spawn::process`.
;; No panic, no core dump (checked). This predates and survives the identity-arm
;; fix — STOP-4 in the brief names a related process/thread asymmetry as a real,
;; separately-tracked defect and says not to chase it; this is reported, not fixed.

(:wat::core::defrecord :vprobe2::WatAstField [form <- :wat::WatAST])
(:wat::core::defrecord :vprobe2::I64Field [n <- :wat::core::i64])

(:wat::core::defn :vprobe2::render [v <- :wat::edn::Validation] -> :wat::core::String
  (:wat::core::match v
    (:wat::edn::Validation::Valid "VALID")
    ((:wat::edn::Validation::Invalid path expected got)
      (:wat::string::concat "INVALID at "
        (:wat::string::concat (:wat::edn::write path)
          (:wat::string::concat " expected="
            (:wat::string::concat expected
              (:wat::string::concat " got=" got))))))))

;; ── GATE ROW 3 — a bare (not Vector-wrapped) :wat::WatAST field ────────────────
(:wat::core::defn :vprobe2::gate-row-3 [] -> :wat::core::nil
  (:wat::core::let
    [good (:vprobe2::WatAstField :form (:wat::core::quote (:wat::core::defrecord :usr::A [c <- :wat::core::i64])))]
    (:wat::kernel::println
      (:wat::string::concat "GATE-3 bare WatAST field => " (:vprobe2::render (:wat::edn::validate good :vprobe2::WatAstField))))))

;; ── GATE ROW 4 — THE NEGATIVE ROW: a genuinely wrong field must still refuse ───
(:wat::core::defn :vprobe2::gate-row-4 [] -> :wat::core::nil
  (:wat::core::let
    [bad (:wat::edn::read "#vprobe2/I64Field {:n \"not-an-i64\"}")]
    (:wat::kernel::println
      (:wat::string::concat "GATE-4 i64 field handed a String => " (:vprobe2::render (:wat::edn::validate bad :vprobe2::I64Field))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:vprobe2::gate-row-3)
    (:vprobe2::gate-row-4)
    (:wat::kernel::println "READ: GATE-3 must be VALID (the identity refinement can never fail). GATE-4 must be INVALID at [n] expected=:wat::core::i64 got=String (nothing else loosened).")))
