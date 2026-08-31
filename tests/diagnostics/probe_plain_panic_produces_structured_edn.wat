;; tests/diagnostics/probe_plain_panic_produces_structured_edn.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
;; (fork + OS-pipe scrape → :wat::kernel::RunResult) onto the PRIMED peer wire — a
;; direct `(:wat::test::spawn-peer (:wat::spawn::process) (:wat::core::forms …))`
;; child + `(:wat::kernel::recv' p)`. `RunResult` is GONE from this file.
;;
;; ★ SUBJECT (do not confuse with the vehicle below): a bare Rust panic (`panic!()`, NOT an
;; `AssertionPayload`) that happens in a forked child crosses the PRIMED wire as a STRUCTURED
;; `LociDiedError::Panic`, with the panic's own String riding `Panic.message` verbatim — never
;; degraded to the exit-code-only fallback "forked program exited N". This is the ONLY thing this
;; probe exists to prove; the panic's SOURCE is incidental and has already been swapped once
;; (BRIEF-construction-inside-a-fn.md) without touching this claim.
;;
;; ★ VEHICLE HISTORY — why the old one died, why THIS one cannot die the same way:
;; The original vehicle set `dim_count=1` (budget=`floor(sqrt(1))=1`) and Bundled a 2-ELEMENT
;; LITERAL vector — cost=2 > budget=1 → `panic!` inside `eval_algebra_bundle`. That vehicle died
;; when `freeze::validate_holon_record_capacity` (BRIEF-construction-inside-a-fn.md, gap (b))
;; started checking every registered `HolonRecord`'s OWN declared field count against the SAME
;; budget at STARTUP: at `dim_count=1`, budget=1 is so small that the STDLIB'S OWN built-in
;; HolonRecord types (e.g. `:wat::telemetry::Scope`, 4 fields) now fail to even START UP, so the
;; child never reaches `:user::main` at all — the vehicle's failure mode moved from "runtime
;; panic" to "the freeze-time check I closed", which is a DIFFERENT, now-INTENTIONAL death, not
;; the bare-panic one this probe is supposed to exercise.
;;
;; The fix is not a bigger literal — a literal vector's element COUNT is exactly the kind of
;; static AST shape a future checker/freeze pass COULD learn to count (the same class of gap gap
;; (a)/(b) just closed). So THIS vehicle builds the vector at RUNTIME via `foldl` over `range 0 n`
;; — its length is the result of actually EXECUTING a fold, not a literal AST shape, so no static
;; analysis (checker OR freeze) can ever bound it ahead of time; it can only be discovered by
;; running the program, which is exactly the "genuine bare Rust panic with no freeze-time
;; analogue" this probe now needs. `dim_count=100` → budget=`floor(sqrt(100))=10`, comfortably
;; above every stdlib HolonRecord's own field count (verified: `dim_count=100` alone, no Bundle
;; call at all, starts up clean) — so the STDLIB never trips the freeze-time check; only the
;; test's OWN 12-atom runtime-built vector (12 > 10) exceeds capacity, in `:user::main`'s body,
;; exactly where the panic needs to fire.
;;
;; The child runs in a SEPARATE process with its own runtime, so `set-dim-count!` /
;; `set-capacity-mode!` are private to the child — the hermetic property the retired
;; run-hermetic provided is preserved by spawn-program' :process.
(:wat::core::defn :probe::plain-panic [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           ;; Config setters are load-time DIRECTIVES collected by the child's
           ;; entry-file pass — they MUST sit at the top level of the forms,
           ;; preceding all other forms (they are not runtime functions callable
           ;; from inside :user::main's body — that surfaces UnknownFunction).
           (:wat::config::set-dim-count! 100)
           (:wat::config::set-capacity-mode! :panic)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             ;; A RUNTIME-BUILT (not literal) 12-atom vector exceeds floor(sqrt(100))=10 budget
             ;; → panic!("capacity exceeded under :panic") fires inside eval_algebra_bundle.
             ;; See this file's header for why the length must be runtime-derived, not a literal.
             (:wat::core::let
               [n     12
                atoms (:wat::core::foldl
                        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::holon::HolonAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::holon::HolonAST])
                          (:wat::core::conj acc (:wat::holon::to-holon i)))
                        (:wat::core::Vector :- [:wat::holon::HolonAST])
                        (:wat::core::range 0 n))
                _bundle (:wat::holon::Bundle atoms)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ;; TRUE variant: a bare Rust panic (capacity exceeded, NOT an AssertionPayload)
          ;; → LociDiedError::Panic; the panic String rides Panic.message. Return it.
          ((:wat::kernel::LociDiedError::Panic message _failure) message)
          ;; LociDiedError is the no-hidden-failures enum — every OTHER death is named
          ;; EXPLICITLY (no `_` lump; verbosity is the shield). A distinct WRONG:<variant>
          ;; sentinel makes a RED name exactly which non-Panic death surfaced instead.
          ((:wat::kernel::LociDiedError::RuntimeError _m) "WRONG:RuntimeError")
          (:wat::kernel::LociDiedError::Disconnected "WRONG:Disconnected")
          (:wat::kernel::LociDiedError::Stopped "WRONG:Stopped")
          (:wat::kernel::LociDiedError::Severed "WRONG:Severed")
          ((:wat::kernel::LociDiedError::StartupError _m) "WRONG:StartupError")
          ((:wat::kernel::LociDiedError::EntryFormFailure _m) "WRONG:EntryFormFailure")
          ((:wat::kernel::LociDiedError::MainSignature _m) "WRONG:MainSignature")
          ((:wat::kernel::LociDiedError::BadReturn _m) "WRONG:BadReturn")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED"))))
