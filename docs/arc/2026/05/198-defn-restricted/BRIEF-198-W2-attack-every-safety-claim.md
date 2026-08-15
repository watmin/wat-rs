# BRIEF — 198 W2: a safety claim is a hypothesis until something attacks it

> Read `DESIGN-STONE-a-restriction-governs-mention-not-head-position.md` first. W2 is the second wall
> it names and cut from the fix strikes. **Run W1 first** — it is a separate strike on the same files'
> neighbourhood. Baseline HEAD `8f0e3939` + W1's landing.

## THE CLASS — proven three times in one day

The substrate writes down **why** something is safe. Three of those arguments were never tested, and
all three were wrong or stale:

1. **`wat/kernel/services/stdio.wat:358`** — *"The gate is SAFE: the reserved-prefix gate forbids
   `:user::` code from authoring a `:wat::` caller, so no user program can construct a passing call
   site."* The premise is TRUE. The conclusion does not follow: **you never need a passing call site**,
   because the check never looked at value-position mentions. It reasoned about the **authoring**
   surface and was silent on the **reference** surface.
2. **`src/check.rs:1400` (pre-fix)** — *"The walker recurses through every `List` and `Vector` child so
   a call buried inside **a let body**… is still caught."* The let body was precisely what escaped.
3. **`src/runtime.rs:15835` + its check-time twin** — the retirement remedy told users *"or use the
   positional prime `:ns::P'`"*, which was an unguarded bypass of the very capability the type
   declared.

**The shape:** a claim of the form *"X is safe because Y cannot be done."* Y is usually true. The
claim fails because the adversary does not need Y.

## ⛔ SCOPE — this is NOT a prose sweep, and the number matters

A broad grep for safety language across `src/` and `wat/` returns **~271 hits**. That is a survey, not
a worklist, and working it would be unbounded. **Do not do that.**

All three known instances were attached to a **capability boundary**. That is the scope:

- every `#[restricted_to(...)]` site (**5**: `src/io.rs:1275`, `src/io.rs:1315`,
  `src/kernel/spawn.rs:452`, `src/kernel/spawn.rs:524`, `src/runtime.rs:26993`)
- every wat-side `:restricted-to` declaration (**7** occurrences in `wat/`)
- the reserved-prefix gate, the binder-namespace gate (251.8a-ii), the dot wall (296 H-1),
  `resolve::gate` / `resolve::register` (296 stone I), and the IPC wall (`wat/spawn.wat:329`)
- any **`SAFE` / `cannot` / `unforgeable` / `no user … can`** claim in the doc comment or the
  surrounding block **of one of the above**

**Verify these counts yourself** — every number on this arc has been wrong at least once, and a file
count is not an item count.

If the scoped set turns out much larger than ~15 claims, report the count and **STOP-2** before
attacking them one by one.

## THE WORK — per claim, in this order

1. **Quote the claim verbatim**, with `file:line`.
2. **State the argument as a syllogism**: *"safe because ADVERSARY CANNOT DO Y."*
3. **Name what Y protects and what it does NOT.** The three known failures all lived in that gap —
   the claim constrains one surface and the property needs two.
4. **Write a probe that attacks the gap**, not the premise. Do not test that Y is true; assume it is.
   Test whether the *property* holds when the adversary does something Y never mentioned.
5. **Run it.** Record the result either way.
   - **HOLDS** → record the claim as ATTACKED-AND-HELD, with the probe that establishes it.
     Strengthen the comment to say what was tested, so the next reader inherits evidence rather than
     assertion.
   - **FAILS** → ⛔ **STOP-1.** That is a live security finding. Capture it verbatim, do not fix it,
     do not weaken the claim to match reality. Report and stop.

**A claim you cannot construct an attack for is not "safe" — it is UNTESTED.** Say so plainly and move
on; do not upgrade it by silence.

## ⛔ THE FIRST CLAIM IS ALREADY KNOWN-STALE — fix it, do not re-derive it

`stdio.wat:358`'s argument is now **half-true**: the reference-surface hole it missed was closed by
`8f0e3939` (the mention rule). **Rewrite that comment** so it states the property that actually holds —
a restricted FQDN may not be NAMED outside its whitelist, in any position — rather than the
authoring-only reasoning that let the funnel stand open. Cite `8f0e3939`.

Do the same for any claim whose reasoning survived only because of a bug that has since been fixed:
the claim is now true **for a different reason**, and a comment that gives the wrong reason will
mislead the next person who extends it.

## PERSISTED OUTPUT — the deliverable is not "I read them"

Write `docs/arc/2026/05/198-defn-restricted/AUDIT-safety-claims.md`: one row per claim —
`file:line` · the claim verbatim · the syllogism · the probe · **HELD / FAILED / UNTESTABLE** · the
disposition. A claim with no row did not get audited.

Probes that hold belong in the test suite where they can rot loudly, not in a scratch dir
(`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`). Scratch `.wat` goes to
`wat-scripts/scratch-pad/` — **never** a `/tmp` path and never the session scratchpad. A `.wat` that
must FAIL cannot live there (the loader gate parses every file under `wat-scripts/`); it belongs
beside its test as a `.wat.bad` fixture.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — a probe breaks a claim.** A live security finding. Capture verbatim, report, stop. Do NOT
  fix it in this strike and do NOT soften the comment to match.
- **STOP-2 — the scoped claim set is much larger than ~15.** Report the count and the list; the
  orchestrator re-scopes. Do not start a 271-item sweep.
- **STOP-3 — you are tempted to mark a claim SAFE because you could not think of an attack.** That is
  UNTESTABLE, not safe. The distinction is the entire point of this strike.
- **STOP-4 — a fix looks necessary to make a claim true.** That is STOP-1 wearing a work item's
  clothes. Report it.

## BLAST RADIUS

Comments in `src/` and `wat/` (claim corrections only — **no behaviour changes**), new probes/tests,
and the new `AUDIT-safety-claims.md`. **If a behaviour change looks necessary, that is STOP-1/STOP-4.**

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(expect 0), then `scripts/floor.sh` and read the **Summary line**. Expect the baseline plus your new
probes; report the real arithmetic.

**On any red you did not intend: do NOT re-run.** Copy the whole stdout+stderr block **verbatim** —
never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** **Run every build and test in the FOREGROUND and block
on it — do not background anything, do not set a monitor and wait.** A rider on this arc died exactly
that way. Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.**
Never `git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds unrelated work.

⛔ **Never execute an arbitrary-fd write, a flood, or a signal as part of an attack probe.**
`--check` is sufficient to prove reachability, and reachability is what these claims are about.

## REPORT

- `AUDIT-safety-claims.md` in full, or its rows inline
- for each claim: the syllogism, the gap you attacked, the probe, and HELD/FAILED/UNTESTABLE
- every claim you marked UNTESTABLE and **why an attack could not be constructed**
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.**
