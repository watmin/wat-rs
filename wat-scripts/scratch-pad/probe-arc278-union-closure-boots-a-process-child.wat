;; probe-arc278-union-closure-boots-a-process-child.wat — a MEASUREMENT, and it must be RUN.
;;
;; THE QUESTION. `defservice` ships its forked child a HAND-ENUMERATED manifest
;; (`<fqdn>::service-forms`). Its sibling `wat/bracket.wat` ships `fn-forms` closure ++ a
;; one-liner `:user::main`. A name-set diff of the two (2026-08-11) showed the manifest
;; carries four things `fn-forms` over `serve` alone does not reach —
;;   :probe::ffx::init · :probe::ffx::dispatch-admin · :probe::ffx::extract-addr · :user::main
;; — because those are called by the CHILD MAIN, not by serve. So the root set is the main's
;; callees, not `{serve}`.
;;
;; A name-set diff is a CLAIM about what should be enough. This probe is the BREAK that earns
;; it: build the union of `fn-forms` closures over the roots, ship it to a REAL forked child
;; via `:wat::test::spawn-peer` (the capability-holding verb; `spawn-program` has very few
;; allowed callers), and see whether the child BOOTS AND RUNS.
;;
;; ⚠ NON-VACUITY. A green here is worthless on its own: a child that never needed a form
;; cannot tell you the form arrived. So the probe runs TWICE — once with the full union
;; (expect the pass-marker) and once with `init`'s closure OMITTED (expect a death whose
;; cause NAMES the unresolved symbol). If BOTH pass, the instrument is measuring nothing and
;; the run is void — that verdict is printed, not inferred.
;;
;; WHAT THIS DOES *NOT* CLAIM. It ships its own minimal `:user::main`, not defservice's
;; generated one (which needs the full listener/self-peer/ship handshake). So it measures
;; whether a UNION OF CLOSURES IS A COMPLETE, RUNNABLE PROGRAM — not whether the generated
;; service main works. That is the next probe, and it is deliberately not conflated here.

(:wat::core::defsurface :probe::FFX :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::FFX::PingRequest [])
   (:wat::core::defenum :probe::FFX::PingResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::FFX  req <- :probe::FFX::PingRequest] -> :probe::FFX::PingResponse :max-request-bytes 524288)])

;; ★ THE SUBJECT — declared at PROGRAM level, NOT inside the surface's `:messages`. This is
;; the declaration the hand-enumerated manifest drops and the closure carries.
(:wat::core::defenum :probe::FFXTag :wat::enum::Pure
  :Alpha []
  :Beta  [])

(:wat::service::defservice :probe::ffx
  :satisfies :probe::FFX
  :durable   [tag <- :probe::FFXTag]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::ffx::Record] -> :probe::ffx::State
          (:probe::ffx::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::core::let
       [t  (:probe::ffx::Record/tag (:probe::ffx::State/durable s))
        ok (:wat::core::match t
             ((:probe::FFXTag::Alpha) true)
             ((:probe::FFXTag::Beta)  false))]
       (:wat::service::Outcome::Reply s (:probe::FFX::PingResponse::Ok ok))))])

;; ── the DECLARED NAME of a top-level form ────────────────────────────────────────────────
;; Shapes seen in a prologue: (defn :n …) · (def :n …) · (defenum :n …) · (recordtype :n …) ·
;; (structtype :n …) · (defmacro :n …) — name at child 1. A retained type source form arrives
;; `do`-wrapped — (do (recordtype :n …) (defmacro :n …)) — so recurse into its first child.
(:wat::core::defn :user::decl-name [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let
    [ch   (:wat::core::ast->children form)
     head (:wat::core::ast-name (:wat::core::first ch))]
    (:wat::core::if (:wat::core::= head ":wat::core::do")
      (:user::decl-name (:wat::core::first (:wat::core::rest ch)))
      (:wat::core::ast-name (:wat::core::first (:wat::core::rest ch))))))

;; ── the declared names of a forms vector, in order ───────────────────────────────────────
(:wat::core::defn :user::decl-names
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   acc   <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::i64::>= i (:wat::core::length forms))
    acc
    (:user::decl-names forms (:wat::i64::+ i 1)
      (:wat::core::conj acc (:user::decl-name (:wat::core::nth forms i))))))

;; ── the DEDUP KEY of a top-level form: its declaration HEAD *and* its name ───────────────
;; ⚠ A NAME IS NOT A KEY. `decl-name` alone is unsound as a dedup key, and this probe proved
;; it by running: the RAW union carried FOUR forms declaring `:probe::ffx::Record` — two
;; `defmacro` (the kwargs constructor) and two `recordtype` (the type declaration) — and a
;; name-keyed first-wins dedup kept the macro and DISCARDED THE TYPE. The child then held a
;; name with no type behind it, so no accessors were minted and `:probe::ffx::Record/tag`
;; came back unresolved. The type declaration was never missing from `fn-forms`; the
;; instrument ate it.
;;
;; This arc's own census had already said so: `SymbolTable::registrations` reports 182 names
;; in this very world as `[Macro, Type]` — one CONCEPT, two FACETS, registered in different
;; registries at different phases (EXPAND vs CHECK). A name maps to a SET, so a set keyed by
;; name collapses facets that were never duplicates. Key on (head, name).
(:wat::core::defn :user::decl-key [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let
    [ch   (:wat::core::ast->children form)
     head (:wat::core::ast-name (:wat::core::first ch))]
    (:wat::core::if (:wat::core::= head ":wat::core::do")
      (:user::decl-key (:wat::core::first (:wat::core::rest ch)))
      (:wat::string::concat head
        (:wat::string::concat " "
          (:wat::core::ast-name (:wat::core::first (:wat::core::rest ch))))))))

;; ── dedup a forms vector by declaration KEY (first occurrence wins) ──────────────────────
;; Each root's `fn-forms` prologue carries its OWN copy of every shared declaration, so a
;; plain concat of N roots declares the same thing N times — a duplicate-define at child
;; startup. Two forms are duplicates only when they share a head AND a name; a `recordtype`
;; and a `defmacro` of the same name are two facets of one concept and BOTH must ship.
;; Whether this dedup belongs to the extractor or the caller is a DESIGN question this probe
;; surfaces rather than settles; doing it here keeps the measurement about completeness.
(:wat::core::defn :user::dedup-forms
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   seen  <- (:wat::core::Vector :- [:wat::core::String])
   out   <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::i64::>= i (:wat::core::length forms))
    out
    (:wat::core::let
      [form (:wat::core::nth forms i)
       k    (:user::decl-key form)]
      (:wat::core::if (:wat::fix::str-in? k seen)
        (:user::dedup-forms forms (:wat::i64::+ i 1) seen out)
        (:user::dedup-forms forms (:wat::i64::+ i 1)
          (:wat::core::conj seen k)
          (:wat::core::conj out form))))))

;; ── the child's main — MINIMAL, and it exercises the roots on purpose ────────────────────
;; Calls `init` (a root the manifest carries and serve's closure does not reach), constructs
;; the service Record, and matches the PROGRAM-LEVEL enum — the exact form that does not
;; cross the fork today. Ends with the pass-marker `println 0` on fd 1, which the parent
;; recv's (the same contract `run-hermetic'` uses, wat/test.wat:418).
;;
;; ⚠ THE ENTRY ARRIVES UNDER ITS *RENAMED* NAME. `fn-forms` fronts its entry through the
;; inline-lambda path and emits `(def <renamed> <entry-form>)` — so `:probe::ffx::init`'s
;; closure declares `:user::root-init`, and the original name is nowhere in the union. (A
;; SELF-RECURSIVE entry like `serve` also appears under its own name, because its body
;; calls it — which is why serve looked fine and init did not. The asymmetry is
;; recursion, not a dropped form.) A caller composing a union must therefore call the
;; name it ASKED FOR, not the name it started from. MEASURED: calling `:probe::ffx::init`
;; here left exactly one unresolved reference in both arms.
(:wat::core::defn :user::child-main-form [] -> :wat::WatAST
  `(:wat::core::defn :user::main [] -> :wat::core::nil
     (:wat::core::let
       [st (:user::root-init (:probe::ffx::Record :tag (:probe::FFXTag::Alpha)))
        t  (:probe::ffx::Record/tag (:probe::ffx::State/durable st))
        ok (:wat::core::match t
             ((:probe::FFXTag::Alpha) 0)
             ((:probe::FFXTag::Beta)  1))]
       (:wat::kernel::println ok))))

;; ── dump every form declaring `target`, WITH ITS SOURCE, in order ────────────────────────
;; Settles dedup-ate-it vs fn-forms-drops-it: run it over the PRE-DEDUP union. Two entries
;; of differing shape ⇒ the dedup's first-wins is unsound. One bare `recordtype` (no
;; `do`-wrapped ctor macro beside it) ⇒ the extractor never emitted the constructor.
(:wat::core::defn :user::dump-named
  [forms  <- (:wat::core::Vector :- [:wat::WatAST])
   i      <- :wat::core::i64
   target <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length forms))
    nil
    (:wat::core::if (:wat::core::= (:user::decl-name (:wat::core::nth forms i)) target)
      (:wat::core::do
        (:wat::kernel::println
          (:wat::string::concat "  ["
            (:wat::string::concat (:wat::i64::to-string i)
              (:wat::string::concat "] "
                (:wat::core::ast->source (:wat::core::nth forms i))))))
        (:user::dump-named forms (:wat::i64::+ i 1) target))
      (:user::dump-named forms (:wat::i64::+ i 1) target))))

;; ── the union of closures over the child main's callees ──────────────────────────────────
;; `with-init?` false is the NEGATIVE CONTROL: omit init's closure and the child must die
;; naming it. Everything else is identical, so the control differs in exactly one form-set.
;; RAW: the plain concat, BEFORE dedup — the honest input to the dedup question.
(:wat::core::defn :user::raw-union [with-init? <- :wat::core::bool]
  -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::let
    [serve-forms (:wat::kernel::fn-forms :probe::ffx::serve :user::root-serve)
     init-forms  (:wat::core::if with-init?
                   (:wat::kernel::fn-forms :probe::ffx::init :user::root-init)
                   (:wat::core::Vector :- [:wat::WatAST]))
     joined      (:wat::core::concat serve-forms init-forms)]
    (:wat::core::conj joined (:user::child-main-form))))

(:wat::core::defn :user::union-forms [with-init? <- :wat::core::bool]
  -> (:wat::core::Vector :- [:wat::WatAST])
  (:user::dedup-forms (:user::raw-union with-init?) 0
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::Vector :- [:wat::WatAST])))

;; THE SETTLING MEASUREMENT — every `:probe::ffx::Record` form, pre-dedup then post-dedup.
(:wat::core::defn :user::settle-record-ctor [] -> :wat::core::nil
  (:wat::core::let
    [raw   (:user::raw-union true)
     dedup (:user::union-forms true)
     _a    (:wat::kernel::println
             (:wat::string::concat "RAW union size="
               (:wat::i64::to-string (:wat::core::length raw))))
     _b    (:wat::kernel::println "RAW forms declaring :probe::ffx::Record —")
     _c    (:user::dump-named raw 0 ":probe::ffx::Record")
     _d    (:wat::kernel::println
             (:wat::string::concat "DEDUPED union size="
               (:wat::i64::to-string (:wat::core::length dedup))))
     _e    (:wat::kernel::println "DEDUPED forms declaring :probe::ffx::Record —")]
    (:user::dump-named dedup 0 ":probe::ffx::Record")))

;; ── run one arm: ship the forms to a real forked child, report what came back ────────────
(:wat::core::defn :user::run-arm
  [label <- :wat::core::String  with-init? <- :wat::core::bool]
  -> :wat::core::bool
  (:wat::core::let
    [forms (:user::union-forms with-init?)
     _n    (:wat::kernel::println
             (:wat::string::concat label
               (:wat::string::concat " forms=" (:wat::i64::to-string (:wat::core::length forms)))))
     ;; What did the union actually DECLARE? The child's "unresolved reference" names a
     ;; symbol; this names what was shipped. Without both, the gap is a guess.
     _dl   (:wat::kernel::println (:wat::string::concat label " declares:"))
     _d    (:wat::kernel::println (:user::decl-names forms 0 (:wat::core::Vector :- [:wat::core::String])))
     p     (:wat::test::spawn-peer (:wat::spawn::process) forms)]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::core::do (:wat::kernel::println (:wat::string::concat label " BOOTED-AND-RAN")) true))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat label " DIED "
            (:wat::kernel::LociDiedError/message cause)))
          false))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::core::do (:wat::kernel::println (:wat::string::concat label " STOPPED")) false))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::core::do (:wat::kernel::println (:wat::string::concat label " CLOSED-NO-MARKER")) false)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_settle (:user::settle-record-ctor)
     full    (:user::run-arm "FULL   " true)
     control (:user::run-arm "CONTROL" false)]
    ;; The verdict is PRINTED, never inferred: the control passing means the instrument is
    ;; vacuous and the full arm's green proves nothing.
    (:wat::core::if control
      (:wat::kernel::println "VERDICT VACUOUS — the control ran without init's closure; this probe measures nothing")
      (:wat::core::if full
        (:wat::kernel::println "VERDICT MEANINGFUL — full union boots and runs; control dies without init")
        (:wat::kernel::println "VERDICT INCOMPLETE — the full union does NOT boot; read its DIED cause above")))))
