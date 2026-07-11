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
;; Exit discipline: recv' raises (EvalBreak) when the parent's Thread' is
;; dropped → the runner's recursion is broken by the raise → it exits cleanly.
;; No explicit termination condition is needed; the channel drain IS the signal.
;;
;; Loads AFTER wat/spawn.wat (uses :wat::kernel::Peer', recv', send').
;;
;; ── Rendezvous convention ───────────────────────────────────────────────────
;;
;; `:user::` is the RENDEZVOUS NAMESPACE — the known-location coordinates where
;; a program exposes what a substrate consumer looks up.  Not private/internal
;; space; a rendezvous space.  `:user::main` is wat-program's coordinate (the
;; kernel-required entry, `[] -> :nil`).  Bracket installs a second one:
;; `:user::bracket::work-fn` — the work function a process-pool child's
;; baked runner (`:wat::bracket::process-runner<I,O>` below) applies.  The
;; runner itself is baked/reserved (never shipped); the child's user.program
;; only ever ships the user's own work-fn, reified at this coordinate, plus a
;; generated `:user::main` that passes the coordinate's value into the runner.

(:wat::core::defn :wat::bracket::runner-loop<I,O>
  [self    <- :wat::kernel::ThreadSelfPeer'<O,I>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::let [item (:wat::kernel::recv' self)
                    _    (:wat::kernel::send' self (work-fn item))]
    (:wat::bracket::runner-loop self work-fn)))

;; PoolMsg<D,I> (the universal pool wire message) is defined in wat/spawn.wat — it
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
;; no stdlib -> user.program forward reference; the process arm's spawn-runner
;; ships only the work-fn (at the :user::bracket::work-fn rendezvous
;; coordinate) and a generated :user::main that passes it in here.
(:wat::core::defn :wat::bracket::process-runner<D,I,O>
  [self    <- :wat::kernel::Peer'<(wat::core::i64,O),wat::bracket::PoolMsg<D,I>>
   work-fn <- :wat::core::Fn(I)->O]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv' self) -> :wat::core::nil
    ((:wat::bracket::PoolMsg::Work pair)
      (:wat::core::let
        [out (:wat::core::Tuple (:wat::core::first pair) (work-fn (:wat::core::second pair)))
         _   (:wat::kernel::send' self out)]
        (:wat::bracket::process-runner self work-fn)))
    ;; A non-dialing pool never sends :Setup (dials empty); the arm is total by
    ;; construction — ignore + recurse (D stays phantom for this runner).
    ((:wat::bracket::PoolMsg::Setup _deps)
      (:wat::bracket::process-runner self work-fn))))

;; ── process-dial-runner — the BAKED dialing process-pool runner (arc 170 M1) ──
;;
;; The 2-param cousin of process-runner: a granted worker that DIALS a granted
;; service and HOLDS the typed peer across items. Generic over <S,R,I,O>: the
;; dialed service's channel is S/R, so the held context is `Peer'<S,R>` and the
;; Setup payload is `Address'<S,R>` (connect' : Address'<S,R> → Peer'<S,R>). Threads
;; `ctx` = `(Option Peer'<S,R>)`, None until the first Setup:
;;   :Setup(deps) → dial-and-HOLD: recurse with `(Some (connect' deps))`. The pid was
;;                  already granted (grant-boot, map-worker) so connect' is admitted.
;;   :Work(pair)  → run the held peer through the 2-param work-fn, send the indexed out.
;; This is the defservice :init/:ephemeral pattern lifted onto the bracket, and the
;; exact shape scratchpad/probe-m1-worker-setup.wat proved GREEN.
(:wat::core::defn :wat::bracket::process-dial-runner<S,R,I,O>
  [self    <- :wat::kernel::Peer'<(wat::core::i64,O),wat::bracket::PoolMsg<wat::kernel::Address'<S,R>,I>>
   work-fn <- :wat::core::Fn(wat::kernel::Peer'<S,R>,I)->O
   ctx     <- (:wat::core::Option :wat::kernel::Peer'<S,R>)]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv' self) -> :wat::core::nil
    ((:wat::bracket::PoolMsg::Setup deps)
      (:wat::bracket::process-dial-runner self work-fn
        (:wat::core::Some (:wat::kernel::connect' deps))))
    ((:wat::bracket::PoolMsg::Work pair)
      (:wat::core::let
        [c   (:wat::core::Option/expect ctx "bracket process-dial-runner: Work before Setup")
         out (:wat::core::Tuple (:wat::core::first pair) (work-fn c (:wat::core::second pair)))
         _   (:wat::kernel::send' self out)]
        (:wat::bracket::process-dial-runner self work-fn ctx)))))

;; ── spawn-runner — the per-tier runner spawn, lifted onto the :Locus surface ──
;;
;; The bracket coordinator (map-worker) is now loci-agnostic: it holds an abstract
;; :wat::spawn::Locus and calls (spawn-runner locus work-fn) once per pool tier.
;; The RAW work fn Fn(I)->O is passed — NOT an index-wrapping closure. Each tier
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
;; Both return :wat::kernel::Peer'<(i64,I),(i64,O)> so collect-loop drains a
;; uniform Vector<Peer'<…>> (select' accepts Peer' as of S3a).

;; Arc 170 M1-pool — the thread self-peer now recv's PoolMsg<Address',(i64,I)> (the
;; universal pool wire), so runner-loop stays the general recv→work-fn→send server
;; (untouched — its two direct tests send raw items) while the index-wrapper UNWRAPS
;; PoolMsg::Work here. A thread pool never dials (dials empty ⇒ no :Setup ever crosses),
;; so the :Setup arm is unreachable-by-construction — it raises rather than fabricate a
;; result. `work-fn` is the raw 1-param Fn(I)->O (thread applies it in-memory).
(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus
  (spawn-runner [self work-fn]
    (:wat::kernel::spawn-program' self
      (:wat::core::fn [sp <- :wat::kernel::ThreadSelfPeer'<(wat::core::i64,O),wat::bracket::PoolMsg<wat::kernel::Address',I>>] -> :wat::core::nil
        (:wat::bracket::runner-loop sp
          (:wat::core::fn [m <- :wat::bracket::PoolMsg<wat::kernel::Address',I>] -> :(wat::core::i64,O)
            (:wat::core::match m -> :(wat::core::i64,O)
              ((:wat::bracket::PoolMsg::Work pair)
                (:wat::core::Tuple (:wat::core::first pair) (work-fn (:wat::core::second pair))))
              ((:wat::bracket::PoolMsg::Setup _deps)
                (:wat::kernel::assertion-failed!
                  "bracket thread runner: unexpected PoolMsg::Setup (thread pools never dial)"
                  :wat::core::None :wat::core::None)))))))))

;; The PROCESS arm (not-shared) — bakes the runner, ships only the user's code
;; (259 S3c; supersedes the S3b shipped-runner shape).
;;
;; The runner is BAKED (`:wat::bracket::process-runner<I,O>` above) — nothing
;; reserved is shipped, so the `ReservedPrefix` problem S3b fought (an
;; un-squattable shipped name has nowhere safe to live in `:wat::`) is simply
;; gone.  We ship only: the user's work-fn, reified at the rendezvous
;; coordinate `:user::bracket::work-fn` (fn-forms), plus a generated
;; `:user::main` that calls the baked runner, passing that coordinate's VALUE
;; in (the runner is baked, so `:user::main` passes the value — it cannot look
;; the coordinate up from stdlib; that would be a stdlib -> user.program
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
;; through the POST-arc-251 canonical form: `wat.kernel/Peer'` (Symbol, dot/slash,
;; NO leading colon) rather than the surface literal `:wat::kernel::Peer'` a plain
;; fn's own (unmangled) param-type keyword carries. The two conventions are
;; interchangeable AS TEXT (both resolve to the same registry key; the wat reader
;; accepts either "." or "::" as a namespace separator and "/" as the terminal
;; separator) — this just re-punctuates one into the other so the compound
;; angle-bracket keyword strings built below (`wat::kernel::Address'<S,R>`, …)
;; stay in the ONE convention every other string in this AST-walk already uses.
;; "wat.kernel/Peer'" -> "wat" "kernel/Peer'" (split ".") -> "wat::kernel/Peer'"
;;                     -> "wat::kernel" "Peer'" (split "/") -> "wat::kernel::Peer'"
(:wat::core::defn :wat::bracket::dotpath->colonpath [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::join "::"
    (:wat::core::string::split
      (:wat::core::string::join "::" (:wat::core::string::split s "."))
      "/")))

;; Arc 170 M1-pool — the AST-walk now also distinguishes the DIAL work-fn. A
;; non-dial work-fn is 1-param `Fn(I)->O` (argspec = 3 AST children: `n <- :I`);
;; a dial work-fn is 2-param `Fn(Peer'<S,R>,I)->O` (6 children: `c <- :Peer'<S,R>
;; n <- :I`). We branch on that count:
;;   NON-DIAL → bake `process-runner`, self-peer recvs `PoolMsg<Address',I>` (D phantom).
;;   DIAL     → bake `process-dial-runner` (threads the held peer + starts at None),
;;              self-peer recvs the CONCRETE `PoolMsg<Address'<S,R>,I>` — S,R lifted off
;;              the 1st param `Peer'<S,R>` by a `Peer'`→`Address'` head-swap (split/join),
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
    (:wat::kernel::spawn-program' self (:wat::bracket::process-work-forms work-fn))))

(:wat::core::defclause :wat::bracket::process-work-forms
  ;; ── KWARGS branch (arc 170 C1 ground case N=1, generalized to N by C2 Strike 1) ──
  ;; work-fn is a BASE NAME keyword naming a kwargs `defn`'s companion (e.g. :probe::enrich).
  ;; Ship `<base>$impl` BY NAME (the fn-forms keyword seam, c8e3c7ff) — its OWN 2-param
  ;; signature is [item <- :I  kwargs <- :<base>::Kwargs] (item FIRST, unlike the raw
  ;; dial shape where the peer is first). Read the ::Kwargs struct's N fields (field-order
  ;; preserved by field-names-of/field-types-of, Strike B) to recover each field's Peer'<S,R>
  ;; — NEVER by reflecting the work-fn value itself. Then synthesize: (1) an N-ARY ADAPTER fn
  ;; (Peer'<S1,R1>,…,Peer'<Sn,Rn>,I)->O that assembles the ::Kwargs struct positionally (field
  ;; order) from the N held dial peers + calls the shipped $impl, and (2) an N-DIAL RUNNER
  ;; emitted as source (the baked single-peer `process-dial-runner` can't carry a variadic
  ;; type-param list — a Tuple<Address'<S1,R1>,…> carrier is concretely-typed PER CALL, so the
  ;; runner recv-ing it must be too; shape proven at
  ;; wat-scripts/probes/arc-170/w3-n-dial-runner.wat). N=1 is not special-cased — it is the
  ;; ground case of the same fold (a 1-element Tuple carrier/ctx).
  ([work-fn <- :wat::core::keyword] -> :wat::core::Vector<wat::WatAST>
    (:wat::core::let
      [base-str      (:wat::core::keyword/to-string work-fn)
       impl-kw       (:wat::core::keyword/from-string (:wat::core::string::concat base-str "$impl"))
       kwargs-ty-str (:wat::core::string::concat base-str "::Kwargs")
       kwargs-ty     (:wat::core::keyword/from-string kwargs-ty-str)
       work-name     (:wat::core::keyword/from-string "user::bracket::work-fn")
       forms         (:wat::kernel::fn-forms impl-kw work-name)
       nforms        (:wat::core::length forms)
       ;; The $impl fn-def-node — the SECOND-TO-LAST shipped form (fn-forms, given a
       ;; KEYWORD naming an ALREADY-REGISTERED fn, ships its own canonical `defn` decl
       ;; verbatim + a trailing `(def work-name <the-name>)` rebind; unlike the Fn-VALUE
       ;; path, which inlines the fn body directly into ONE trailing `(def work-name (fn …))`
       ;; — measured via scratchpad/probe-c1-kwargs-impl-astname.wat).
       def-node      (:wat::core::Option/expect (:wat::core::get forms (:wat::core::i64::- nforms 2))
                       "process-work-forms(kwargs): fn-forms produced no $impl define")
       dn-ch         (:wat::core::ast->children def-node)
       argspec       (:wat::core::Option/expect (:wat::core::get dn-ch 2) "process-work-forms(kwargs): no argspec")
       arg-ch        (:wat::core::ast->children argspec)
       ;; item = the $impl's FIRST param's type (index 2 of the flat [name <- ty …] triple
       ;; list) — NOT last, unlike the raw dial shape (kwargs puts item before the bundle).
       item-ty       (:wat::core::Option/expect (:wat::core::get arg-ch 2) "process-work-forms(kwargs): no item type")
       ret-ty        (:wat::core::Option/expect (:wat::core::get dn-ch 4) "process-work-forms(kwargs): no ret type")
       item-nm       (:wat::core::ast-name item-ty)
       ret-nm        (:wat::core::ast-name ret-ty)
       item-t        (:wat::core::string::subs item-nm 1 (:wat::core::string::length item-nm))
       ret-t         (:wat::core::string::subs ret-nm 1 (:wat::core::string::length ret-nm))
       ;; ── arc 170 C2 Strike 1 (record redirect): the ::Kwargs fields, reconciled BY NAME ──
       ;; The coords carrier D is the `<base>::Coords` RECORD (minted at the kwargs-defn site,
       ;; wat/core.wat) — addressed by field NAME, so N has NO positional-accessor cap and DATA
       ;; fields fall out for free. The runner recv's ONE `::Coords` record as the Setup payload,
       ;; reconciles it → `::Kwargs` by field name (Peer' field → connect' the Address'; data field
       ;; → copy the value through, routed off `field-types-of`), holds the assembled `::Kwargs`,
       ;; and invokes `$impl` per Work item. `fnames`/`ftypes` are field-ordered + positionally
       ;; aligned (Strike B), so a single fold over 0..n builds the ordered ctor args.
       fnames        (:wat::runtime::field-names-of kwargs-ty)
       ftypes        (:wat::runtime::field-types-of kwargs-ty)
       n             (:wat::core::length ftypes)
       _n-check      (:wat::core::if (:wat::core::= (:wat::core::length fnames) n)
                       -> :wat::core::nil nil
                       (:wat::kernel::assertion-failed!
                         "bracket process-work-forms: field-names-of/field-types-of length mismatch"
                         :wat::core::None :wat::core::None))
       coords-ty-str (:wat::core::string::concat base-str "::Coords")
       sp-out-str    (:wat::core::string::concat "(wat::core::i64," (:wat::core::string::concat ret-t ")"))
       sp-out        (:wat::core::keyword-node (:wat::core::string::concat ":" sp-out-str))
       ;; sp-in D = the ::Coords RECORD (a plain type path), NOT a Tuple: PoolMsg<<base>::Coords,I>.
       sp-in-str     (:wat::core::string::concat "wat::bracket::PoolMsg<"
                       (:wat::core::string::concat coords-ty-str
                         (:wat::core::string::concat "," (:wat::core::string::concat item-t ">"))))
       sp-in         (:wat::core::keyword-node (:wat::core::string::concat ":" sp-in-str))
       runner-self-kw (:wat::core::keyword-node
                        (:wat::core::string::concat ":wat::kernel::Peer'<"
                          (:wat::core::string::concat sp-out-str
                            (:wat::core::string::concat "," (:wat::core::string::concat sp-in-str ">")))))
       ;; ctx holds the assembled ::Kwargs (the N-heterogeneous dialed-peer bundle), None until Setup.
       ctx-ty-kw     (:wat::core::keyword-node
                       (:wat::core::string::concat ":wat::core::Option<" (:wat::core::string::concat kwargs-ty-str ">")))
       ret-kw        (:wat::core::keyword-node (:wat::core::string::concat ":" ret-t))
       kwargs-kw     (:wat::core::keyword-node (:wat::core::string::concat ":" kwargs-ty-str))
       ;; kwargs-ctor-args: one form per ::Kwargs field, DECLARED order, each read off the ::Coords
       ;; record BY NAME (`(:<base>::Coords/<field> deps)`). A Peer'-typed field (its ::Kwargs type
       ;; is a Peer'<S,R> — an `ast-kind` "list" whose head names Peer') gets `connect'`ed
       ;; (Address'→Peer'); a data field is copied through verbatim. `deps` is the Setup binder
       ;; symbol (literal in the runner quasiquote below — the same across-quasiquote literal-symbol
       ;; reference the C1 dial-runner already used, proven to survive the ship-as-source round-trip).
       kwargs-ctor-args
       (:wat::core::foldl
         (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST> i <- :wat::core::i64] -> :wat::core::Vector<wat::WatAST>
           (:wat::core::let
             [fname-str   (:wat::core::keyword/to-string (:wat::core::Option/expect (:wat::core::get fnames i) "process-work-forms(kwargs): fnames index"))
              accessor-kw (:wat::core::keyword-node
                            (:wat::core::string::concat ":"
                              (:wat::core::string::concat coords-ty-str
                                (:wat::core::string::concat "/" fname-str))))
              ft          (:wat::core::Option/expect (:wat::core::get ftypes i) "process-work-forms(kwargs): ftypes index")
              is-peer     (:wat::core::if (:wat::core::= (:wat::core::ast-kind ft) "list")
                            (:wat::core::string::contains?
                              (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children ft))) "Peer'")
                            false)
              form        (:wat::core::if is-peer
                            `(:wat::kernel::connect' (~accessor-kw deps))
                            `(~accessor-kw deps))]
             (:wat::core::conj acc form)))
         (:wat::core::Vector :wat::WatAST)
         (:wat::core::range 0 n))
       ;; N-DIAL RUNNER — emitted as source. recv PoolMsg<::Coords,I>; Setup deps (a ::Coords record)
       ;; → reconcile-by-name into the ::Kwargs bundle (connect' the Peer' fields, copy the data
       ;; fields), HOLD it; Work pair → invoke `$impl` (via the `:user::bracket::work-fn` keyword,
       ;; arc-009-lifted through `apply`'s head — the C1-proven invocation seam) with the item + the
       ;; held ::Kwargs bundle, send the indexed result, recurse. No separate adapter fn: the
       ;; reconciliation IS the assembly, done once at Setup.
       runner-def
       `(:wat::core::defn :user::bracket::dial-runner
          [self <- ~runner-self-kw
           ctx  <- ~ctx-ty-kw]
          -> :wat::core::nil
          (:wat::core::match (:wat::kernel::recv' self) -> :wat::core::nil
            ((:wat::bracket::PoolMsg::Setup deps)
              (:user::bracket::dial-runner self
                (:wat::core::Some (~kwargs-kw ~@kwargs-ctor-args))))
            ((:wat::bracket::PoolMsg::Work pair)
              (:wat::core::let
                [k   (:wat::core::Option/expect ctx "dial-runner: Work before Setup")
                 out (:wat::core::Tuple (:wat::core::first pair)
                       (:wat::core::apply -> ~ret-kw :user::bracket::work-fn (:wat::core::second pair) [k]))
                 _   (:wat::kernel::send' self out)]
                (:user::bracket::dial-runner self ctx)))))
       main-def
       `(:wat::core::defn :user::main [] -> :wat::core::nil
          (:user::bracket::dial-runner
            (:wat::program::self-peer ~sp-out ~sp-in)
            :wat::core::None))]
      (:wat::core::concat forms (:wat::core::Vector :wat::WatAST runner-def main-def))))
  ;; ── existing Fn branch (arc 170 M1-pool, arity 3/6 dispatch) — UNCHANGED logic,
  ;; only the tail (spawn-program' call -> plain forms-vector return) is refactored so
  ;; both clauses share the one call site above.
  ([work-fn <- :W] -> :wat::core::Vector<wat::WatAST>
    (:wat::core::let
      [work-name (:wat::core::keyword/from-string "user::bracket::work-fn")
       forms     (:wat::kernel::fn-forms work-fn work-name)
       ;; ── derive the concrete arg/return type keywords off the reified work-fn ──
       def-node  (:wat::core::Option/expect (:wat::core::last forms) "spawn-runner: fn-forms produced no define")
       fn-form   (:wat::core::first (:wat::core::drop (:wat::core::ast->children def-node) 2))
       fn-ch     (:wat::core::ast->children fn-form)
       argspec   (:wat::core::first (:wat::core::drop fn-ch 1))
       arg-ch    (:wat::core::ast->children argspec)
       arity     (:wat::core::length arg-ch)   ;; 3 = 1-param (non-dial); 6 = 2-param (dial)
       ;; item type I = the LAST param's type (both arities); O = the return type.
       arg-ty    (:wat::core::Option/expect (:wat::core::last arg-ch) "spawn-runner: work-fn has no arg type")
       ret-ty    (:wat::core::first (:wat::core::drop fn-ch 3))
       ;; ast-name → ":wat::core::i64"; strip the leading ':' for the type bodies.
       arg-nm    (:wat::core::ast-name arg-ty)
       ret-nm    (:wat::core::ast-name ret-ty)
       arg-t     (:wat::core::string::subs arg-nm 1 (:wat::core::string::length arg-nm))
       ret-t     (:wat::core::string::subs ret-nm 1 (:wat::core::string::length ret-nm))
       ;; ── self-peer SEND type = (i64,O) (output tuple), both arities ──
       sp-out    (:wat::core::keyword-node
                   (:wat::core::string::concat ":(wat::core::i64,"
                     (:wat::core::string::concat ret-t ")")))
       ;; ── main-def — dispatch on arity ──
       main-def
       (:wat::core::if (:wat::core::= arity 6)
         ;; DIAL: derive Address'<S,R> off the 1st param Peer'<S,R>; recv PoolMsg<Address'<S,R>,I>.
         (:wat::core::let
           [c-ty   (:wat::core::first (:wat::core::drop arg-ch 2))          ;; 1st param's TYPE node
            c-nm   (:wat::core::ast-name c-ty)                              ;; ":wat::kernel::Peer'<S,R>"
            addr   (:wat::core::string::join "Address'" (:wat::core::string::split c-nm "Peer'"))  ;; ":wat::kernel::Address'<S,R>"
            ;; strip the leading ':' (ast-name keywords always carry one) — inlined via `subs`,
            ;; not `string::strip-leading-colon` (that helper loads in string.wat, AFTER this file).
            addr-b (:wat::core::string::subs addr 1 (:wat::core::string::length addr))
            sp-in  (:wat::core::keyword-node
                     (:wat::core::string::concat ":wat::bracket::PoolMsg<"
                       (:wat::core::string::concat addr-b
                         (:wat::core::string::concat "," (:wat::core::string::concat arg-t ">")))))]
           `(:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::bracket::process-dial-runner
                (:wat::program::self-peer ~sp-out ~sp-in)
                :user::bracket::work-fn
                :wat::core::None)))
         ;; NON-DIAL: recv PoolMsg<Address',I> (D phantom — no Setup ever sent).
         (:wat::core::let
           [sp-in  (:wat::core::keyword-node
                     (:wat::core::string::concat ":wat::bracket::PoolMsg<wat::kernel::Address',"
                       (:wat::core::string::concat arg-t ">")))]
           `(:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::bracket::process-runner
                (:wat::program::self-peer ~sp-out ~sp-in)
                :user::bracket::work-fn))))]
      (:wat::core::concat forms (:wat::core::Vector :wat::WatAST main-def)))))

;; ── collect-loop — tail-recursive collector; drains M results from N runners ──
;;
;; State: peers (the live Thread' vector), items (the full input vector),
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
;; select' now returns ServiceEvent<I,O> (Stone 259 Lost-locus).  :Message is
;; the normal case.  :Closed/:Lost are honest arms — a bracket runner should
;; never disconnect or crash in normal operation; if it does, raise via
;; assertion-failed! so the failure is visible rather than silently swallowed.

;; Arc 170 C2 Strike 1c — generalized `Address'` (bare) to `D`. Purely a WIDENING of the
;; declared type (this fn's own logic never touches the Setup/D payload — it only ever
;; handles Work/select' events on the (i64,O) channel), so every existing bare-Address'
;; caller (map-worker) still infers D=Address' unchanged. Needed so `:wat::bracket::uses'`
;; (below) can reuse this SAME collector over `Peer'<PoolMsg<D,I>,…>` peers where D is the
;; concretely-typed `<base>::Coords` RECORD (the field-ordered coords carrier — Address' +
;; data fields, one named record) rather than the erased bare `Address'` — `:wat::spawn::Locus`'s
;; `spawn-runner` surface method (wat/spawn.wat) is FIXED to bare Address' (out of Strike 1's
;; scope), so a Coords-carrying pool cannot round-trip through it; `uses'` spawns directly
;; instead (see below), and needs its peers' D to be the real `::Coords` type — hence this
;; generalization is a prerequisite for reuse, not a reinvention (COMPONENDO DELEO).
(:wat::core::defn :wat::bracket::collect-loop<D,I,O>
  [peers     <- :wat::core::Vector<wat::kernel::Peer'<wat::bracket::PoolMsg<D,I>,(wat::core::i64,O)>>
   items     <- :wat::core::Vector<I>
   pairs-acc <- :wat::core::Vector<(wat::core::i64,O)>
   cursor    <- :wat::core::i64
   collected <- :wat::core::i64
   m         <- :wat::core::i64]
  -> :wat::core::Vector<(wat::core::i64,O)>
  (:wat::core::if (:wat::core::= collected m)
    pairs-acc
    (:wat::core::let
      [event    (:wat::kernel::select' peers)]
      (:wat::core::match event
        -> :wat::core::Vector<(wat::core::i64,O)>
        ((:wat::spawn::ServiceEvent::Message peer-pos pair)
          (:wat::core::let
            [cursor'  (:wat::core::if (:wat::core::< cursor m)
                        (:wat::core::let [_ (:wat::kernel::send'
                                              (:wat::core::nth peers peer-pos)
                                              (:wat::bracket::PoolMsg::Work
                                                (:wat::core::Tuple cursor (:wat::core::nth items cursor))))]
                          (:wat::core::+ cursor 1))
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

;; ── map-worker — general pool engine (per-runner state via worker-init) ───────
;;
;; Each runner i is built from `(worker-init i)`: the OUTER call is per-runner
;; setup (once, when the runner is built — the place to allocate a resource
;; reused across that runner's items); the INNER result is the per-item work-fn.
;; `worker-id` is the runner index passed to `worker-init`.  The coordinator
;; (spawn+prime+collect+sort) lives here ONCE; `map` and `each` are thin wrappers.

;; Arc 170 M1-pool — `worker-init` returns a GENERIC W (the raw work-fn: 1-param
;; `Fn(I)->O` for thread/non-dial, 2-param `Fn(Peer'<S,R>,I)->O` for a dialing process
;; pool). The pool never applies it here — it hands it to `spawn-runner` (which reifies
;; or applies per tier). Adding W keeps this engine tier- AND arity-agnostic.
(:wat::core::defn :wat::bracket::map-worker<I,O,W>
  [locus       <- :wat::spawn::Locus
   items       <- :wat::core::Vector<I>
   worker-init <- :wat::core::Fn(wat::core::i64)->W]
  -> :wat::core::Vector<O>
  (:wat::core::let
    [m  (:wat::core::length items)
     rc (:wat::spawn::runner-count locus)
     n  (:wat::core::if (:wat::core::< rc m) rc m)
     ;; Arc 170 capability circuit, stone A — the ONE vector of Capability handles this locus
     ;; carries (collapsed from the former two-vector grants/dials split, stone 2). Empty for
     ;; thread/remote (the firm boundary); the process locus's :uses field otherwise. Read
     ;; ONCE; grant-boot below folds over it before each worker's first item (grant), then
     ;; again for the Setup dial (each handle's own `coordinate`-derived address);
     ;; revoke-shutdown folds over it after the drain. A foldl over an empty vector is a
     ;; no-op, so a plain (process) (no :uses) takes no grant/dial path — same as thread.
     uses (:wat::spawn::uses locus)
     ;; Arc 118.2a — `map` flipped LAZY; `peers` feeds `collect-loop` (Vector<Peer'<...>> param
     ;; — repeatedly `select'`-ed, must be eager) and later `sort-by`, so materialize here.
     peers (:wat::core::mapv
             (:wat::core::fn [i <- :wat::core::i64]
                 -> :wat::kernel::Peer'<wat::bracket::PoolMsg<wat::kernel::Address',I>,(wat::core::i64,O)>
               (:wat::core::let
                 [work-fn (worker-init i)                          ;; per-runner setup, once
                  p (:wat::spawn::Locus/spawn-runner locus work-fn)
                  ;; GRANT-BOOT: if the far end is a process (peer-pid → Some pid), grant that
                  ;; kernel-vouched pid to each Capability handle (ack'd request/reply) BEFORE
                  ;; the first item is sent — so the grant lands before the worker's work-fn
                  ;; dials. A thread peer (peer-pid → None) skips: the in-process handle IS the
                  ;; capability.
                  _ (:wat::core::match (:wat::kernel::peer-pid p) -> :wat::core::nil
                      ((:wat::core::Some pid)
                        (:wat::core::foldl
                          (:wat::core::fn [_acc <- :wat::core::nil  g <- :(wat::core::keyword,wat::capability::Capability)] -> :wat::core::nil
                            (:wat::capability::Capability/grant (:wat::core::second g) (:wat::core::Vector :wat::core::i64 pid)))
                          nil
                          uses))
                      (:wat::core::None nil))
                  ;; SETUP-DIAL: hand the worker each handle's `coordinate`-derived address as a
                  ;; PoolMsg::Setup — the worker connect's-and-holds the granted service (ocap
                  ;; over the wire). A foldl over an empty `uses` (thread/non-dial) is a no-op.
                  ;; Runs AFTER grant-boot (grant-then-dial) and BEFORE the first Work item so
                  ;; the peer is held first. The NAME half of each pair (`first`) is unused
                  ;; here — grant/dial are name-blind; the NAME only matters to the kwargs
                  ;; AST-walk's field reconciliation (process-work-forms below), which reads
                  ;; it off the `::Kwargs` type, not off this fold.
                  _ (:wat::core::foldl
                      (:wat::core::fn [_acc <- :wat::core::nil  g <- :(wat::core::keyword,wat::capability::Capability)] -> :wat::core::nil
                        (:wat::kernel::send' p (:wat::bracket::PoolMsg::Setup (:wat::capability::Capability/coordinate (:wat::core::second g)))))
                      nil
                      uses)
                  _ (:wat::kernel::send' p (:wat::bracket::PoolMsg::Work (:wat::core::Tuple i (:wat::core::nth items i))))]
                 p))
             (:wat::core::range 0 n))
     pairs  (:wat::bracket::collect-loop peers items
              (:wat::core::Vector :(wat::core::i64,O)) n 0 m)
     ;; REVOKE-SHUTDOWN: the drain is complete but the peers are still alive (still in scope,
     ;; still hold their Pidfd → peer-pid still Some). For each process peer, revoke its pid
     ;; from each Capability handle (ack'd) — the grant a worker held cannot outlive its
     ;; reaping. A thread peer (None) skips. Runs BEFORE the return so no grant escapes the
     ;; bracket.
     _revoke (:wat::core::foldl
               (:wat::core::fn [_acc <- :wat::core::nil
                                p    <- :wat::kernel::Peer'<wat::bracket::PoolMsg<wat::kernel::Address',I>,(wat::core::i64,O)>]
                 -> :wat::core::nil
                 (:wat::core::match (:wat::kernel::peer-pid p) -> :wat::core::nil
                   ((:wat::core::Some pid)
                     (:wat::core::foldl
                       (:wat::core::fn [_a <- :wat::core::nil  g <- :(wat::core::keyword,wat::capability::Capability)] -> :wat::core::nil
                         (:wat::capability::Capability/revoke (:wat::core::second g) (:wat::core::Vector :wat::core::i64 pid)))
                       nil
                       uses))
                   (:wat::core::None nil)))
               nil
               peers)
     sorted (:wat::core::sort-by
              (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :wat::core::i64
                (:wat::core::first pr))
              pairs)]
    ;; Arc 118.2a — `map` flipped LAZY; the function's declared return type is `Vector<O>`.
    (:wat::core::mapv
      (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :O
        (:wat::core::second pr))
      sorted)))

;; ── uses' — the N-service coords-carrying pool coordinator (arc 170 C2 Strike 1c) ──
;;
;; `map-worker`'s peers are pinned to `Peer'<PoolMsg<Address'(bare),I>,…>` by
;; `:wat::spawn::Locus`'s `spawn-runner` surface method (wat/spawn.wat:386-388) — a FIXED
;; declared return type, unchanged by this stone. A coords-carrying pool needs its Setup
;; payload to be the REAL, concretely-typed field-ordered `<base>::Coords` RECORD (Address' +
;; data fields, one named record — built by the evolved W2a checker, Strike 1a) — a type that
;; CANNOT round-trip through that fixed bare-Address' surface method (a record does not
;; `assignable` to a bare parametric-head Path; only `Address'<S,R>` itself widens to bare
;; `Address'` via the reflexive same-head `is_subtype` edge, check.rs:15367-15375). So `uses'`
;; spawns DIRECTLY via `spawn-program'` (bypassing `Locus/spawn-runner` for this one path)
;; instead of inventing a new locus-dispatch mechanism. It REUSES `collect-loop` (generalized
;; to `D` above) and DUPLICATES (not reinvents — the SAME shape, unavoidable since it can't
;; call through map-worker's own peers-mapv closure) map-worker's grant-boot/revoke-shutdown
;; folds verbatim — name-blind, unchanged (`:wat::bracket::map-worker` itself is untouched by
;; this stone). `coords` is the field-ordered `::Coords` record VALUE (built by the caller —
;; the hand-wired Strike-1 proof, or later the `bracket/uses` macro's checker-call result);
;; `work-fn` is the kwargs work-fn's BASE keyword (e.g. :probe::enrich) — `process-work-forms`
;; (Strike 1b) builds the child's forms ONCE (identical for every runner in the pool). `uses`
;; carries ONLY the SERVICE handles (data kwargs need no grant — they copy as EDN, not dial).
;; PROCESS ONLY (dialing a service needs the firm process boundary — the whole point of this
;; stone; a thread locus never needs to dial at all).
(:wat::core::defn :wat::bracket::uses'<D,I,O>
  [locus   <- :wat::spawn::ProcessOpts
   uses    <- :wat::core::Vector<(wat::core::keyword,wat::capability::Capability)>
   items   <- :wat::core::Vector<I>
   work-fn <- :wat::core::keyword
   coords  <- :D]
  -> :wat::core::Vector<O>
  (:wat::core::let
    [m      (:wat::core::length items)
     rc     (:wat::spawn::ProcessOpts/runner-count locus)
     n      (:wat::core::if (:wat::core::< rc m) rc m)
     forms  (:wat::bracket::process-work-forms work-fn)   ;; built ONCE — identical per runner
     peers  (:wat::core::mapv
              (:wat::core::fn [i <- :wat::core::i64]
                  -> :wat::kernel::Peer'<wat::bracket::PoolMsg<D,I>,(wat::core::i64,O)>
                (:wat::core::let
                  [p (:wat::kernel::spawn-program' locus forms)
                   ;; GRANT-BOOT — verbatim copy of map-worker's own fold (name-blind, UNCHANGED).
                   _ (:wat::core::match (:wat::kernel::peer-pid p) -> :wat::core::nil
                       ((:wat::core::Some pid)
                         (:wat::core::foldl
                           (:wat::core::fn [_acc <- :wat::core::nil  g <- :(wat::core::keyword,wat::capability::Capability)] -> :wat::core::nil
                             (:wat::capability::Capability/grant (:wat::core::second g) (:wat::core::Vector :wat::core::i64 pid)))
                           nil
                           uses))
                       (:wat::core::None nil))
                   ;; SETUP-DIAL — arc 170 C2 Strike 1c: ONE Setup(coords), not N per-handle
                   ;; Setups. `coords` is already the field-ordered, concretely-typed Tuple —
                   ;; no per-handle `Capability/coordinate` fold needed here at all.
                   _ (:wat::kernel::send' p (:wat::bracket::PoolMsg::Setup coords))
                   _ (:wat::kernel::send' p (:wat::bracket::PoolMsg::Work (:wat::core::Tuple i (:wat::core::nth items i))))]
                  p))
              (:wat::core::range 0 n))
     pairs   (:wat::bracket::collect-loop peers items
               (:wat::core::Vector :(wat::core::i64,O)) n 0 m)
     ;; REVOKE-SHUTDOWN — verbatim copy of map-worker's own fold (UNCHANGED).
     _revoke (:wat::core::foldl
               (:wat::core::fn [_acc <- :wat::core::nil
                                p    <- :wat::kernel::Peer'<wat::bracket::PoolMsg<D,I>,(wat::core::i64,O)>]
                 -> :wat::core::nil
                 (:wat::core::match (:wat::kernel::peer-pid p) -> :wat::core::nil
                   ((:wat::core::Some pid)
                     (:wat::core::foldl
                       (:wat::core::fn [_a <- :wat::core::nil  g <- :(wat::core::keyword,wat::capability::Capability)] -> :wat::core::nil
                         (:wat::capability::Capability/revoke (:wat::core::second g) (:wat::core::Vector :wat::core::i64 pid)))
                       nil
                       uses))
                   (:wat::core::None nil)))
               nil
               peers)
     sorted  (:wat::core::sort-by
               (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :wat::core::i64
                 (:wat::core::first pr))
               pairs)]
    (:wat::core::mapv
      (:wat::core::fn [pr <- :(wat::core::i64,O)] -> :O
        (:wat::core::second pr))
      sorted)))

;; ── map — thin wrapper over map-worker (Ruby's Parallel.map) ─────────────────
;;
;; Passes a constant `worker-init` that ignores the runner id and returns the
;; shared work-fn.  The coordinator (spawn+prime+collect+sort) lives in map-worker.

;; Arc 170 M1-pool — `work-fn` is a generic W: a 1-param `Fn(I)->O` for a plain pool,
;; a 2-param `Fn(Peer'<S,R>,I)->O` for a dialing process pool (`(process/uses …)`).
;; map-worker + spawn-runner route it per tier/arity; O is pinned by the result usage.
(:wat::core::defn :wat::bracket::map<I,O,W>
  [locus   <- :wat::spawn::Locus
   items   <- :wat::core::Vector<I>
   work-fn <- :W]
  -> :wat::core::Vector<O>
  (:wat::bracket::map-worker locus items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :W
      work-fn)))

;; ── each-worker — general side-effect pool (per-runner state via worker-init) ─
;;
;; `map-worker` that DISCARDS: run worker-init-derived per-item fns over every
;; item through the pool, then return nil.

(:wat::core::defn :wat::bracket::each-worker<I,O,W>
  [locus       <- :wat::spawn::Locus
   items       <- :wat::core::Vector<I>
   worker-init <- :wat::core::Fn(wat::core::i64)->W]
  -> :wat::core::nil
  (:wat::core::do (:wat::bracket::map-worker locus items worker-init) nil))

;; ── each — thin wrapper over each-worker (Ruby's Parallel.each) ──────────────
;;
;; Passes a constant `worker-init` that ignores the runner id.

(:wat::core::defn :wat::bracket::each<I,O,W>
  [locus   <- :wat::spawn::Locus
   items   <- :wat::core::Vector<I>
   work-fn <- :W]
  -> :wat::core::nil
  (:wat::bracket::each-worker locus items
    (:wat::core::fn [_worker-id <- :wat::core::i64] -> :W
      work-fn)))
