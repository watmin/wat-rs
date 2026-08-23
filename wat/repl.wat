;; wat-scripts/demos/repl/repl.wat — the wat REPL, correct-but-slow (arc 170).
;;
;; THE SHAPE. A REPL is a loop over ONE state — the user's definition set — and one verb:
;;
;;     read   →  a line of stdin, `read-string`d into a form
;;     eval   →  `(:wat::eval-with-defs! form defs)` — the form, in a world built from defs
;;     print  →  the outcome, as EDN (it already IS data; nothing formats it)
;;     loop   →  tail-invoke with defs GROWN if the line was a declaration, unchanged if not
;;
;; THE STATE, and it is a LOOP PARAMETER — not a defservice.
;;
;; ⚠ This paragraph used to describe a `:durable` / `:ephemeral` split, i.e. defservice clause
;; vocabulary, for a file that contains NO defservice: `:repl::turn`/`eval-and-loop`/`eval-form`
;; are three plain `defn`s and `defs` is a tail-call parameter. That wording is a leftover from
;; the earlier REPL demos, which WERE a spawned service you dialled — a design where the session
;; genuinely crossed callers. The architecture changed; the comment did not, and it cost an hour
;; of "is this a service or not" before anyone opened the file. Kept visible: a stale comment
;; reads as grounded precisely because it is specific.
;;
;;   defs — a `Vector<WatAST>`, threaded through the tail call. Forms are PURE by nature (a tree
;;          of keywords and literals holds no fd and no peer), so the definition set is data that
;;          ships and replays. It lives for the process and dies with it.
;;
;; The live `Environment` is NOT in this file at all — it is Rust-side, threaded by
;; `eval-with-defs!` (`run_constrained(ast, env, sym)` has always taken `env` separately from the
;; symbol table), which is why a bound service survives every re-freeze for free.
;;
;; WHY NOT A SERVICE. The builder's own test (arc 278 R27) is whether state crosses CALLERS: a
;; defservice exists so state can outlive and be reached by more than one of them — that is what
;; the address and the actor-as-mutex are for. `defs` never crosses one. A single loop owns it
;; from the first turn to EOF, and the stdio it reads and writes through
;; (`stdin-svc`/`stdout-svc`, wat/kernel/services/stdio.wat) ARE services — the I/O is
;; service-backed, the session is not. A defservice here would mint an address nobody dials and
;; a mutex with nothing to serialize.
;;
;; WHY IT IS SLOW ON PURPOSE. Every turn re-derives the entire world from `defs`. That is
;; the R1/R9 dual-impl discipline: this is the correct-but-slow ORACLE. Its correctness is
;; not argued, it is structural — the turn runs the ORDINARY program pipeline, so this REPL
;; is exactly as strongly typed as a compiled program. The fast incremental data plane gets
;; built later, behind a differential against this. Never delete the oracle.
;;
;; WHY THERE IS NO `declaration?` PREDICATE. There cannot be an honest one at this layer.
;; `defn` and `defrecord` fail eval with `unknown-function` — byte-identical to a TYPO —
;; because both are macros with no runtime verb to find (measured:
;; wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat). The substrate classifies
;; instead, on the POST-EXPANSION residue, and hands back a named outcome. A consequence
;; worth having: a user's OWN macro that expands to a `def` is classified correctly, and
;; this file never learns it exists.
;;
;;   run:  wat --repl
;;   then: type a form per line —
;;           (:wat::core::defn :usr::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* n 2))
;;           (:usr::double 21)
;;
;; EOF STOPS CLEANLY. `read-frame` hands EOF back as a VALUE (`ReadFrameOutcome::Eof`), so
;; Ctrl-D ends the session by returning, not by raising. This paragraph used to say the
;; opposite — that EOF raised a `LociDiedError/Panic` cascade and `readln` had no matchable
;; EOF — and it stayed here, false, after closures #2 and #4 gave the reader an outcome wall
;; and the frame vocabulary. The BODY was migrated; this header was not. Kept visible because
;; it is the exact defect class this arc hunts: a claim nothing contradicts, which reads as
;; grounded precisely because it names a limitation instead of a capability.

(:wat::core::defn :repl::eval-and-loop
  [defs <- (:wat::core::Vector :- [:wat::WatAST])
   text <- :wat::core::String]
  -> :wat::core::nil
  ;; The PARSE is a turn outcome too, not a precondition. The codemod's uniform arm for this
  ;; site was `assertion-failed!` — correct for a tool parsing a file it owns, fatal for a
  ;; prompt — so the REPL refines it, which is the entire reason `read-string` became total:
  ;; a caller that wants to survive bad input can now write that down.
  (:wat::core::match (:wat::core::read-string text)
    ((:wat::core::ReadOutcome::Malformed cause)
      (:wat::core::do
        (:wat::kernel::println cause)
        (:repl::turn defs)))
    ((:wat::core::ReadOutcome::Forms forms)
      (:repl::eval-form defs (:wat::core::first forms)))))

(:wat::core::defn :repl::eval-form
  [defs <- (:wat::core::Vector :- [:wat::WatAST])
   form <- :wat::WatAST]
  -> :wat::core::nil
  (:wat::core::let
    []
    (:wat::core::match (:wat::eval-with-defs! form defs)

      ;; A DECLARATION joined the world. Nothing to show — but the definition set grows,
      ;; and THIS is the only arm that grows it.
      ;; (a UNIT variant matches BARE — the inner parens are for tagged variants only)
      (:wat::eval::FormOutcome::Declared
        (:repl::turn (:wat::core::conj defs form)))

      ;; An EXPRESSION produced a value. The world is unchanged.
      ((:wat::eval::FormOutcome::Evaluated v)
        (:wat::core::do
          (:wat::kernel::println v)
          (:repl::turn defs)))

      ;; It did not type-check in this world. Nothing ran; the session is untouched.
      ;; `cause` is a navigable error TREE, not prose — `:causes` down to a real `:span`.
      ((:wat::eval::FormOutcome::CheckFailed cause)
        (:wat::core::do
          (:wat::kernel::println cause)
          (:repl::turn defs)))

      ;; It type-checked, ran, and unwound. Also non-fatal: one bad line does not end a
      ;; session, which is the whole reason a REPL's failures must be VALUES.
      ((:wat::eval::FormOutcome::Raised cause)
        (:wat::core::do
          (:wat::kernel::println cause)
          (:repl::turn defs))))))

;; The READ half. `read-frame` hands back the frame's RAW TEXT — a user types wat source,
;; not an EDN literal — and hands back EOF as a VALUE, so Ctrl-D returns cleanly instead of
;; raising a death cascade through the loop.
;;
;; A frame is not a line in general — the reader accumulates until the buffer forms a
;; complete EDN value — but for WAT source it is, and one form per line is the contract
;; here. MEASURED: the frame scanner continues only while the prefix is INCOMPLETE EDN and
;; terminates on MALFORMED, and wat source is never valid EDN (`:wat::core::defn` is the
;; "keyword begins with ::" case), so a half-typed form ends the frame at the newline.
;; A multi-line form therefore reaches `read-string` truncated and raises UnclosedParen.
;; That is a real limitation of this REPL, named rather than discovered by the next person.
(:wat::core::defn :repl::turn
  [defs <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::read-frame )
    ((:wat::kernel::ReadFrameOutcome::Frame text)
      (:repl::eval-and-loop defs text))
    ;; the honest stop — and this time the comment is true
    (:wat::kernel::ReadFrameOutcome::Eof nil)
    ;; Arc 170 — a process-wide stop (SIGTERM/SIGINT) arrived while parked
    ;; here waiting on the next line. Same clean-exit shape as ::Eof: this
    ;; loop's only job is to stop reading, and both outcomes agree on that.
    (:wat::kernel::ReadFrameOutcome::Stopped nil)))

;; NO `:user::main` HERE — deliberately. This is a stdlib MODULE, not a program: it exposes
;; `:repl::turn` and nothing else runs on load. The entry point lives in the CLI's `--repl`
;; mode, which is a one-form shim calling `:repl::turn` with an empty definition set. Putting
;; a `:user::main` in a stdlib file would hand one to EVERY wat program and collide with the
;; author's own; keeping the shim in the CLI is also just where an entry point belongs.
;;
;; The consequence worth having: the REPL is a LIBRARY. Any program can `(:repl::turn defs)`
;; to embed a loop of its own, seeded with whatever definitions it likes.
