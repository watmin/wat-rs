# DESIGN — the eprintln annihilation: move the recv'-wall surfacing OFF the death channel

> **THE FINDING (builder, 2026-07-22): "we have some heretics in our code we didn't know about — eprintln abuse."**
> The recv' OUTCOME WALL (R53) surfaces `::Lost`/`::Closed` via `eprintln` — but **`eprintln` is wat's PANIC**
> (`panic_any` → structured exit, *terminal*) **and** it needs the stdio service running. Using it as the recv'-read
> surfacing is a category error: it kills serve-loops (a client-triggerable DoS), it MASKS the failure in no-stdio
> contexts (`ServiceNotRunning`), and it lies on `::Closed` (a clean EOF is not a death). **R53's no-hidden-failures
> law, bitten by R53's own wall's mechanism** — an echo of R41→R53, one layer deeper.

## The channel discipline (R51 `TYPO TANGO` — the typed-effect channels)
Three channels, three jobs — do not confuse them:
- **`eprintln` / stderr = the DEATH channel.** Strict-EDN out **+ TERMINATE** (the dying declaration). RESERVED for a
  genuine top-level crash, in a context that has stdio and *intends to die*.
- **`telemetry` = the LOG channel.** Observe / record / continue. Where "surface the cause but keep running" belongs.
- **`RecvOutcome::Lost[cause]` = the failure AS DATA.** Matchable; the reader *decides* what to do with it.

**The abuse = using the DEATH channel where the LOG channel (telemetry), the DATA (the matchable value), or a
catchable RAISE (`assertion-failed!`) is what belongs.** eprintln is our panic; we scattered it across every recv'
reader as if it were a log line.

## The heretics — mapped (2026-07-22 reconnaissance)
- **Class A — the recv'-wall surfacing: ~194 arms.** Every `::Lost`/`::Closed` arm that `eprintln`s — the stdlib
  wraps (`spawn.wat`/`service.wat`/`bracket.wat`, S1), the S3 fleet's test wraps + scratch (this session, copied from
  the brief's exemplar — the abuse was codified in that brief; owned). Three sins: **terminal** (DoS in serve-loops),
  **stdio-dependent** (masks in no-stdio → the 39-test weigh cascade), **`::Closed`-is-not-a-death**.
- **Class B — pre-existing channel-confusion: `examples/console-demo/wat/main.wat`.** eprintln used as `:warn`/`:error`
  LOG-routing to stderr, expecting to *continue* — but it terminates, so only the first event emits and the demo
  dies; the second `CircuitBreak` is dead code. The death channel misused for logging.
- **NOT heretics (~20 legitimate uses):** `probe_arc278_eprintln_terminal` (tests the terminal behavior),
  `wat_run_sandboxed` (tests the dying declaration), the eprintln test-helpers, `intrinsic-metadata` (die-with-reason
  script), `wat-tests/test.wat` (captures eprintln's stderr in a hermetic sandbox). These correctly *exercise* the
  death channel — leave them.

## THE CONTRACT DECISION (pinned)
The recv'-wall arms move off `eprintln`:
- **`::Lost cause` → `(:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :None :None)`** — a
  **catchable, structured, NO-STDIO** re-raise. The reader matched the value and *deliberately* chose to raise it
  (R53-sanctioned — a visible act, not a mask). Proven `--check`-green (the S3 death-path rider used exactly this).
  *(Refinement — see OPEN: whether a `Failure → Error` bridge lets `(raise! cause)` preserve the full structured
  Failure faithfully instead of `Failure/message`-only; `raise!` today takes `:wat::core::Error`, the cause is
  `:wat::kernel::Failure`, so the direct form does not type-check yet.)*
- **`::Closed`** — split by intent:
  - EXPECTED clean end (a STREAM-loop reader whose terminal is a clean EOF) → the **clean terminal value** (nil/done).
    Never surface a clean close.
  - UNEXPECTED mid-stream close → `(:wat::kernel::assertion-failed! "recv': <peer> closed unexpectedly" :None :None)`.
- **`eprintln` STAYS** the death channel for the ~20 legitimate uses (intended top-level death, in a stdio context).
- **Class B (console-demo)** → route `:warn`/`:error` through **telemetry** (the log channel). Deferred to when the
  telemetry surface is wired (item (c)); tracked here, not guessed.

## ═══ RULING (2026-07-22g, four-questions, builder-ratified "it has been reasoned") — the GENERATED CLIENT METHOD returns `RecvOutcome<Response>`; it does NOT raise ═══
The eprintln annihilation LANDED (codemod `eprintln-recv-arm-to-assertion-failed.wat`, 192 arms, dry-run-verified,
applied) and did its job: it UN-MASKED the true cascade (`ServiceNotRunning` masks → real roots). The weigh reads
**59** real failures in 3 buckets, and the master root is NOT eprintln — it is the recv'-wall's own flagged todo:

**The master root (breadcrumb-22c's named "Path B Rust op-call at runtime.rs:~5488, mirror service.wat:1174"):**
the GENERATED client-method dispatch does `__r ← (recv' peer)` then `(match __r ((S::Reply::Op resp) resp))` — but
the S1 wall made `recv'` return `RecvOutcome<S::Reply>`, so `__r` is a `RecvOutcome` and the bare-reply match finds
no arm → `PatternMatchFailed` "type wat::core::Enum". Path-B (`runtime.rs:5436-5468`) is NOT wrapped at all;
the defservice op-methods (`service.wat:1163-1196`) ARE wrapped but **raise** (`assertion-failed!`) on `Lost`/`Closed`
and return bare `resp-ty`.

**THE FORK (four-questions):** (a) generated method typed `-> Response`, raises on `Lost`/`Closed` — **fails
Obvious + Honest** (the signature hides the death; a raise unwinds past the reader = the exact R53/R41 masking,
relocated into codegen). (b) generated method returns **`RecvOutcome<Response>`**, every caller matches
`Message`/`Lost`/`Closed` — **passes all four** (every outcome in the type; verbosity is the shield, R52; no hidden
failure). **VERDICT: (b).** This CORRECTS the "(pinned) `::Lost cause → assertion-failed!`" line above — that pin is
right for an **author-written** recv' arm (a match-then-die the author *visibly* chooses; my codemod's 192
conversions are all such author sites and STAND), but WRONG for the **generated client method**, which must not bake
a raise behind a lying `-> Response` type.

**(b) ⟂ the arc-294 "client = reason-free 500" ruling — both hold.** They are different axes: (b) = HOW the failure
is surfaced (a matchable value, not a raise); 294 = WHAT the client learns (no internal cause; the full cause stays
on the owner's crash channel). So the client method returns `RecvOutcome<Response>` where the client-facing `Lost`
variant is **reason-free** (cause scrubbed); the owner still gets the full cause on its channel.

**THE STRIKE (codegen core → prove on per_op → fleet the call-site cascade → weigh → the ONE atomic commit):**
1. **The return type** — `types.rs` surface-method synthesis (`SurfaceMember::Method` `ret`, ~1806): `<Op>Response`
   → `RecvOutcome<<Op>Response>`; the check-side Nature::Peer method-call inference (`check.rs` ~15635) follows.
2. **Path-B body** (`runtime.rs:5436-5468`): wrap in the RecvOutcome match — `Message(reply)` → `Message(match reply
   ((S::Reply::Op resp) resp))`; `Lost` → `Lost(<reason-free>)`; `Closed` → `Closed`. Return the outcome; DELETE the
   stale "recv' surfaces Failed first" comment (it describes the pre-wall recv').
3. **defservice op-method body** (`service.wat:1163-1196`): return the outcome (Message-unwrap, Lost reason-free,
   Closed) instead of `assertion-failed!`; return type → `RecvOutcome<resp-ty>`.
4. **The call-site cascade** (~59 sites — `per_op`, `journal :init` (`wat/telemetry/journal.wat:83`), `sift`, `span`,
   `s2s`, `dead_child_speaks`, arc170/209/272…): each matches `RecvOutcome<Response>` — outer transport-outcome, inner
   response-domain — choosing per-site what `Lost`/`Closed` do (an author may `assertion-failed!` visibly, or handle).
   `dead_child_speaks` must now MATCH `_e` and choose its death visibly (today it discards `_e` and relied on the
   codegen raise — the (a) behavior).
5. **Bucket C** (arc209_c2 `Counter::Op` vs `counter::Op` type-name casing) — a SEPARATE check-time codegen bug,
   independent of this; its own small fix.

Orchestrator draws + proves the boss (codegen core → per_op green by own re-run); shadowdancers fleet the call-site
cascade; every kill weighed by own `--release` re-run. Folds into the ONE atomic commit (258 + S1 + S3 + eprintln + this).

## The disconfirming probe (the hardest boss — prove the core move first)
Flip the stdlib spawn readiness barrier (`spawn.wat:369/370/403/404`, the confirmed `sift_logs` ServiceNotRunning
culprit) `eprintln → assertion-failed!`, rebuild (bake it), re-run `sift_logs`. **PASS condition:** the
`ServiceNotRunning` MASK is gone — the test either passes, or reveals the *real* underlying failure it was masking
(no longer `eprintln: called before stdio services running`). **RESULT (2026-07-22, PROVEN):** flipped
`spawn.wat:369/370/403/404` `eprintln → assertion-failed!`, rebuilt (baked), re-ran `sift_logs`. The
`ServiceNotRunning` MASK is GONE — the test now fails with the **real** underlying bug it was hiding:
`#wat.runtime/PatternMatchFailed` at `wat/telemetry/journal.wat:83` (the generated `Store/ensure-schema` client-method
match not handling a reply variant). **The 39 whole-floor-weigh failures were not 39 eprintln bugs — they were real
failures the death-channel mask swallowed.** The core move is proven; the spawn.wat flip stays (correct); the other
~190 arms + the unmasked cascade (journal.wat:83 …) are the landing.

> **Why `assertion-failed!` not `raise!` (the mechanism, grounded):** the "native raise" is ONE mechanism —
> `panic_any(AssertionPayload)` → the uniform structured-exit / catch_unwind path — with FOUR sibling verbs:
> `panic!` (message) · `raise!` (a `:wat::core::Error` value) · `assertion-failed!` (message+actual+expected) ·
> `eprintln` (a value **+ emits to stderr**). `eprintln` is the ONLY face that touches stdio — the exact seam the abuse
> fell through; the other three are pure `panic_any`, no stdio. `(raise! cause)` is blocked only because `raise!` takes
> `:wat::core::Error` (message+location+**causes** chain — the *domain* error) and the `::Lost` cause is a
> `:wat::kernel::Failure` (message+location+**frames+actual+expected** — the *kernel crash* payload): they meet at
> message/location but `Failure` lacks `causes`, so it is not an `Error`. (`macro-error` is a SEPARATE channel —
> `EvalBreak::Diagnostic`/`MacroAbort`, a returned expand-time diagnostic, not `panic_any`.)

## Execution (when landed)
1. **A wat-fix codemod** over the `(:wat::kernel::eprintln <X>)`-inside-a-`RecvOutcome::{Lost,Closed}`-arm pattern
   (structural: eprintln whose enclosing match-arm head is `RecvOutcome::Lost`/`::Closed`) → `assertion-failed!`.
   Uniform across stdlib + the S3 test wraps + scratch. `::Lost cause → (assertion-failed! (Failure/message cause)
   :None :None)`; `::Closed <str> → (assertion-failed! <str> :None :None)`.
2. **The stream-loop `::Closed` exception** (a small hand-set — `bracket_runner_stream_of_messages`,
   `bracket_runner_large_stream`, `probe-m1-phantom-d`, `w3-n-dial-runner`): `::Closed` → the clean terminal, NOT
   `assertion-failed!` (a clean end must not raise). The whole-floor weigh names these (they fail on
   clean-close-raises); hand-fix.
3. **Whole-floor weigh** (`cargo nextest run --release`) — the semantic gate. With the mask gone, the weigh's
   remaining failures read TRUE (the real child-side / underlying failures the mask was hiding). Drive the cascade.
4. **console-demo (Class B)** → telemetry, when wired.
5. Update `wat-tests/core/*` / the north-star probes; retire the superseded `recv-outcome-vocabulary.wat` (its `Crashed
   → eprintln` shape is the original abuse, now annihilated).

## OPEN (refinements, decide at land time)
- **`Failure → Error` bridge for `raise!`** — to re-raise the *full* structured Failure (message + actual + expected +
  frames) faithfully instead of `Failure/message`-only. Cleaner; preserves `structured_peer_death`'s sentinels
  without the threaded-`actual`/`expected` hack. Needs either `Failure <: Error` or a small converter.
- **Serve-loop `::Lost` = telemetry-log + continue** (not raise) — the true no-DoS answer for a service reading a
  *client's* death. Blocked on the telemetry surface (item (c)); until then a serve-loop that raises on a client death
  is *honest* (surfaces, no mask) even if not yet *resilient*. Hold both.

## Status
**DRAWN.** The core move (`::Lost`/`::Closed` off `eprintln` → `assertion-failed!`, no-stdio) is proven `--check`-green
(the death-path wraps) and being proven runtime-green (the spawn-barrier probe → `sift_logs` mask clears). The codemod
+ the stream-loop fix + the whole-floor weigh are the landing. Folds into the same atomic commit as S1/S3/258.
