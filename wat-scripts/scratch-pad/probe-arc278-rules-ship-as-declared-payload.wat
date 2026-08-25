;; PROBE — can a rule + the fn its `where` calls cross as a DECLARED payload, with no
;; inference anywhere, and fire on the far side?
;;
;; THE CLAIM UNDER TEST. `probe-arc278-where-body-dep-not-shipped.wat` measured
;; `PC 6 · BASE 5 · SUBJECT 5` — closure extraction (`fn-forms`) does NOT collect a fn
;; referenced only inside a `(:wat::rete::where …)` body. Every fix drawn so far (the
;; expansion-time lift, a macro-phase reflection door, a type-level field accessor, the
;; boundary-declaration stone) makes that INFERENCE smarter.
;;
;; This probe asks whether the inference is needed at all. A rule is FORMS; the forms are
;; retained (`FunctionBody::Wat(ast)`); forms are pure data that "ships and replays"
;; (wat/repl.wat:20-22); and `:wat::eval-with-defs!` already builds a world from a supplied
;; `(Vector :- [WatAST])` — "a wat program can hold an accumulated definition set … and has no way
;; to run anything IN it. This verb closes exactly that gap and nothing else."
;; (src/check.rs:16868-16876). So: DECLARE the payload, do not infer it.
;;
;; ★ MEASURED 2026-08-12 — SUBJECT `EVALUATED derived=1` · CONTROL `CHECK-FAILED`, naming
;; `:usr::big?` as "1 unresolved reference" under "the accumulated definition set no longer
;; freezes on its own". The declared payload CARRIES the where-body dep and the rule fires.
;; The `6/5/5` false negative is not fixed here — it is BYPASSED: nothing walks, so nothing
;; can under-walk. And the failure mode inverts from `fn-forms`' silent under-ship (the child
;; dies at startup naming a symbol nobody shipped) to a STATIC, LOCATED freeze error.
;;
;; ⚠ NON-VACUITY — two arms differing in ONE form:
;;   SUBJECT — payload holds the records, `:usr::big?`, AND the rule. Must EVALUATE, count=1.
;;   CONTROL — the SAME payload with `:usr::big?` OMITTED. Must CHECK-FAIL.
;; If both evaluate, the payload is not what carries the dep and this probe measures nothing.
;; If both check-fail, the harness is broken and the SUBJECT number means nothing either.
;;
;; ⚠ WHAT A GREEN SUBJECT DOES *NOT* PROVE: it says nothing about payload SIZE, about
;; ordering guarantees for a larger defs vector, or about a rule that closes over a runtime
;; VALUE (that is genuinely closure territory and `fn-forms` is right for it). It proves one
;; thing: the where-body dep crosses when the user names it.
;;
;; Everything below lives inside `(:wat::core::forms …)` or `(:wat::core::quote …)`, which are
;; `Boundary::AllData` — the checker does not recurse into them. So `:usr::big?` being absent
;; from the CONTROL payload is NOT a static error in THIS file; it is a runtime outcome on the
;; far side, which is exactly the thing being measured.

;; ⛔ MEASURED 2026-08-12, FIRST RUN — the payload could NOT be built with
;; `(:wat::core::forms …)`. Both `defrule`s inside the forms blocks were MACRO-EXPANDED and
;; then rete-validated against the LOCAL world:
;;
;;   #wat.rete/UnknownFactType  defrule `usr::rule-userfn`: `:usr::Temp` is not a registered
;;                              fact type   (lines 41 and 50, INSIDE the forms blocks)
;;
;; `:usr::Temp` was declared in the SAME payload — the validator simply does not look there.
;; Root, grounded: `src/rete/validate.rs:453` `walk_for_make_rule` is a raw recursive descent
;; over every form and consults NO `Boundary` (the same class as walk.rs's stale `skip(4)`,
;; task #90). And it matches the EXPANDED `make-rule`, which means `forms`' arguments reach
;; the macro expander even though `resolve::boundary` classifies them `AllData`.
;;
;; So `forms` is data to the RESOLVER and code to the EXPANDER. That asymmetry is a real
;; finding and it is NOT what this probe set out to measure — it is filed, and the payload is
;; built with `quote` instead, which `expand_form` DOES treat as an inert data form. That
;; substitution turns the difference into a one-variable differential: quote-inert vs
;; forms-expanded, same declarations.

;; ── SUBJECT payload — the helper is named, so it ships ─────────────────────────────────
(:wat::core::defn :probe::payload-complete [] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::Vector :wat::WatAST
    (:wat::core::quote (:wat::core::defrecord :usr::Temp [c <- :wat::core::i64]))
    (:wat::core::quote (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64]))
    (:wat::core::quote
      (:wat::rete::core::defn :usr::big? [n <- :wat::core::i64] -> :wat::core::bool
        (:wat::rete::core::i64::> n 100)))
    (:wat::core::quote
      (:wat::rete::defrule :usr::rule-userfn
        :when [(:usr::Temp (?c <- :c)) (:wat::rete::where (:usr::big? ?c))]
        :then [(:usr::Hot :c ?c)]))))

;; ── CONTROL payload — byte-identical except `:usr::big?` is NOT named ──────────────────
(:wat::core::defn :probe::payload-missing-helper [] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::Vector :wat::WatAST
    (:wat::core::quote (:wat::core::defrecord :usr::Temp [c <- :wat::core::i64]))
    (:wat::core::quote (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64]))
    (:wat::core::quote
      (:wat::rete::defrule :usr::rule-userfn
        :when [(:usr::Temp (?c <- :c)) (:wat::rete::where (:usr::big? ?c))]
        :then [(:usr::Hot :c ?c)]))))

;; ── the evaluand — reflect the rules OUT of the shipped world, fire, count ─────────────
;; `collect-rules` is the existing namespace reflection (src/rete/collect.rs:50) — the far
;; side never names a rule, it asks the world it was handed. 150 > 100, so a correct run
;; derives exactly one `:usr::Hot`.
(:wat::core::defn :probe::evaluand [] -> :wat::WatAST
  (:wat::core::quote
    (:wat::core::length
      (:wat::rete::query
        (:wat::rete::fire-rules
          (:wat::rete::insert
            (:wat::rete::compile-all
              (:wat::rete::collect-rules :usr)
              (:wat::core::PersistentVector
                (:wat::rete::make-query "usr::Hot"
                  (:wat::core::quote [])
                  (:wat::core::quote [(:usr::Hot)]))))
            (:usr::Temp :c 150)))
        (:wat::rete::make-query "usr::Hot"
          (:wat::core::quote [])
          (:wat::core::quote [(:usr::Hot)]))))))

;; ── run one payload and name which arm fired ──────────────────────────────────────────
(:wat::core::defn :probe::run
  [label <- :wat::core::String
   defs  <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::core::nil
  (:wat::core::match (:wat::eval-with-defs! (:probe::evaluand) defs)

    (:wat::eval::FormOutcome::Declared
      (:wat::kernel::println (:wat::string::concat label " => DECLARED (the evaluand is an expression; this arm means the probe is wrong)")))

    ((:wat::eval::FormOutcome::Evaluated v)
      (:wat::kernel::println
        (:wat::string::concat label " => EVALUATED derived="
          (:wat::core::i64::to-string v))))

    ((:wat::eval::FormOutcome::CheckFailed cause)
      (:wat::core::do
        (:wat::kernel::println (:wat::string::concat label " => CHECK-FAILED (static; nothing ran)"))
        (:wat::kernel::println cause)))

    ((:wat::eval::FormOutcome::Raised cause)
      (:wat::core::do
        (:wat::kernel::println (:wat::string::concat label " => RAISED (dynamic)"))
        (:wat::kernel::println cause)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:probe::run "SUBJECT (helper IN payload)" (:probe::payload-complete))
    (:probe::run "CONTROL (helper OMITTED)   " (:probe::payload-missing-helper))
    (:wat::kernel::println
      "READ: SUBJECT EVALUATED derived=1 AND CONTROL CHECK-FAILED => a declared payload carries the where-body dep, with zero inference. Any other pairing refutes it.")))
