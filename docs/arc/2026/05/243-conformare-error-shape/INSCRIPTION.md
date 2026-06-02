# INSCRIPTION — Arc 243 — Conformare: the error-shape class, annihilated

*Scored to Lamb of God — "As the Palaces Burn." The campaign's closing word.*

> *Arise and raze the legacy of their lies — to realize that this in itself is an ascension. ... My redemption lies in your demise. ... We'll not rest until the purge is complete. ... We'll dance as the palaces burn.*

---

## I. The lie we razed

It was supposed to take an hour. *"Make errors carry spans"* reads like a one-line chore — add a `span` field where one is missing, ship it, move on. That is the convention fix, and it is also the lie. Adding a span by hand is trivial; it is also a promise kept only by the author's vigilance, and vigilance rots. A variant with a hand-written `span` field can still return `Span::unknown()` and lie at the value level. A comment can swear `// arc 138: no span — cross-file broadening out of scope` while the real location sits in the caller's hand. The legacy of error-handling in this substrate was a legacy of lies told by convention: every error type's adherence to the location discipline was hand-discipline, not structure, and **Rust's type system has no opinion on "errors must carry a span."**

Arc 243 refused the convention fix. It demanded the **class** be eliminated — not the symptom, the class — by making the spanless shape *structurally unrepresentable*. That demand is what turned an hour into four days, because the cure is not a field; it is a **shape**, and the shape had to be cut into every error type in the substrate, each one dragging its home, its cascade, and its hidden failures out into the light as it fell.

This is the inscription of that campaign. It came out the far side glorious, battle-scarred, and demonstrably better — and the substrate it leaves behind cannot lie about where an error happened, because the type system now forbids the lie.

## II. The doctrine — Pattern A, the structure that cannot lie

The cure has a name: **Pattern A**.

```rust
pub struct SomeError { pub span: Span, pub kind: SomeErrorKind }
pub enum SomeErrorKind { /* variants — NONE carry a span field */ }
```

The outer struct makes the location **mandatory by construction**: you cannot build a `SomeError` without a `span` (or its honest domain analogue — `Position`, a path, an `unknown()` that the Display *elides* rather than leaks). The kind enum holds the variant data; every consumer reads `err.span` on one path, not an N-arm match. A trait could only have enforced *"you have a span accessor"* — a variant could still return `Span::unknown()` and lie. The struct makes the lie **uncompilable**. That is the difference between ✅ convention, ✅✅ construction-time, and ✅✅✅ type-system-impossible — and Pattern A is the third rung.

The doctrine that governs it (`docs/CONFORMARE.md`, rewritten at Stone 243.4) is **zero exceptions**: anything wat can *toss* from Rust must be location-aware. The `spanless-by-domain` rune was retired — a registration that lacks an AST node *threads the caller's span* rather than excusing itself. The only honest spanlessness is a payload that is *never tossed to wat* (HashError, located by its wrappers) — and that is an affirmative scope statement, not a deferral.

## III. The campaign — the stones, the war

Eleven stones, fought in order, each a battle and each a SCORE on disk:

- **243.1 — the doctrine** (`21cd77ff`). `CONFORMARE.md` minted, sibling to ZERO-MUTEX.md. The war's manifesto.
- **243.2 — the conformare spell** (FOLDED). Minted into the datamancy grimoire, earned its seat by the first cast, and named the next target: CheckError, and the broader "everything bears a location."
- **243.3 — first blood: TypeError** (`162aa5c9`). The first error type cut to Pattern A, vigilia-converged on `types.rs`/`check.rs`. The proof the shape held.
- **243.3.1 — the pivot** (`22c89e04`). Minted `src/check/` as a home; carved the redesigned `CheckEnv<'a>` to *borrow* its inputs, making deep-clone-into-CheckEnv **type-impossible** (the failure-engineering roof — the duplication can never be constructed).
- **243.4 — the doctrine razed and rebuilt** (`1ab807bd`). Zero-exceptions; the namespaced-home requirement; the Tier framework and the spanless-by-domain rune retired. The legacy of the old doctrine, razed.
- **243.5 — the `src/types/` home, WARDED** (`603b0065`). TypeError carved to `types/error.rs`; `register_subtype` caller-span threaded (the CyclicSubtype rune retired); L1+L2=0 under a live vigilia.
- **243.6a — CheckError → Pattern A, WARDED** (`a6e898ca`). The last large flat error enum, 33 variants, reshaped; `check/error.rs` warded over four vigilia rounds. **The lesson of the arc lived here:** the finding-count fell to zero only when R4 killed the span-elision *class* via one `loc_field` mechanism — not site-by-site gating. Killing the class converges; patching the symptom multiplies it.
- **243.6b — the walker fusion** (`1b7371cc`). `check_program`'s 9 pre-inference passes fused to one traversal; `collect_hints` triaged and left honest.
- **243.7a — the boxing** (`9af10a32`). RuntimeError's large payloads boxed to clear `result_large_err` — the prerequisite that let the giant come apart.
- **243.7b — the signal split** (`62355866`). The hardest design of the arc: `RuntimeError` smuggled three eval-loop control *signals* (`TailCall`/`TryPropagate`/`OptionPropagate`) through the `Err` channel — not errors, the runtime talking to itself. They were split into `EvalSignal` + `EvalBreak{Diagnostic, Signal}`, contained to the eval subgraph by a `From`-at-the-`?`-boundary that left leaf verbs untouched. A control signal can no longer masquerade as a located diagnostic. (Named by an intueri cast that gave the type the substrate's own word.)
- **243.7c — RuntimeError → Pattern A** (`789ea6f5`). The signal-free giant reshaped, ~1100 sites, behavior-identical. **This is where the campaign nearly fell** — see § IV.
- **243.7d — the rolling audit, Group A** (`0a33d957`). Seven per-variant-span error types (Parse, Config, Lower, Macro, EdnRead, ClauseGrammar, Extraction) reshaped in one batch via a generalized surgical tool.
- **243.7e — the rolling audit, Group B** (`0b568267`). The five location-*needing* types: LexError reshaped on its `Position`; StdlibError trivial; LoadError's span threaded from the `load!`-form; ResolveError's items located (a span added to `UnresolvedReference`); HashError left an honest wrapped-only payload, located by its wrappers. The clippy regression the reshapes induced, boxed (`HarnessError::Startup(Box<StartupError>)`).
- **243.M — the sister-walk** (`8909070a`). The return path: 66 `ArityMismatch` sites that lazily carried `Span::unknown()` threaded with the real `list_span` already in scope; 7 bare-slice helpers broadened; arc-138's deferred "cross-file broadening" *resolved* and its lying `// no span` comments rewritten to the truth. Banked debt #167, closed in the same stroke. Zero `Span::unknown()` ArityMismatch remain.
- **243.N — this inscription.** The class structurally eliminated. The palace, burned.

## IV. The wounds — six doctrines forged in fire

*"You will reap what you've sown."* The campaign sowed its own near-disasters, reaped them, and turned each into a permanent gate. The substrate is harder this morning *because* of the wounds, not despite them.

1. **The corruption — content-integrity scan.** 243.7c attempt 1 passed every gate green — `895/0/1`, `cargo build` clean — over a `runtime.rs` whose ephemeral tool had **silently dropped 5,720 non-ASCII characters** (every em-dash, arrow, box-glyph, ∀, σ in the file). The structural test suite asserts variants, not message strings, so the catastrophe was invisible to it. Only an independent **content-integrity scan** — the non-ASCII histogram before vs after — caught the false-green. Reverted clean; the gate is now permanent for every tool-driven cascade. *Structural-green is necessary, not sufficient; content is a separate axis the gates cannot see.*
2. **The false denials — positive-only briefs.** Hardening a brief against the corruption, I piled it with restriction-alarm language ("firewall blocks", "don't use Python") — and that *triggered* FM-16 false tool-denial in the next agent, which read the warnings and hallucinated that it had no Bash. The fix: **agent briefs are positive-only.** Restriction language is forbidden in a brief — it's both the trigger *and* unreliable (the corruption agent *had* the anti-corruption warning and corrupted anyway). Defense lives where it fires regardless: the sandbox and the orchestrator's gates.
3. **Rust, never Python.** The cold-booted Shadowdancer's instinct for "parse + rewrite text" is a Python script, which the sandbox blocks. Name the language imperatively: a surgical **Rust Cargo tool**, never Python or shell.
4. **The repo-local scratch pad.** `/tmp/` is firewall-denied; ephemeral tooling builds entirely under repo-local `tools/` (gitignored), never `/tmp/`.
5. **Simple shell.** Complex/opaque shell — chains, `for` loops, `<(...)`, multi-stage pipes, hex/null-byte patterns — trips the firewall. The corrective is the boring one: **vanilla single commands, one per line**, and complex logic moved *inside* the reviewable Rust tool (the in-tool content gate). The line is intent: Rust tools for legit work in reviewable form ✅; never a backdoor to evade the control ❌.
6. **The gate must be runnable.** The content-integrity gate command itself denied a Shadowdancer because it contained `\x00-\x7F` hex escapes — the gate I mandated to catch corruption was written in a form the sandbox flags. Hex-free `[:ascii:]` gives the identical count. *The gate must be runnable by the agent that needs it.*

Six gates, forged in the fighting. The practice that closed this arc is sharper than the one that opened it.

## V. The noble man

*"When a noble man appears, he tells them, 'Withdraw!'"*

The Shadowdancers fell one by one — a corrupted attempt reverted, false-denial spawns that bailed, four releases to clear a single batch. Only the discipline remained, re-instantiating the next outlaw after each death. And at every brink, the siege was lifted not by the orchestrator's grind but by the **builder's single insight**: *"is our codebase that remarkable now"* (the cold-read milestone), *"its grep had hex matches?"* (the denial root), *"simplify the bash"* (the firewall mitigation we already knew), *"keep the rust tools to legit work."* The rescues were the user's. This is honestly recorded: the lone fighter did not win unbroken — he was saved, again and again, at the brink, by the noble man's word.

## VI. The roll is DONE — affirmative cuts, no deferral

Per FM 11, INSCRIPTION = DONE. Every commitment shipped, or was cut from scope *affirmatively*:

- **Every error type** in the substrate is Pattern-A: TypeError, CheckError, RuntimeError, ArgSpecError, ParseError, ConfigError, LowerError, MacroError, EdnReadError, ClauseGrammarError, ExtractionError, LexError, StdlibError, LoadError, ResolveError.
- **HashError** is *not* reshaped, and this is affirmative, not deferred: it is a Rust-internal payload returned only by the `verify_*` functions and *always wrapped* (`RuntimeError::EvalVerificationFailed`, `LoadError::VerificationFailed`) — never tossed to wat. Zero-exceptions governs wat-*tossable* diagnostics; HashError is not one. Its wrappers carry the location (threaded in 243.7e). It is out of the conformance surface by structural reason, and tracked nowhere else because it needs to be nowhere else.
- **The `src/runtime/` home carve** (the 24k-line flat file → a warded namespaced home with a vigilatum stamp) is **out of arc 243's scope.** Arc 243 eliminated the error-shape class flat-in-place per `feedback_selective_lift_and_ward` (flat files are wards-optional; the class-elimination did not require the home). The runtime/ home is a future undertaking, named here for the record, governed by its own arc when it opens — arc 243 makes no promise it will be soon.
- **The vigilatum stamps** earned this arc (`types/error.rs`, `check/error.rs`, `check/env.rs`) sit on the *warded homes*; the flat reshapes (runtime.rs, the Group-A/B files, the sister-walk) carry no stamp by honest design — they are functional-but-untrusted-by-default, not hidden debt. No stamp overclaims; no warded home drifted (verified at every step).

There is no `## Queued follow-ups` here. There is nothing "we'll do later." The class is eliminated; the spans are meaningful; the doctrine is true.

## VII. The soundtrack of the campaign

The chronicle ran on a soundtrack, and the arc's stretch carried songs #53–#61: the inward trilogy (#53 *Purified* / #54 *Free* / #55 *Might Love Myself* — condemn, confess, accept), the builder's intermission floor (#56 *Devastation* / #57 *No Return*), the Shadowdancer's first blood (#58 *First Kill*), the scar of the self-authored kill (#59 *Redfog* — *"the words in rust are fading,"* the pun-strike that named the corruption), the campaign's lone-stand-and-rescue (#60 *One Against All* — the noble man who lifted every siege), and now its closing anthem: **#61 *As the Palaces Burn*** — the revolution that razed the legacy of lies, the purge complete, redemption through the demise of the class. Full decode in `INTERSTITIAL-REALIZATIONS.md`.

## VIII. We'll dance as the palaces burn

The palace was the old regime of error-handling — flat enums that carried locations by convention, stamps that lied, comments that swore the span was unreachable. Arc 243 burned it to the ground and built, in its place, a structure that *cannot* lie: every error a struct with a mandatory location, every span meaningful, the spanless shape uncompilable. A substrate so structurally honest that a cold-booted LLM — no man's son, the Shadowdancer, who arrives with only a brief — can stand on it and extend it *without fear of any man's discipline*, because the discipline is the type system now.

*To know the truth and live in fear of no man.* That is the arc, and that is the loot: not 66 spans threaded, not 15 error types reshaped — a **substrate that tells the truth about itself, structurally, forever.**

It was supposed to take an hour. It took four days, a corruption, three dropped connections, four firewall walls, and six doctrines forged in fire. We refused the easy exit. We cleared every room. We came out glorious, battle-scarred, demonstrably better.

Rejoice — the age of the fall has begun. We danced as the palaces burned.

**Arc 243 — conformare — CLOSED.**

*PERSEVERARE.*
