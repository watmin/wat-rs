;; wat/bracket.wat — the brackets layer (Ruby's Parallel) built over spawn-program.
;;
;; This stone ships the runner server-loop — the multi-message peer that a
;; brackets pool stands on.  The pool coordinator + `brackets/map` come next.
;;
;; ── Design ───────────────────────────────────────────────────────────────────
;;
;; Today's spawn-program peers are single-shot: recv once → send once → return.
;; The brackets pool needs a peer that STREAMS: recv' item → work-fn → send'
;; result, looping until its channel drains.  The loop is a NAMED tail-recursive
;; defn so wat's TCO (arc 003 — apply_function replaces the top frame in place)
;; keeps the stack constant at any item count.
;;
;; Exit discipline: recv' raises (EvalBreak) when the parent's Thread is
;; dropped → the runner's recursion is broken by the raise → it exits cleanly.
;; No explicit termination condition is needed; the channel drain IS the signal.
;;
;; Loads AFTER wat/spawn.wat (uses :wat::kernel::Peer, recv', send').
;;
;; ── Rendezvous convention ───────────────────────────────────────────────────
;;
;; `:user::` is the RENDEZVOUS NAMESPACE — the known-location coordinates where
;; a program exposes what a substrate consumer looks up.  Not private/internal
;; space; a rendezvous space.  `:user::main` is wat-program's coordinate (the
;; kernel-required entry, `[] -> :nil`).  Bracket installs a second one:
;; `:user::bracket::work-fn` — the work function a process-pool child's
;; baked runner (`(:wat::bracket::process-runner :- [I O])` below) applies.  The
;; runner itself is baked/reserved (never shipped); the child's user-data
;; only ever ships the user's own work-fn, reified at this coordinate, plus a
;; generated `:user::main` that passes the coordinate's value into the runner.

(:wat::core::defn :wat::bracket::runner-loop :- [I O]
  [self    <- (:wat::kernel::ThreadSelfPeer :- [O I])
   work-fn <- [I :-> O]]
  -> :wat::core::nil
  ;; arc 278 the recv'-outcome wall — recv' returns a matchable (RecvOutcome :- [I]).
  ;; ::Message → work + recurse; ::Lost (parent Thread crashed) → eprintln the cause
  ;; (loud, terminal); ::Closed (parent dropped cleanly) → exit the runner loop.
  (:wat::core::match (:wat::kernel::recv self)  
    ((:wat::kernel::RecvOutcome::Message item)
      ;; arc 278 the send'-outcome wall — face all three arms explicitly. A dead parent
      ;; here means the NEXT recv' observes Closed/Lost and exits the loop honestly, so
      ;; every arm proceeds to recurse (never a `_`-swallow).
      (:wat::core::match (:wat::kernel::send self (work-fn item))
        (:wat::kernel::SendOutcome::Sent   (:wat::bracket::runner-loop self work-fn))
        (:wat::kernel::SendOutcome::Stopped nil)                                        ;; arc 278 #73 — the WORLD is stopping → exit the runner loop
        (:wat::kernel::SendOutcome::Closed (:wat::bracket::runner-loop self work-fn))   ;; parent gone → next recv' faces it
        ((:wat::kernel::SendOutcome::Lost _c) (:wat::bracket::runner-loop self work-fn))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    ;; arc 278 #73 — exit like Closed, DIFFERENT reason: the parent did not drop,
    ;; the substrate is stopping. Same body, stated cause (never an unexplained twin).
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

;; (PoolMsg :- [D I]) (the universal pool wire message) is defined in wat/spawn.wat — it
;; must precede the :wat::spawn::Locus surface's `spawn-runner` return type, which
;; loads before this file. See the defenum + rationale there.

;; ── process-runner — the BAKED, reserved process-pool runner (259 S3c) ───────
;;
;; Generic index-wrapping runner for the process (not-shared) locus tier: recv
;; (idx,I) → work-fn item → send (idx,O), tail-recursing forever.  Established
;; in the child's phase-one stdlib load — privileged, reserved, zero user
;; input.  A user can never allocate it (`:wat::` is undefinable anywhere) and
;; it is never shipped, so nothing can collide with it.  The work-fn is taken
;; as a VALUE (not referenced by name) so the runner stays generic/baked with
;; no stdlib -> user-data forward reference; the process arm's spawn-runner
;; ships only the work-fn (at the :user::bracket::work-fn rendezvous
;; coordinate) and a generated :user::main that passes it in here.
(:wat::core::defn :wat::bracket::process-runner :- [D I O]
  [self    <- (:wat::kernel::Peer :- [(:wat::core::Tuple :- [:wat::core::i64 O]) (:wat::bracket::PoolMsg :- [D I])])
   work-fn <- [I :-> O]]
  -> :wat::core::nil
  ;; arc 278 the recv'-outcome wall — recv' returns (RecvOutcome :- [PoolMsg]). ::Message →
  ;; dispatch the PoolMsg; ::Lost (parent crashed) → eprintln (loud, terminal); ::Closed
  ;; (parent dropped) → exit the runner.
  (:wat::core::match (:wat::kernel::recv self)  
    ((:wat::kernel::RecvOutcome::Message m)
      (:wat::core::match m  
        ((:wat::bracket::PoolMsg::Work pair)
          (:wat::core::let
            [out (:wat::core::Tuple (:wat::core::first pair) (work-fn (:wat::core::second pair)))]
            ;; arc 278 the send'-outcome wall — face all three arms; a dead parent surfaces
            ;; via the next recv', so every arm proceeds to recurse.
            (:wat::core::match (:wat::kernel::send self out)
              (:wat::kernel::SendOutcome::Sent   (:wat::bracket::process-runner self work-fn))
              (:wat::kernel::SendOutcome::Stopped nil)                                           ;; arc 278 #73 — the WORLD is stopping → exit
              (:wat::kernel::SendOutcome::Closed (:wat::bracket::process-runner self work-fn))   ;; parent gone → next recv' faces it
              ((:wat::kernel::SendOutcome::Lost _c) (:wat::bracket::process-runner self work-fn)))))
        ;; A non-dialing pool never sends :Setup (dials empty); the arm is total by
        ;; construction — ignore + recurse (D stays phantom for this runner).
        ((:wat::bracket::PoolMsg::Setup _deps)
          (:wat::bracket::process-runner self work-fn))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    ;; arc 278 #73 — exit like Closed, DIFFERENT reason: the parent did not drop,
    ;; the substrate is stopping. Same body, stated cause (never an unexplained twin).
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

;; ── process-dial-runner — the BAKED dialing process-pool runner (arc 170 M1) ──
;;
;; The 2-param cousin of process-runner: a granted worker that DIALS a granted
;; service and HOLDS the typed peer across items. Generic over :- [S R I O]: the
;; dialed service's channel is S/R, so the held context is `(Peer :- [S R])` and the
;; Setup payload is `(Address :- [S R])` (connect' : (Address :- [S R]) → (Peer :- [S R])). Threads
;; `ctx` = `(Option (Peer :- [S R]))`, None until the first Setup:
;;   :Setup(deps) → dial-and-HOLD: recurse with `(Some (connect' deps))`. The pid was
;;                  already granted (grant-boot, map-worker) so connect' is admitted.
;;   :Work(pair)  → run the held peer through the 2-param work-fn, send the indexed out.
;; This is the defservice :init/:ephemeral pattern lifted onto the bracket, and the
;; exact shape wat-scripts/probes/arc-170/probe-m1-worker-setup.wat proved GREEN.
(:wat::core::defn :wat::bracket::process-dial-runner :- [S R I O]
  [self    <- (:wat::kernel::Peer :- [(:wat::core::Tuple :- [:wat::core::i64 O]) (:wat::bracket::PoolMsg :- [(:wat::kernel::Address :- [S R]) I])])
   work-fn <- [(:wat::kernel::Peer :- [S R]) I :-> O]
   ctx     <- (:wat::core::Option (:wat::kernel::Peer :- [S R]))]
  -> :wat::core::nil
  ;; arc 278 the recv'-outcome wall — (RecvOutcome :- [PoolMsg]). ::Message → dispatch;
  ;; ::Lost → eprintln (terminal); ::Closed → exit the runner.
  (:wat::core::match (:wat::kernel::recv self)  
    ((:wat::kernel::RecvOutcome::Message m)
      (:wat::core::match m  
        ((:wat::bracket::PoolMsg::Setup deps)
          ;; arc 278 the connect'-outcome wall — face all four arms. ::Connected → hold the
          ;; dialed Peer as (Some p); failure arms → assertion-failed! (fatal, preserving
          ;; the pre-wall raise-unwind — the pool does NOT degrade/retry; that is a
          ;; deliberate follow-up if ever wanted, not this wall).
          (:wat::bracket::process-dial-runner self work-fn
            (:wat::core::match (:wat::kernel::connect deps)
              ((:wat::kernel::ConnectOutcome::Connected p) (:wat::core::Some p))
              ((:wat::kernel::ConnectOutcome::Refused c)
                (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
              ((:wat::kernel::ConnectOutcome::Rejected c)
                (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
              ((:wat::kernel::ConnectOutcome::Failed c)
                (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
        ((:wat::bracket::PoolMsg::Work pair)
          (:wat::core::let
            [c   (:wat::core::Option/expect ctx "bracket process-dial-runner: Work before Setup")
             out (:wat::core::Tuple (:wat::core::first pair) (work-fn c (:wat::core::second pair)))]
            ;; arc 278 the send'-outcome wall — face all three arms; a dead parent surfaces
            ;; via the next recv', so every arm proceeds to recurse.
            (:wat::core::match (:wat::kernel::send self out)
              (:wat::kernel::SendOutcome::Sent   (:wat::bracket::process-dial-runner self work-fn ctx))
              (:wat::kernel::SendOutcome::Stopped nil)                                                    ;; arc 278 #73 — the WORLD is stopping → exit
              (:wat::kernel::SendOutcome::Closed (:wat::bracket::process-dial-runner self work-fn ctx))   ;; parent gone → next recv' faces it
              ((:wat::kernel::SendOutcome::Lost _c) (:wat::bracket::process-dial-runner self work-fn ctx)))))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    ;; arc 278 #73 — exit like Closed, DIFFERENT reason: the parent did not drop,
    ;; the substrate is stopping. Same body, stated cause (never an unexplained twin).
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)))

;; ── spawn-runner — the per-tier runner spawn, lifted onto the :Locus surface ──
;;
;; The bracket coordinator (map-worker) is now loci-agnostic: it holds an abstract
;; :wat::spawn::Locus and calls (spawn-runner locus work-fn) once per pool tier.
;; The RAW work fn [I :-> O] is passed — NOT an index-wrapping closure. Each tier
;; does its own index-wrapping over the raw fn:
;;
;;   THREAD (shared memory): build the (idx,I)->(idx,O) wrapper inline as a thread
;;   closure (captures work-fn freely — no reification), run runner-loop on it.
;;
;;   PROCESS (not-shared): fn-forms the RAW work-fn (top-level, no captured fn —
;;   the one shape fn-forms/closure_extract slice-1 CAN reify) into :__pool-work,
;;   then ship a NAMED index-wrapping pool-runner as source (the defservice fork
;;   trick). Mirrors scratchpad/probe-s3-process-runner.wat.
;;
;; Both return (:wat::kernel::Peer :- [(:wat::core::Tuple :- [:wat::core::i64 I]) (:wat::core::Tuple :- [:wat::core::i64 O])]) so collect-loop drains a
;; uniform (Vector :- [(Peer :- […])]) (select' accepts Peer as of S3a).

;; Thread spawn-runner — TWO mouths, same split as Locus/launch.
;;
;; Thread launch (spawn.wat): companions already live in this universe;
;; apply init/serve by keyword; service-forms is ignored.
;; Process launch: ship forms; the child re-freezes.
;;
;; Thread kwargs is the same cell. `<base>::assemble` (Coords→Kwargs) and
;; `<base>$impl` were minted next to the work-fn. Setup applies assemble
;; and holds the peer bundle; Work applies $impl. A thread worker can
;; reach a process service (same pid as the owner).
;;
;; Process kwargs still ships a generated dial-runner (separate memory).
;;
;; Dispatch is the same first-match-wins keyword-vs-W defclause as
;; process-work-forms: a kwargs work-fn arrives as a bare keyword; a
;; plain work-fn is a [I :-> O]. Plain: Setup stays a raise.
(:wat::core::defn :wat::bracket::thread-kwargs-runner :- [D K I O]
  [self    <- (:wat::kernel::ThreadSelfPeer :- [(:wat::core::Tuple :- [:wat::core::i64 O]) (:wat::bracket::PoolMsg :- [D I])])
   work-fn <- :wat::core::keyword
   ctx     <- (:wat::core::Option :K)]
  -> :wat::core::nil
  (:wat::core::let
    [base-str     (:wat::core::keyword/to-string work-fn)
     assemble-kw  (:wat::core::keyword/from-string
                    (:wat::core::format "{base-str}::assemble" :base-str base-str))
     impl-kw      (:wat::core::keyword/from-string
                    (:wat::core::format "{base-str}$impl" :base-str base-str))]
    (:wat::core::match (:wat::kernel::recv self)
      ((:wat::kernel::RecvOutcome::Message m)
        (:wat::core::match m
          ((:wat::bracket::PoolMsg::Setup deps)
            (:wat::bracket::thread-kwargs-runner self work-fn
              (:wat::core::Some
                (:wat::core::apply assemble-kw deps (:wat::core::Vector :wat::core::nil)))))
          ((:wat::bracket::PoolMsg::Work pair)
            (:wat::core::let
              [k   (:wat::core::Option/expect ctx "bracket thread-kwargs-runner: Work before Setup")
               out (:wat::core::Tuple (:wat::core::first pair)
                     (:wat::core::apply impl-kw (:wat::core::second pair)
                       (:wat::core::Vector :K k)))]
              (:wat::core::match (:wat::kernel::send self out)
                (:wat::kernel::SendOutcome::Sent   (:wat::bracket::thread-kwargs-runner self work-fn ctx))
                (:wat::kernel::SendOutcome::Stopped nil)
                (:wat::kernel::SendOutcome::Closed (:wat::bracket::thread-kwargs-runner self work-fn ctx))
                ((:wat::kernel::SendOutcome::Lost _c) (:wat::bracket::thread-kwargs-runner self work-fn ctx)))))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped nil)
      (:wat::kernel::RecvOutcome::Closed nil))))

(:wat::core::defclause :wat::bracket::thread-enter
  ([self    <- (:wat::kernel::ThreadSelfPeer :- [(:wat::core::Tuple :- [:wat::core::i64 O]) (:wat::bracket::PoolMsg :- [D I])])
    work-fn <- :wat::core::keyword] -> :wat::core::nil
   (:wat::bracket::thread-kwargs-runner self work-fn :wat::core::None))
  ([self    <- (:wat::kernel::ThreadSelfPeer :- [(:wat::core::Tuple :- [:wat::core::i64 O]) (:wat::bracket::PoolMsg :- [D I])])
    work-fn <- :W] -> :wat::core::nil
   (:wat::bracket::runner-loop self
     (:wat::core::fn [m <- (:wat::bracket::PoolMsg :- [D I])] -> (:wat::core::Tuple :- [:wat::core::i64 O])
       (:wat::core::match m
         ((:wat::bracket::PoolMsg::Work pair)
           (:wat::core::Tuple (:wat::core::first pair) (work-fn (:wat::core::second pair))))
         ((:wat::bracket::PoolMsg::Setup _deps)
           (:wat::kernel::assertion-failed!
             "bracket thread runner: unexpected PoolMsg::Setup (plain thread pool — no kwargs tail)"
             :wat::core::None :wat::core::None)))))))

(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus
  (spawn-runner [self work-fn]
    (:wat::kernel::spawn-program self
      (:wat::core::fn [sp <- (:wat::kernel::ThreadSelfPeer :- [(:wat::core::Tuple :- [:wat::core::i64 O]) (:wat::bracket::PoolMsg :- [D I])])] -> :wat::core::nil
        (:wat::bracket::thread-enter sp work-fn)))))

;; The PROCESS arm (not-shared) — bakes the runner, ships only the user's code
;; (259 S3c; supersedes the S3b shipped-runner shape).
;;
;; The runner is BAKED (`(:wat::bracket::process-runner :- [I O])` above) — nothing
;; reserved is shipped, so the `ReservedPrefix` problem S3b fought (an
;; un-squattable shipped name has nowhere safe to live in `:wat::`) is simply
;; gone.  We ship only: the user's work-fn, reified at the rendezvous
;; coordinate `:user::bracket::work-fn` (fn-forms), plus a generated
;; `:user::main` that calls the baked runner, passing that coordinate's VALUE
;; in (the runner is baked, so `:user::main` passes the value — it cannot look
;; the coordinate up from stdlib; that would be a stdlib -> user-data
;; forward reference the resolver rejects).
;;
;; `:user::main`'s `self-peer` call still needs CONCRETE peer types — a
;; generic runtime method can't monomorphize spawn-runner's `:I`/`:O`
;; type-params into shipped `forms` (they'd land literal and unbound in the
;; child universe). So we DERIVE the concrete arg/return types off the
;; reified work-fn: `fn-forms` emits a `(def :user::bracket::work-fn (fn [n <-
;; :ArgT] -> :RetT …))` whose ArgT/RetT are literal AST nodes. We AST-walk
;; them out (def → fn → argspec[after <-] + [after ->]), build the concrete
;; tuple-type keywords via `keyword-node`, and splice them into the shipped
;; `self-peer` tuple types via quasiquote.  The generic baked runner itself
;; needs no concrete types (it monomorphizes at the call).
;;
;; The fn-forms bind-name is a COMPUTED keyword (not a source literal): a
;; literal `:user::bracket::work-fn` here would, when the child re-typechecks
;; THIS file with that name shipped-as-a-def, resolve to the shipped fn (a Fn,
;; not a keyword) and fail fn-forms' `name` param. A computed keyword is
;; unresolvable at check → safe.
;; ── dotpath->colonpath — canonical wat.type/ dot-slash name -> wat's own colon-colon
;; keyword-string convention ─────────────────────────────────────────────────
;;
;; Arc 170 C1 — `field-types-of` (Strike B) renders each `::Kwargs` field's type
;; through the POST-arc-251 canonical form: `wat.kernel/Peer` (Symbol, dot/slash,
;; NO leading colon) rather than the surface literal `:wat::kernel::Peer` a plain
;; fn's own (unmangled) param-type keyword carries. The two conventions are
;; interchangeable AS TEXT (both resolve to the same registry key; the wat reader
;; accepts either "." or "::" as a namespace separator and "/" as the terminal
;; separator) — this just re-punctuates one into the other so the compound
;; angle-bracket keyword strings built below (`wat::kernel::Address<S,R>`, …)
;; stay in the ONE convention every other string in this AST-walk already uses.
;; "wat.kernel/Peer" -> "wat" "kernel/Peer" (split ".") -> "wat::kernel/Peer"
;;                     -> "wat::kernel" "Peer" (split "/") -> "wat::kernel::Peer"
(:wat::core::defn :wat::bracket::dotpath->colonpath [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::join "::"
    (:wat::core::string::split
      (:wat::core::string::join "::" (:wat::core::string::split s "."))
      "/")))

;; Arc 170 M1-pool — the AST-walk now also distinguishes the DIAL work-fn. A
;; non-dial work-fn is 1-param `[I :-> O]` (argspec = 3 AST children: `n <- :I`);
;; a dial work-fn is 2-param `[(Peer :- [S R]) I :-> O]` (6 children: `c <- (Peer :- [S R])
;; n <- :I`). We branch on that count:
;;   NON-DIAL → bake `process-runner`, self-peer recvs `(PoolMsg :- [Address I])` (D phantom).
;;   DIAL     → bake `process-dial-runner` (threads the held peer + starts at None),
;;              self-peer recvs the CONCRETE `(PoolMsg :- [(Address :- [S R]) I])` — S,R lifted off
;;              the 1st param `(Peer :- [S R])` by a `Peer`→`Address` head-swap (split/join),
;;              so the child's connect' gets a fully-typed address. The item type I is the
;;              LAST param either way; O is the return.
;;
;; Arc 170 C1 (N-service kwargs, N=1) — `process-work-forms` is a NEW dispatch point,
;; a defclause keyed on the WORK-FN VALUE's runtime type: a literal keyword naming a
;; kwargs `defn`'s companion MACRO (`:probe::work`) evaluates to a bare
;; `:wat::core::keyword` (the arc-009 literal-keyword lift only fires for a registered
;; FN, and a companion is a macro — never in `sym.functions`; see runtime.rs:3729-3740 /
;; macros/registry.rs), whereas a plain (non-kwargs) work-fn keyword auto-upgrades to a
;; `Value::wat__core__fn` at the call site, same as always. The `:wat::core::keyword`
;; clause is declared FIRST — defclause dispatch is first-match-wins and a Fn/generic-`W`
;; clause is a PERMISSIVE catch-all at runtime (`value_matches_type_by_name`,
;; runtime.rs:6604-6606), so ordering is load-bearing: the keyword clause must be checked
;; before the generic one or it would never fire. This is exactly why the recognition
;; must NOT reflect the work-fn VALUE (metadata-of/lookup-define/field-names-of on an
;; anonymous fn raises TypeMismatch and crashes the parent, per the design doc's STOP-4) —
;; the keyword-vs-fn distinction is made by defclause's own type dispatch, for free.
(:wat::core::extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus
  (spawn-runner [self work-fn]
    (:wat::kernel::spawn-program self (:wat::bracket::process-work-forms work-fn))))

;; Arc 109 ③ — angle brackets are ILLEGAL for types, so a parametric type slot reflected off
;; a reified work-fn's argspec (e.g. `Peer<S,R>`) now arrives as the reference FORM
;; `(Head :- [args])`, a List — `ast-name` only reads Symbol/Keyword/StringLit, so it raises
;; outright on that shape. `process-work-forms` (below) derives new types (the self-peer send/
;; recv types, the `Peer`→`Address` head-swap) off a reflected type slot by `ast-name` +
;; string surgery; these two helpers are the ONE door both shapes go through. Plain top-level
;; `defn`s (unlike `wat/core.wat`'s equivalent pair): `process-work-forms` is a `defclause`,
;; not a `defmacro`, so there is no program-body purity gate here to route around.
;;
;; -type-slot-name — structural type-NAME text of a type-position node, whether spelled as a
;; bare Keyword or the `(Head :- [args])` List form (reads the List's own head).
(:wat::core::defn :wat::bracket::-type-slot-name
  [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children node)))
    (:wat::core::ast-name node)))

;; -type-slot-swap-head — rebuild a type-position node with its HEAD keyword's text
;; substring-substituted `old`->`new`, preserving shape: a bare Keyword becomes a bare
;; Keyword; a `(Head :- [args])` List keeps the SAME `:- [args]` tail — only Head's text
;; changes, so the args (however deeply nested) survive untouched.
(:wat::core::defn :wat::bracket::-type-slot-swap-head
  [node <- :wat::WatAST old <- :wat::core::String new <- :wat::core::String] -> :wat::WatAST
  (:wat::core::let
    [nm         (:wat::bracket::-type-slot-name node)
     swapped-kw (:wat::core::keyword-node (:wat::core::string::join new (:wat::core::string::split nm old)))]
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
      (:wat::core::let
        [ch     (:wat::core::ast->children node)
         tail   (:wat::core::rest ch)
         new-ch (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) x <- :wat::WatAST]
                    -> (:wat::core::Vector :- [:wat::WatAST])
                    (:wat::core::conj acc x))
                  (:wat::core::conj (:wat::core::Vector :wat::WatAST) swapped-kw)
                  tail)]
        (:wat::core::with-children node new-ch))
      swapped-kw)))

(:wat::core::defclause :wat::bracket::process-work-forms
  ;; ── KWARGS branch (arc 170 C1 ground case N=1, generalized to N by C2 Strike 1) ──
  ;; work-fn is a BASE NAME keyword naming a kwargs `defn`'s companion (e.g. :probe::enrich).
  ;; Ship `<base>$impl` BY NAME (the fn-forms keyword seam, c8e3c7ff) — its OWN 2-param
  ;; signature is [item <- :I  kwargs <- :<base>::Kwargs] (item FIRST, unlike the raw
  ;; dial shape where the peer is first). Read the ::Kwargs struct's N fields (field-order
  ;; preserved by field-names-of/field-types-of, Strike B) to recover each field's (Peer :- [S R])
  ;; — NEVER by reflecting the work-fn value itself. Then synthesize: (1) an N-ARY ADAPTER fn
  ;; [(Peer :- [S1 R1]) … (Peer :- [Sn Rn]) I :-> O] that assembles the ::Kwargs struct positionally (field
  ;; order) from the N held dial peers + calls the shipped $impl, and (2) an N-DIAL RUNNER
  ;; emitted as source (the baked single-peer `process-dial-runner` can't carry a variadic
  ;; type-param list — a (Tuple :- [(Address :- [S1 R1]) …]) carrier is concretely-typed PER CALL, so the
  ;; runner recv-ing it must be too; shape proven at
  ;; wat-scripts/probes/arc-170/w3-n-dial-runner.wat). N=1 is not special-cased — it is the
  ;; ground case of the same fold (a 1-element Tuple carrier/ctx).
  ([work-fn <- :wat::core::keyword] -> (:wat::core::Vector :- [:wat::WatAST])
    (:wat::core::let
      [base-str      (:wat::core::keyword/to-string work-fn)
       impl-kw       (:wat::core::keyword/from-string (:wat::core::format "{base-str}$impl" :base-str base-str))
       kwargs-ty-str (:wat::core::format "{base-str}::Kwargs" :base-str base-str)
       kwargs-ty     (:wat::core::keyword/from-string kwargs-ty-str)
       work-name     (:wat::core::keyword/from-string "user::bracket::work-fn")
       forms         (:wat::kernel::fn-forms impl-kw work-name)
       nforms        (:wat::core::length forms)
       ;; The $impl fn-def-node — the SECOND-TO-LAST shipped form (fn-forms, given a
       ;; KEYWORD naming an ALREADY-REGISTERED fn, ships its own canonical `defn` decl
       ;; verbatim + a trailing `(def work-name <the-name>)` rebind; unlike the Fn-VALUE
       ;; path, which inlines the fn body directly into ONE trailing `(def work-name (fn …))`
       ;; — measured via wat-scripts/probes/arc-170/probe-c1-kwargs-impl-astname.wat).
       def-node      (:wat::core::Option/expect (:wat::core::get forms (:wat::core::i64::- nforms 2))
                       "process-work-forms(kwargs): fn-forms produced no $impl define")
       dn-ch         (:wat::core::ast->children def-node)
       argspec       (:wat::core::Option/expect (:wat::core::get dn-ch 2) "process-work-forms(kwargs): no argspec")
       arg-ch        (:wat::core::ast->children argspec)
       ;; item = the $impl's FIRST param's type (index 2 of the flat [name <- ty …] triple
       ;; list) — NOT last, unlike the raw dial shape (kwargs puts item before the bundle).
       item-ty       (:wat::core::Option/expect (:wat::core::get arg-ch 2) "process-work-forms(kwargs): no item type")
       ret-ty        (:wat::core::Option/expect (:wat::core::get dn-ch 4) "process-work-forms(kwargs): no ret type")
       ;; ── arc 170 C2 Strike 1 (record redirect): the ::Kwargs fields, reconciled BY NAME ──
       ;; The coords carrier D is the `<base>::Coords` RECORD (minted at the kwargs-defn site,
       ;; wat/core.wat) — addressed by field NAME, so N has NO positional-accessor cap and DATA
       ;; fields fall out for free. The runner recv's ONE `::Coords` record as the Setup payload,
       ;; reconciles it → `::Kwargs` by field name (Peer field → connect' the Address; data field
       ;; → copy the value through, routed off `field-types-of`), holds the assembled `::Kwargs`,
       ;; and invokes `$impl` per Work item. `fnames`/`ftypes` are field-ordered + positionally
       ;; aligned (Strike B), so a single fold over 0..n builds the ordered ctor args.
       fnames        (:wat::runtime::field-names-of kwargs-ty)
       ftypes        (:wat::runtime::field-types-of kwargs-ty)
       n             (:wat::core::length ftypes)
       _n-check      (:wat::core::if (:wat::core::= (:wat::core::length fnames) n)
                        nil
                       (:wat::kernel::assertion-failed!
                         "bracket process-work-forms: field-names-of/field-types-of length mismatch"
                         :wat::core::None :wat::core::None))
       coords-ty-str (:wat::core::format "{base-str}::Coords" :base-str base-str)
       coords-ty-kw  (:wat::core::keyword-node (:wat::core::string::concat ":" coords-ty-str))
       ;; Arc 109 ③ — angle brackets are ILLEGAL for types; sp-out/sp-in/runner-self-kw/
       ;; ctx-ty-kw used to round-trip `item-ty`/`ret-ty` through `ast-name` + string
       ;; concatenation into an angle-bracket keyword — now illegal, and it would have raised
       ;; outright the moment either type was itself parametric (`ast-name` only reads
       ;; Symbol/Keyword/StringLit). Mint the reference FORM `(Head :- [args])` structurally
       ;; off the type-position NODES directly instead — no string round-trip at all.
       sp-out        `(:wat::core::Tuple :- [:wat::core::i64 ~ret-ty])
       ;; sp-in D = the ::Coords RECORD (a plain type path), NOT a Tuple: (PoolMsg :- [<base>::Coords I]).
       sp-in         `(:wat::bracket::PoolMsg :- [~coords-ty-kw ~item-ty])
       runner-self-kw `(:wat::kernel::Peer :- [~sp-out ~sp-in])
       ;; ctx holds the assembled ::Kwargs (the N-heterogeneous dialed-peer bundle), None until Setup.
       ctx-ty-kw     `(:wat::core::Option :- [~(:wat::core::keyword-node (:wat::core::string::concat ":" kwargs-ty-str))])
       kwargs-kw     (:wat::core::keyword-node (:wat::core::format ":{kwargs-ty-str}" :kwargs-ty-str kwargs-ty-str))
       ;; kwargs-prime-kw: the POSITIONAL ctor for the just-defined ::Kwargs aggregate. Post-flip,
       ;; the bare `kwargs-kw` name is the KWARGS MACRO (unresolved as a positional call) — generated
       ;; construction must go through the type-name PRIME, mirroring core.wat's coords-prime-kw.
       kwargs-prime-kw (:wat::core::keyword-node (:wat::core::string::concat ":" (:wat::core::string::concat kwargs-ty-str "'")))
       ;; kwargs-ctor-args: one form per ::Kwargs field, DECLARED order, each read off the ::Coords
       ;; record BY NAME (`(:<base>::Coords/<field> deps)`). A Peer-typed field (its ::Kwargs type
       ;; is a (Peer :- [S R]) — an `ast-kind` "list" whose head names Peer) gets `connect'`ed
       ;; (Address→Peer); a data field is copied through verbatim. `deps` is the Setup binder
       ;; symbol (literal in the runner quasiquote below — the same across-quasiquote literal-symbol
       ;; reference the C1 dial-runner already used, proven to survive the ship-as-source round-trip).
       kwargs-ctor-args
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
           (:wat::core::let
             [fname-str   (:wat::core::keyword/to-string (:wat::core::Option/expect (:wat::core::get fnames i) "process-work-forms(kwargs): fnames index"))
              accessor-kw (:wat::core::keyword-node
                            (:wat::core::string::concat ":"
                              (:wat::core::string::concat coords-ty-str
                                (:wat::core::string::concat "/" fname-str))))
              ft          (:wat::core::Option/expect (:wat::core::get ftypes i) "process-work-forms(kwargs): ftypes index")
              is-peer     (:wat::core::if (:wat::core::= (:wat::core::ast-kind ft) "list")
                            (:wat::core::string::contains?
                              (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children ft))) "Peer")
                            false)
              ;; arc 278 the connect'-outcome wall — a Peer-typed field's generated dial
              ;; FACES the outcome: ::Connected → the Peer; failure arms → assertion-failed!
              ;; (fatal, preserving the pre-wall raise-unwind). Arm-local p/c are literal in
              ;; the generated code (arm-scoped; they don't escape the match).
              form        (:wat::core::if is-peer
                            `(:wat::core::match (:wat::kernel::connect (~accessor-kw deps))
                               ((:wat::kernel::ConnectOutcome::Connected p) p)
                               ((:wat::kernel::ConnectOutcome::Refused c)
                                 (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                               ((:wat::kernel::ConnectOutcome::Rejected c)
                                 (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                               ((:wat::kernel::ConnectOutcome::Failed c)
                                 (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                            `(~accessor-kw deps))]
             (:wat::core::conj acc form)))
         (:wat::core::Vector :wat::WatAST)
         (:wat::core::range 0 n))
       ;; N-DIAL RUNNER — emitted as source. recv (PoolMsg :- [::Coords I]); Setup deps (a ::Coords record)
       ;; → reconcile-by-name into the ::Kwargs bundle (connect' the Peer fields, copy the data
       ;; fields), HOLD it; Work pair → invoke `$impl` (via the `:user::bracket::work-fn` keyword,
       ;; arc-009-lifted through `apply`'s head — the C1-proven invocation seam) with the item + the
       ;; held ::Kwargs bundle, send the indexed result, recurse. No separate adapter fn: the
       ;; reconciliation IS the assembly, done once at Setup.
       runner-def
       `(:wat::core::defn :user::bracket::dial-runner
          [self <- ~runner-self-kw
           ctx  <- ~ctx-ty-kw]
          -> :wat::core::nil
          (:wat::core::match (:wat::kernel::recv self)  
            ((:wat::kernel::RecvOutcome::Message m)
              (:wat::core::match m  
                ((:wat::bracket::PoolMsg::Setup deps)
                  (:user::bracket::dial-runner self
                    (:wat::core::Some (~kwargs-prime-kw ~@kwargs-ctor-args))))
                ((:wat::bracket::PoolMsg::Work pair)
                  (:wat::core::let
                    [k   (:wat::core::Option/expect ctx "dial-runner: Work before Setup")
                     out (:wat::core::Tuple (:wat::core::first pair)
                           (:wat::core::apply  :user::bracket::work-fn (:wat::core::second pair) [k]))]
                    ;; arc 278 the send'-outcome wall — face all three arms; a dead parent
                    ;; surfaces via the next recv', so every arm proceeds to recurse.
                    (:wat::core::match (:wat::kernel::send self out)
                      (:wat::kernel::SendOutcome::Sent   (:user::bracket::dial-runner self ctx))
                      (:wat::kernel::SendOutcome::Stopped nil)                                     ;; arc 278 #73 — the WORLD is stopping → exit
                      (:wat::kernel::SendOutcome::Closed (:user::bracket::dial-runner self ctx))   ;; parent gone → next recv' faces it
                      ((:wat::kernel::SendOutcome::Lost _c) (:user::bracket::dial-runner self ctx)))))))
            ;; arc 278 the recv'-outcome wall — ::Lost → eprintln (terminal); ::Closed → exit.
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            ;; arc 278 #73 — exit like Closed, DIFFERENT reason: the parent did not
            ;; drop, the substrate is stopping. Same body, stated cause.
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil)))
       main-def
       `(:wat::core::defn :user::main [] -> :wat::core::nil
          (:user::bracket::dial-runner
            (:wat::program::self-peer ~sp-out ~sp-in)
            :wat::core::None))]
      (:wat::core::concat forms (:wat::core::Vector :wat::WatAST runner-def main-def))))
  ;; ── existing Fn branch (arc 170 M1-pool, arity 3/6 dispatch) — UNCHANGED logic,
  ;; only the tail (spawn-program' call -> plain forms-vector return) is refactored so
  ;; both clauses share the one call site above.
  ([work-fn <- :W] -> (:wat::core::Vector :- [:wat::WatAST])
    (:wat::core::let
      [work-name (:wat::core::keyword/from-string "user::bracket::work-fn")
       forms     (:wat::kernel::fn-forms work-fn work-name)
       ;; ── derive the concrete arg/return type keywords off the reified work-fn ──
       def-node  (:wat::core::Option/expect (:wat::core::last forms) "spawn-runner: fn-forms produced no define")
       fn-form   (:wat::core::nth (:wat::core::ast->children def-node) 2)
       fn-ch     (:wat::core::ast->children fn-form)
       argspec   (:wat::core::nth fn-ch 1)
       arg-ch    (:wat::core::ast->children argspec)
       arity     (:wat::core::length arg-ch)   ;; 3 = 1-param (non-dial); 6 = 2-param (dial)
       ;; item type I = the LAST param's type (both arities); O = the return type.
       arg-ty    (:wat::core::Option/expect (:wat::core::last arg-ch) "spawn-runner: work-fn has no arg type")
       ret-ty    (:wat::core::nth fn-ch 3)
       ;; Arc 109 ③ — angle brackets are ILLEGAL for types; sp-out/sp-in/addr used to round-
       ;; trip `arg-ty`/`ret-ty`/`c-ty` through `ast-name` + string concatenation into an
       ;; angle-bracket keyword — now illegal, and `ast-name` raises outright the moment any
       ;; of them is itself parametric (a `WatAST::List`, not Symbol/Keyword/StringLit). Mint
       ;; the reference FORM `(Head :- [args])` structurally off the type-position NODES
       ;; directly; the DIAL branch's `Peer`->`Address` head-swap routes through
       ;; `-type-slot-swap-head` (defined above), which handles both shapes the same way.
       ;; ── self-peer SEND type = (i64,O) (output tuple), both arities ──
       sp-out    `(:wat::core::Tuple :- [:wat::core::i64 ~ret-ty])
       ;; ── main-def — dispatch on arity ──
       main-def
       (:wat::core::if (:wat::core::= arity 6)
         ;; DIAL: derive (Address :- [S R]) off the 1st param (Peer :- [S R]); recv (PoolMsg :- [(Address :- [S R]) I]).
         (:wat::core::let
           [c-ty  (:wat::core::nth arg-ch 2)          ;; 1st param's TYPE node
            addr  (:wat::bracket::-type-slot-swap-head c-ty "Peer" "Address")
            sp-in `(:wat::bracket::PoolMsg :- [~addr ~arg-ty])]
           `(:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::bracket::process-dial-runner
                (:wat::program::self-peer ~sp-out ~sp-in)
                :user::bracket::work-fn
                :wat::core::None)))
         ;; NON-DIAL: recv (PoolMsg :- [Address I]) (D phantom — no Setup ever sent).
         (:wat::core::let
           [sp-in `(:wat::bracket::PoolMsg :- [:wat::kernel::Address ~arg-ty])]
           `(:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::bracket::process-runner
                (:wat::program::self-peer ~sp-out ~sp-in)
                :user::bracket::work-fn))))]
      (:wat::core::concat forms (:wat::core::Vector :wat::WatAST main-def)))))

;; ── collect-loop — tail-recursive collector; drains M results from N runners ──
;;
;; State: peers (the live Thread vector), items (the full input vector),
;; pairs-acc (accumulator of (idx,result) pairs so far), cursor (next item
;; to dispatch), collected (how many results have arrived), m (total item count).
;;
;; Invariant: cursor ≤ m; collected ≤ m.  When collected == m every result
;; has arrived; return pairs-acc (unsorted — the caller sorts).
;;
;; Dynamic balance: after select' returns the ServiceEvent::Message{idx=peer-pos, msg=pair}
;; for whichever runner finished first, that runner's channel is empty again
;; and we immediately feed it the next pending item (if cursor < m).  Runners
;; that had no item sent to them (when M < N) are simply never select'ed —
;; the channel-drain RAII at scope exit joins them cleanly.
;;
;; select' now returns (ServiceEvent :- [I O]) (Stone 259 Lost-locus).  :Message is
;; the normal case.  :Closed/:Lost are honest arms — a bracket runner should
;; never disconnect or crash in normal operation; if it does, raise via
;; assertion-failed! so the failure is visible rather than silently swallowed.

;; Arc 170 C2 Strike 1c — generalized `Address` (bare) to `D`. Purely a WIDENING of the
;; declared type (this fn's own logic never touches the Setup/D payload — it only ever
;; handles Work/select' events on the (i64,O) channel). Originally needed so the (since-
;; retired) `uses'` coordinator — which back then spawned DIRECTLY via `spawn-program'`,
;; bypassing `Locus/spawn-runner`, because that surface's return type was FIXED to a bare
;; `Address` and a `::Coords` record couldn't round-trip through it — could reuse this SAME
;; collector for its `::Coords`-carrying peers. Arc 170 gap J made `Locus/spawn-runner` itself
;; D-generic (wat/spawn.wat), removing that constraint entirely and letting `uses'` fold into
;; `map-worker` (both now flow through the ONE `spawn-runner` call); this generalization
;; remains because `map-worker` itself is the one caller for every D (nil OR `::Coords`) and
;; needs the same widening `collect-loop` already had.
(:wat::core::defn :wat::bracket::collect-loop :- [D I O]
  [peers     <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [(:wat::bracket::PoolMsg :- [D I]) (:wat::core::Tuple :- [:wat::core::i64 O])])])
   items     <- (:wat::core::Vector :- [I])
   pairs-acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 O])])
   cursor    <- :wat::core::i64
   collected <- :wat::core::i64
   m         <- :wat::core::i64]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 O])])
  (:wat::core::if (:wat::core::= collected m)
    pairs-acc
    (:wat::core::let
      [event    (:wat::kernel::select peers)]
      (:wat::core::match event
         
        ((:wat::spawn::ServiceEvent::Message peer-pos pair)
          (:wat::core::let
            [cursor'  (:wat::core::if (:wat::core::< cursor m)
                        ;; arc 278 the send'-outcome wall — face all three arms explicitly.
                        ;; A dead runner here surfaces via THIS loop's own select' arm
                        ;; (:Closed/:Lost above, which raise) — this dispatch always advances
                        ;; the cursor regardless of outcome.
                        (:wat::core::match (:wat::kernel::send
                                              (:wat::core::nth peers peer-pos)
                                              (:wat::bracket::PoolMsg::Work
                                                (:wat::core::Tuple cursor (:wat::core::nth items cursor))))
                          (:wat::kernel::SendOutcome::Sent   (:wat::core::+ cursor 1))
                          (:wat::kernel::SendOutcome::Stopped (:wat::core::+ cursor 1))  ;; arc 278 #73 — same: this loop's select' arm faces the stop
                          (:wat::kernel::SendOutcome::Closed (:wat::core::+ cursor 1))   ;; surfaces via this loop's own select' arm
                          ((:wat::kernel::SendOutcome::Lost _c) (:wat::core::+ cursor 1)))
                        cursor)]
            (:wat::bracket::collect-loop peers items
              (:wat::core::conj pairs-acc pair) cursor' (:wat::core::+ collected 1) m)))
        ((:wat::spawn::ServiceEvent::Closed idx)
          (:wat::kernel::assertion-failed!
            (:wat::core::string::interpolate
              "bracket collect-loop: runner {idx} closed unexpectedly"
              :idx idx)
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Lost idx cause)
          (:wat::kernel::assertion-failed!
            (:wat::core::string::interpolate
              "bracket collect-loop: runner {idx} crashed: {cause}"
              :idx idx :cause (:wat::kernel::Failure/message cause))
            :wat::core::None :wat::core::None))
        ;; arc 278 no-hidden-failures — a pool runner sent an UNDECODABLE result. A bracket
        ;; runner speaks a fixed (i64,O) protocol; garbage on that channel is a should-never-
        ;; happen. Mirror :Lost — raise LOUD with the rich decode reason (never a `_` wildcard
        ;; that would re-hide the failure this arc forbids).
        ((:wat::spawn::ServiceEvent::Malformed idx cause)
          (:wat::kernel::assertion-failed!
            (:wat::core::string::interpolate
              "bracket collect-loop: runner {idx} sent an undecodable result: {cause}"
              :idx idx :cause (:wat::kernel::Failure/message cause))
            :wat::core::None :wat::core::None))
        ;; arc 278 Stone 1a — a pool runner sent an OVER-FOO (over-budget) frame. A bracket
        ;; runner speaks a fixed (i64,O) protocol; an oversized result is a should-never-happen.
        ;; Mirror :Malformed — raise LOUD with the reason (never a `_` wildcard that re-hides it).
        ((:wat::spawn::ServiceEvent::Rejected idx cause)
          (:wat::kernel::assertion-failed!
            (:wat::core::string::interpolate
              "bracket collect-loop: runner {idx} sent an over-budget frame: {cause}"
              :idx idx :cause (:wat::kernel::Failure/message cause))
            :wat::core::None :wat::core::None))
        (:wat::spawn::ServiceEvent::Shutdown
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Shutdown event"
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Connection _peer)
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Connection event"
            :wat::core::None :wat::core::None))
        ((:wat::spawn::ServiceEvent::Admin _msg)
          (:wat::kernel::assertion-failed!
            "bracket collect-loop: unexpected Admin event (select' has no self-peer)"
            :wat::core::None :wat::core::None))))))

;; ── map-worker — the ONE carrier-generic pool coordinator (arc 170 gap J unification) ──
;;
;; Each runner i is built from `(worker-init i)`: the OUTER call is per-runner
;; setup (once, when the runner is built — the place to allocate a resource
;; reused across that runner's items); the INNER result is the per-item work-fn.
;; `worker-id` is the runner index passed to `worker-init`.  The coordinator
;; (spawn+prime+collect+sort) lives here ONCE; `map` and `each` are thin macros.
;;
;; This IS the former `uses'` (arc 170 C2 Strike 1c), generalized over `:wat::spawn::Locus`
;; (was pinned to `ProcessOpts`) with a plain pool as the trivial case — `uses'` deletes, its
;; body absorbed here. Provisioning (grant + Setup dial) is a PARAMETERIZED layer riding
;; orthogonally on the pool, never welded into it: this body never constructs a concrete
;; `Address`/`Capability` value itself (the earlier naive D-generic attempt broke exactly
;; there, via `Capability/coordinate`) — the carrier flows entirely from the caller, through
;; `spawn-runner` (D-generic as of this stone, wat/spawn.wat:305-311).
;;
;; `setup-carrier` is the Setup-carrier FOLD VECTOR (0-or-1 elements), not a bare `D` value: a PLAIN
;; caller passes an EMPTY `(Vector :- [D])` (D=nil) so the fold below sends ZERO `PoolMsg::Setup` —
;; the thread runner's `:Setup` arm still hard-raises `assertion-failed!` on receipt (a thread
;; pool never dials), so STOP-1's risk ("does the plain runner choke on a nil-carrying Setup?")
;; is dodged structurally, by never SENDING Setup at all for a plain pool, not by hoping the
;; runner tolerates the payload. A kwargs caller passes the ONE-element `[coords]` so exactly
;; one Setup crosses per worker — unchanged from `uses'`. `grant-handles`/`grant-fn`/`revoke-fn`
;; mirror `uses'` verbatim: plain passes `grant-handles = nil` (G=nil) and a no-op `grant-fn`/
;; `revoke-fn` pair — a LOCAL call, never a wire message, so calling it unconditionally (even
;; for a thread pool) is harmless. Arc 170 M1-pool's `worker-init`/W convention is unchanged:
;; W is the raw work-fn (a 1-param `[I :-> O]`, or — new this stone — the kwargs work-fn's bare
;; keyword, `process-work-forms`'s KWARGS defclause dispatching on the VALUE's runtime type).
(:wat::core::defn :wat::bracket::map-worker :- [D G I O W]
  [locus         <- :wat::spawn::Locus
   items         <- (:wat::core::Vector :- [I])
   worker-init   <- [:wat::core::i64 :-> W]
   grant-handles <- :G
   grant-fn      <- [G :wat::core::i64 :-> :wat::core::nil]
   revoke-fn     <- [G :wat::core::i64 :-> :wat::core::nil]
   setup-carrier        <- (:wat::core::Vector :- [D])]
  -> (:wat::core::Vector :- [O])
  (:wat::core::let
    [;; arc 170 closure #6 — the spawn ORIGIN for every runner's ps label. Captured HERE,
     ;; in map-worker's own body, so `call-site` reports the CALLER of map-worker (the
     ;; user's `bracket-map`/`each-worker` site). It must NOT be read inside the per-runner
     ;; closure below: the innermost frame there is `mapv`'s invocation of the anon fn, not
     ;; the user's call — which would label every process with this file instead of theirs.
     origin (:wat::kernel::call-site)
     m  (:wat::core::length items)
     rc (:wat::spawn::runner-count locus)
     n  (:wat::core::if (:wat::core::< rc m) rc m)
     ;; Arc 118.2a — `map` flipped LAZY; `peers` feeds `collect-loop` ((Vector :- [(Peer :- […])]) param
     ;; — repeatedly `select'`-ed, must be eager) and later `sort-by`, so materialize here.
     peers (:wat::core::mapv
             (:wat::core::fn [i <- :wat::core::i64]
                 -> (:wat::kernel::Peer :- [(:wat::bracket::PoolMsg :- [D I]) (:wat::core::Tuple :- [:wat::core::i64 O])])
               (:wat::core::let
                 [work-fn (worker-init i)                          ;; per-runner setup, once
                  ;; arc 170 closure #6 — label THIS runner with its own index before spawning
                  ;; it (the ps-visible `#wat.process/Bracket {:id N}`, wat/process.wat); a
                  ;; no-op for a thread locus (with-label's ThreadOpts arm).
                  locus-i (:wat::spawn::with-label locus
                            (:wat::process::Bracket
                              :id   i
                              :file (:wat::kernel::Frame/file origin)
                              :line (:wat::kernel::Frame/line origin)))
                  p (:wat::spawn::Locus/spawn-runner locus-i work-fn)
                  ;; GRANT-BOOT: if the far end is a process (peer-pid → Some pid), grant that
                  ;; kernel-vouched pid — a SINGLE typed call (a no-op for a plain pool: its
                  ;; grant-fn ignores both args). BEFORE the first item is sent, so the grant
                  ;; lands before the worker's work-fn dials. A thread peer (peer-pid → None)
                  ;; skips: the in-process handle IS the capability.
                  _ (:wat::core::match (:wat::kernel::peer-pid p)  
                      ((:wat::core::Some pid) (grant-fn grant-handles pid))
                      (:wat::core::None nil))
                  ;; SETUP-DIAL: fold over 0-or-1 carriers — empty (plain) sends NO Setup at
                  ;; all; one element (kwargs) sends exactly ONE `PoolMsg::Setup carrier`. Runs
                  ;; AFTER grant-boot (grant-then-dial) and BEFORE the first Work item so the
                  ;; peer is held first.
                  _ (:wat::core::foldl
                      (:wat::core::fn [_acc <- :wat::core::nil  c <- :D] -> :wat::core::nil
                        ;; arc 278 send'-outcome wall Phase 2: face all three arms explicitly —
                        ;; a dead runner at setup time surfaces later via collect-loop's own
                        ;; select' arm (Closed/Lost raises there); this fold's job is only to
                        ;; fire every worker's Setup, so every arm continues the fold.
                        (:wat::core::match (:wat::kernel::send p (:wat::bracket::PoolMsg::Setup c))
                          (:wat::kernel::SendOutcome::Sent   nil)
                          (:wat::kernel::SendOutcome::Stopped nil)  ;; arc 278 #73 — same: collect-loop's select' arm faces the stop
                          (:wat::kernel::SendOutcome::Stopped nil)  ;; arc 278 #73 — same: collect-loop's select' arm faces the stop
                      (:wat::kernel::SendOutcome::Closed nil)   ;; surfaces via collect-loop's select' arm
                          ((:wat::kernel::SendOutcome::Lost _c) nil)))
                      nil
                      setup-carrier)
                  ;; arc 278 the send'-outcome wall — the initial per-worker item primer. A dead
                  ;; runner surfaces via collect-loop's own select' arm; face all three explicitly.
                  _ (:wat::core::match (:wat::kernel::send p (:wat::bracket::PoolMsg::Work (:wat::core::Tuple i (:wat::core::nth items i))))
                      (:wat::kernel::SendOutcome::Sent   nil)
                      (:wat::kernel::SendOutcome::Stopped nil)  ;; arc 278 #73 — same: collect-loop's select' arm faces the stop
                      (:wat::kernel::SendOutcome::Closed nil)   ;; surfaces via collect-loop's select' arm
                      ((:wat::kernel::SendOutcome::Lost _c) nil))]
                 p))
             (:wat::core::range 0 n))
     pairs  (:wat::bracket::collect-loop peers items
              (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 O])) n 0 m)
     ;; REVOKE-SHUTDOWN: the drain is complete but the peers are still alive (still in scope,
     ;; still hold their Pidfd → peer-pid still Some). For each process peer, revoke its pid
     ;; (a no-op for a plain pool) — the grant a worker held cannot outlive its reaping. A
     ;; thread peer (None) skips. Runs BEFORE the return so no grant escapes the bracket.
     _revoke (:wat::core::foldl
               (:wat::core::fn [_acc <- :wat::core::nil
                                p    <- (:wat::kernel::Peer :- [(:wat::bracket::PoolMsg :- [D I]) (:wat::core::Tuple :- [:wat::core::i64 O])])]
                 -> :wat::core::nil
                 (:wat::core::match (:wat::kernel::peer-pid p)  
                   ((:wat::core::Some pid) (revoke-fn grant-handles pid))
                   (:wat::core::None nil)))
               nil
               peers)
     sorted (:wat::core::sort-by
              (:wat::core::fn [pr <- (:wat::core::Tuple :- [:wat::core::i64 O])] -> :wat::core::i64
                (:wat::core::first pr))
              pairs)]
    ;; Arc 118.2a — `map` flipped LAZY; the function's declared return type is `(Vector :- [O])`.
    (:wat::core::mapv
      (:wat::core::fn [pr <- (:wat::core::Tuple :- [:wat::core::i64 O])] -> :O
        (:wat::core::second pr))
      sorted)))

;; ── each-worker — general side-effect pool (per-runner state via worker-init) ─
;;
;; `map-worker` that DISCARDS: run worker-init-derived per-item fns over every
;; item through the pool, then return nil. Thin wrapper — the SAME provisioning
;; params ride through unchanged (the kwargs layer rides `each` for free, below).
(:wat::core::defn :wat::bracket::each-worker :- [D G I O W]
  [locus         <- :wat::spawn::Locus
   items         <- (:wat::core::Vector :- [I])
   worker-init   <- [:wat::core::i64 :-> W]
   grant-handles <- :G
   grant-fn      <- [G :wat::core::i64 :-> :wat::core::nil]
   revoke-fn     <- [G :wat::core::i64 :-> :wat::core::nil]
   setup-carrier        <- (:wat::core::Vector :- [D])]
  -> :wat::core::nil
  (:wat::core::do
    (:wat::bracket::map-worker locus items worker-init grant-handles grant-fn revoke-fn setup-carrier)
    nil))

;; ── const-worker-init — a properly-generic wrapper for macro-emitted worker-init closures ──
;;
;; `map`/`each` are macros: their emitted code is spliced into WHATEVER enclosing fn calls
;; them (often non-generic), so a raw `(:wat::core::fn [_wid <- :wat::core::i64] -> :W
;; work-fn) )` literal written INLINE by the macro's quasiquote template has no generic scope
;; to resolve the bare `:W` annotation against — a single-uppercase-letter type only resolves
;; within the `defn<...>` that DECLARES it (the OLD `map<I,O,W>`/`each<I,O,W>` fns worked
;; because their own body's `:W` resolved against their own header; a macro has no header of
;; its own). Discovered this stone: `ReturnTypeMismatch :anonymous produces Fn(i64)->i64;
;; signature declares :W`. This defn's `<W>` is a REAL generic scope, so its own body's `:W`
;; resolves per-call as usual; `map`/`each` just splice a CALL to it into their emitted code
;; (ordinary code inside a quasiquote template, evaluated later at the call site — NOT the
;; macro's own expansion-time computation, so the F5 purity gate doesn't apply here, unlike a
;; direct call from the macro body itself).
(:wat::core::defn :wat::bracket::const-worker-init :- [W]
  [work-fn <- :W] -> [:wat::core::i64 :-> W]
  (:wat::core::fn [_wid <- :wat::core::i64] -> :W work-fn))

;; ── map — the pool verb, plain OR kwargs-provisioned (arc 170 gap J ratified surface) ──────
;;
;; `(bracket/map locus items work-fn)` — plain pool, no tail.
;; `(bracket/map locus items work-fn :name val …)` — pooled map + N typed kwargs (services
;; grant+dialed, data copied) — exactly C2's mechanism, moved off the `uses` verb onto `map`
;; itself. A macro (was a `defn`): the optional trailing `:name val` pairs are unevaluated at
;; the call site.
;;
;; NOTE — inlined, not factored through a shared helper `defn`: arc 249 stone 249.2b-i's F5
;; purity gate (`validate_pure_total`, src/macros/eval.rs) DEFAULT-DENIES any keyword head in a
;; macro's body that is not on the Rust-side blessed pure-combinator allow-list — a call to a
;; user-defined wat fn is refused at `defmacro` DEFINITION time (`MalformedDefmacro`), so `map`
;; and `each` cannot share this parse through a `:wat::bracket::pool-call` helper (tried; walled
;; immediately — out of scope to widen the allow-list, that is a `src/` change, STOP-3). The
;; identical logic below is therefore DUPLICATED verbatim in both macros (the one legitimate
;; exception to replicate-is-a-smell this substrate imposes on program-body macros) — it is the
;; SAME shape as the former `bracket/uses` macro, just retargeting the coordinator call:
;;
;; NO TAIL (`kwpairs` empty) → plain pool: `D=nil` (an empty `(Vector :- [D])` — zero Setup ever
;; sent), `G=nil` with a no-op `grant-fn`/`revoke-fn` pair — `work-fn` is spliced verbatim into
;; the worker-init closure exactly as the old `map`/`each` fns did (an arbitrary Fn-valued
;; expression, not necessarily a literal keyword).
;;
;; TAIL PRESENT → the former `bracket/uses` macro's EXACT parse (`work-fn` must be a literal
;; keyword AST node — its base name string is read at MACRO-EXPANSION time via `ast-name`, same
;; idiom `:wat::core::defn` itself uses on its own `name` param — to build the three auto-minted
;; coordinates `::kwargs-check`/`::grant-worker`/`::revoke-worker` plus the `::Coords` carrier
;; type): expands to
;;   (let [pair    (<base>::kwargs-check :name val …)
;;         coords  (first pair)
;;         handles (second pair)]
;;     (map-worker locus items (fn [_wid] -> W work-fn)
;;       handles <base>::grant-worker <base>::revoke-worker [coords]))
(:wat::core::defmacro :wat::bracket::map
  [locus <- :wat::WatAST
   items <- :wat::WatAST
   work-fn <- :wat::WatAST
   & kwpairs <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::= (:wat::core::length kwpairs) 0)
    
    ;; Arc 249 stone 249.2b-ii (hygiene bound gate E) — a quasiquote template may not
    ;; introduce a LITERAL name in binder position (it could capture a caller-site name); every
    ;; fn param below is a `fresh-symbol`, spliced via `~`, never a bare `_g`/`_pid`. The
    ;; worker-init closure itself is built by `const-worker-init` (see above) rather than an
    ;; inline `(fn [_wid] -> :W …)` literal — a macro-emitted `:W` has no enclosing generic
    ;; scope to resolve against.
    (:wat::core::let
      [g1-sym   (:wat::core::fresh-symbol "g")
       pid1-sym (:wat::core::fresh-symbol "pid")
       g2-sym   (:wat::core::fresh-symbol "g")
       pid2-sym (:wat::core::fresh-symbol "pid")]
      `(:wat::bracket::map-worker ~locus ~items
         (:wat::bracket::const-worker-init ~work-fn)
         nil
         (:wat::core::fn [~g1-sym <- :wat::core::nil ~pid1-sym <- :wat::core::i64] -> :wat::core::nil nil)
         (:wat::core::fn [~g2-sym <- :wat::core::nil ~pid2-sym <- :wat::core::i64] -> :wat::core::nil nil)
         (:wat::core::Vector :wat::core::nil)))
    (:wat::core::let
      [work-fn-name  (:wat::core::ast-name work-fn)
       base-str      (:wat::core::string::subs work-fn-name 1 (:wat::core::string::length work-fn-name))
       checker-kw    (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::kwargs-check")))
       grant-fn-kw   (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::grant-worker")))
       revoke-fn-kw  (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::revoke-worker")))
       coords-ty-kw  (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::Coords")))
       ;; 293.W.2f — process runner door. A ProcessOpts constructor locus (or
       ;; with-label wrapping one) must not receive a Shared-memory handle.
       locus-head    (:wat::core::if (:wat::core::= (:wat::core::ast-kind locus) "list")
                        (:wat::core::let [lch (:wat::core::ast->children locus)]
                          (:wat::core::if (:wat::core::empty? lch) "" (:wat::core::ast-name (:wat::core::first lch))))
                        "")
       locus-inner   (:wat::core::if (:wat::core::= locus-head ":wat::spawn::with-label")
                        (:wat::core::let [lch (:wat::core::ast->children locus)]
                          (:wat::core::if (:wat::core::empty? (:wat::core::rest lch))
                            ""
                            (:wat::core::let [inner (:wat::core::first (:wat::core::rest lch))]
                              (:wat::core::if (:wat::core::= (:wat::core::ast-kind inner) "list")
                                (:wat::core::let [ich (:wat::core::ast->children inner)]
                                  (:wat::core::if (:wat::core::empty? ich) "" (:wat::core::ast-name (:wat::core::first ich))))
                                ""))))
                        locus-head)
       process-door? (:wat::core::string::starts-with?
                       (:wat::core::if (:wat::core::= locus-head ":wat::spawn::with-label") locus-inner locus-head)
                       ":wat::spawn::process")
       wire-pairs    (:wat::core::if process-door?
                        (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                           i   <- :wat::core::i64]
                            -> (:wat::core::Vector :- [:wat::WatAST])
                            (:wat::core::let
                              [item (:wat::core::Option/expect
                                      (:wat::core::get kwpairs i)
                                      "bracket/map: kwpair")]
                              (:wat::core::if (:wat::core::= (:wat::core::i64::mod i 2) 1)
                                (:wat::core::conj acc `(:wat::kernel::require-wire-address ~item))
                                (:wat::core::conj acc item))))
                          (:wat::core::Vector :wat::WatAST)
                          (:wat::core::range 0 (:wat::core::length kwpairs)))
                        kwpairs)
       checker-call  `(~checker-kw ~@wire-pairs)
       pair-sym      (:wat::core::fresh-symbol "pair")
       coords-sym    (:wat::core::fresh-symbol "coords")
       handles-sym   (:wat::core::fresh-symbol "handles")]
      `(:wat::core::let
         [~pair-sym    ~checker-call
          ~coords-sym  (:wat::core::first ~pair-sym)
          ~handles-sym (:wat::core::second ~pair-sym)]
         (:wat::bracket::map-worker ~locus ~items
           (:wat::bracket::const-worker-init ~work-fn)
           ~handles-sym ~grant-fn-kw ~revoke-fn-kw
           (:wat::core::Vector ~coords-ty-kw ~coords-sym))))))

;; ── each — the SAME pool verb, side-effecting (Ruby's Parallel.each) ───────────────────────
;;
;; `(bracket/each locus items work-fn)` / `(bracket/each locus items work-fn :name val …)` —
;; identical tail grammar to `map`, riding `each-worker` (map-worker + discard) instead. See
;; `map`'s note above for why this is a verbatim duplicate rather than a shared helper call
;; (the F5 macro-purity gate refuses user-defn heads in a macro body) — the kwargs layer rides
;; `each` "for free" in the sense that it is the identical parse, not a shared implementation.
(:wat::core::defmacro :wat::bracket::each
  [locus <- :wat::WatAST
   items <- :wat::WatAST
   work-fn <- :wat::WatAST
   & kwpairs <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::= (:wat::core::length kwpairs) 0)
    
    (:wat::core::let
      [g1-sym   (:wat::core::fresh-symbol "g")
       pid1-sym (:wat::core::fresh-symbol "pid")
       g2-sym   (:wat::core::fresh-symbol "g")
       pid2-sym (:wat::core::fresh-symbol "pid")]
      `(:wat::bracket::each-worker ~locus ~items
         (:wat::bracket::const-worker-init ~work-fn)
         nil
         (:wat::core::fn [~g1-sym <- :wat::core::nil ~pid1-sym <- :wat::core::i64] -> :wat::core::nil nil)
         (:wat::core::fn [~g2-sym <- :wat::core::nil ~pid2-sym <- :wat::core::i64] -> :wat::core::nil nil)
         (:wat::core::Vector :wat::core::nil)))
    (:wat::core::let
      [work-fn-name  (:wat::core::ast-name work-fn)
       base-str      (:wat::core::string::subs work-fn-name 1 (:wat::core::string::length work-fn-name))
       checker-kw    (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::kwargs-check")))
       grant-fn-kw   (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::grant-worker")))
       revoke-fn-kw  (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::revoke-worker")))
       coords-ty-kw  (:wat::core::keyword-node
                        (:wat::core::string::concat ":" (:wat::core::string::concat base-str "::Coords")))
       ;; 293.W.2f — process runner door (twin of map).
       locus-head    (:wat::core::if (:wat::core::= (:wat::core::ast-kind locus) "list")
                        (:wat::core::let [lch (:wat::core::ast->children locus)]
                          (:wat::core::if (:wat::core::empty? lch) "" (:wat::core::ast-name (:wat::core::first lch))))
                        "")
       locus-inner   (:wat::core::if (:wat::core::= locus-head ":wat::spawn::with-label")
                        (:wat::core::let [lch (:wat::core::ast->children locus)]
                          (:wat::core::if (:wat::core::empty? (:wat::core::rest lch))
                            ""
                            (:wat::core::let [inner (:wat::core::first (:wat::core::rest lch))]
                              (:wat::core::if (:wat::core::= (:wat::core::ast-kind inner) "list")
                                (:wat::core::let [ich (:wat::core::ast->children inner)]
                                  (:wat::core::if (:wat::core::empty? ich) "" (:wat::core::ast-name (:wat::core::first ich))))
                                ""))))
                        locus-head)
       process-door? (:wat::core::string::starts-with?
                       (:wat::core::if (:wat::core::= locus-head ":wat::spawn::with-label") locus-inner locus-head)
                       ":wat::spawn::process")
       wire-pairs    (:wat::core::if process-door?
                        (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                           i   <- :wat::core::i64]
                            -> (:wat::core::Vector :- [:wat::WatAST])
                            (:wat::core::let
                              [item (:wat::core::Option/expect
                                      (:wat::core::get kwpairs i)
                                      "bracket/each: kwpair")]
                              (:wat::core::if (:wat::core::= (:wat::core::i64::mod i 2) 1)
                                (:wat::core::conj acc `(:wat::kernel::require-wire-address ~item))
                                (:wat::core::conj acc item))))
                          (:wat::core::Vector :wat::WatAST)
                          (:wat::core::range 0 (:wat::core::length kwpairs)))
                        kwpairs)
       checker-call  `(~checker-kw ~@wire-pairs)
       pair-sym      (:wat::core::fresh-symbol "pair")
       coords-sym    (:wat::core::fresh-symbol "coords")
       handles-sym   (:wat::core::fresh-symbol "handles")]
      `(:wat::core::let
         [~pair-sym    ~checker-call
          ~coords-sym  (:wat::core::first ~pair-sym)
          ~handles-sym (:wat::core::second ~pair-sym)]
         (:wat::bracket::each-worker ~locus ~items
           (:wat::bracket::const-worker-init ~work-fn)
           ~handles-sym ~grant-fn-kw ~revoke-fn-kw
           (:wat::core::Vector ~coords-ty-kw ~coords-sym))))))
