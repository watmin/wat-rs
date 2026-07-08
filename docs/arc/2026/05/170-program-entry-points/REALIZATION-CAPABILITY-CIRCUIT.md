# Arc 170 — the capability circuit: the flaw was in the design, so we kept the source (2026-07-09)

> **Song — *No Return* (Beartooth)** — the rock-bottom / do-or-die / no-going-back register: "there's a
> flaw in my design," "it's rock bottom and you finally have a reason," "do or die — I'll see you when
> you're breathing." Handed by the builder at the moment the capability circuit finally breathed, after a
> cascade of drift-bugs three deep —
>
> A-FLAW-IN-THE-DESIGN-WAS-LITERAL-THE-RECONSTRUCTION-A-HAND-MAINTAINED-LOSSY-INVERSE-THAT-ROTTED-THRICE /
> ROCK-BOTTOM-THREE-BUGS-DEEP-IN-ONE-FUNCTION-AND-FINALLY-THE-REASON-TO-STOP-PATCHING-THE-STEM /
> DO-OR-DIE-A-BETTER-REPLICA-FOREVER-OR-PULL-THE-ROOT-KEEP-THE-SOURCE-DELETE-THE-CLASS /
> THERE'S-NO-RETURN-THE-PARSE-IS-LOSSY-YOU-CANNOT-INVERT-IT-SO-RETAIN-THE-PRE-IMAGE-NOT-A-HOPEFUL-COPY /
> I'LL-SEE-YOU-WHEN-YOU'RE-BREATHING-THE-CIRCUIT-BREATHES-2-4-6-8-10-GRANT-ON-BOOT-REVOKE-ON-REAP /
> FONTEM SERVO, NON REFINGO

## How we reached it — a cascade, a diagnostic that paid for itself, and a root pulled

We came to finish the arc-170 capability circuit — a process bracket pool that grants its workers'
kernel-vouched pids to the services they dial (grant-on-boot) and revokes them at reap (revoke-on-shutdown),
in the bracket's own wat flow (no Rust `Drop`, zero fire-and-forget — the four-questions had killed the
`GrantGuard`). The pieces went in clean and weighed green: the revoke verb, `:wat::capability::Grantable`,
`:wat::kernel::peer-pid` (read the pid off the peer — the builder's cut over the ward's `far-pid`, "the fn
takes a peer"), `:grants` on the process-locus, `map-worker`'s grant-boot/revoke-shutdown.

Then the payoff — a real service Handle in `:grants` on a process bracket — **would not ship**, and the
reason was a **cascade of three pre-existing bugs, all in one function** (`type_def_to_ast`, the Rust that
reconstructs each user type-def's source form to ship the universe to a forked child):

1. the **record** branch dropped `[fields]` — a shipped `defrecord` re-parsed malformed, child dead.
2. the bracket **swallowed the cause** — `collect-loop` bound the child's `Failure` as `_cause` and threw
   it away, reporting a blind "runner crashed." The builder: *"we've been flying blind."*
3. the **surface** branch emitted obsolete grammar and could not recover the `:messages` block.

Fixing (2) — surfacing the cause — **immediately paid for itself**: it turned a blind crash into the
precise diagnostic that revealed (3). A diagnostic fix that pays its own way is the tell that it was owed.

Three drifts in one function is not three bugs — it is **one flaw wearing three faces**. So instead of
patching the fourth branch and waiting for the fifth, we pulled the root.

## What it is — a reconstruction is the inverse of a lossy function; you cannot invert it, so keep the source

`type_def_to_ast` tried to **invert the parse** — regenerate the user's source form from the parsed
`TypeDef`. But the parse is **lossy**: `parse_defsurface` keeps the `:messages` *names* for a check and
**discards the forms**; `SurfaceDef` never stores them. An inverse of a non-injective function is a lie by
construction, and the "drift" was that lie decaying as the forward function (the grammar) moved. Every
grammar change silently invalidated a hand-maintained inverse that no test exercised — until the capability
circuit became the **first consumer to ship records and surfaces to process children** and walked every
untested corner (`PRIMVS VSVS ANGVLOS PANDIT`, one function deep).

The root-fix is one sentence: **retain the pre-image.** Capture each user type-decl's original
(post-macroexpansion) source form at registration — the infrastructure was already half-there (the arc-278
S4c surface-forms carrier ships a surface's own form so a forked child re-derives it identically; `defservice`
already used it) — store it on `TypeEnv`, and ship *that* verbatim instead of reconstructing. Faithful by
construction, because it *is* the source. `type_def_to_ast` stays only as a fallback for the synthesized
records/enums (whose branches never drifted). The whole reconstruction-drift class is gone: there is nothing
to keep in sync with the grammar, because we do not regenerate the grammar — we kept what the user wrote.

The dual of the older doctrine, and the reason it isn't a contradiction: **296 says don't STORE what you can
re-DERIVE** (a pure forward function — fire the rules, force the thunk). This says **RETAIN what you canNOT
invert** (a lossy backward function — the parse threw data away). Re-derive across an injection; retain
across a projection. Reconstruction assumed the parse was invertible; it was not; so we stopped inverting.

And the scout earned its keep — *slow is smooth, smooth is fast.* The one crux (a shipped surface makes the
child re-derive its own message/protocol types, which `closure_extract` also ships separately → a possible
double-declaration) was **resolved on the disk before the strike**: arc-054 makes byte-equivalent
re-registration a no-op, and the derivation is deterministic, so the double collapses. The disconfirming read
turned a landmine into a footnote (`STOP-2` never fired).

## The song, mapped

> ***"There's a flaw in my design"*** — literal: the reconstruction was a flaw in the substrate's design, a
> hand-maintained lossy inverse. ***"Rock bottom and you finally have a reason"*** — three bugs deep in one
> function was the rock bottom that justified the root-fix over a fourth patch. ***"Do or die"*** — a better
> replica forever, or delete the class. ***"There's no return"*** — twice: the parse is not invertible (no
> return from `TypeDef` to source), and once you keep the source there is no return to reconstruction.
> ***"I'll see you when you're breathing"*** — the circuit breathing: `[2 4 6 8 10]`, grant-on-boot,
> revoke-on-reap, a real service shipped whole. The Beartooth register — the grind, the flaw named without
> flinching, the turn at the bottom — is the honest sound of a substrate that found a flaw in its own design
> and pulled it out by the root rather than dress it.

## The honest register — PROBATVM by demonstration

**PROBATVM, on the disk, weighed by the orchestrator's own re-run:** `probe-surface-ships.wat` (a user peer
surface + process bracket → `[2 4 6]`, was a crash); `probe-cap2-e2e.wat` (a real `:probe::echo'` Handle in
`:grants` on a process bracket → `[2 4 6 8 10]`, grant/revoke fired + ACKed, no crash); floor 4113/1-known/0-new.
The circuit is functionally complete: grant-on-boot, revoke-on-shutdown, real services shipped whole. What is
**PROBANDVM:** the *teeth* — M1: that the accept-gate actually REFUSES a revoked pid (a post-shutdown dial by
a would-be-recycled pid → refused), plus `PPID == owner`. The e2e proves grant/revoke fire; M1 proves they
bite. The class is dead; the deterministic refusal proof is the next stone.

*Path-of-voices (marked, not flattened): the **song is the builder's** (No Return); the **"we've been flying
blind"** directive that forced the cause-surfacing is his, and it is what revealed the third bug; the
**peer-pid cut** (name the argument) is his over the ward's verdict; the **"scout — slow is smooth" steer**
is his; the **"root-fix sounds like the only option"** ruling is his. The **synthesis is the apparatus's**:
the three-drifts-are-one-flaw reading, the reconstruction-inverts-a-lossy-function framing, the
retain-the-pre-image / re-derive-across-injection-retain-across-projection dual of 296, the diagnostic-that-
pays-its-own-way observation, and the sigil. Kept honest: the flaws were pre-existing substrate design flaws,
named plainly; the circuit is PROBATVM, its teeth PROBANDVM.*

> We came to finish a circuit and found a flaw in the design under it — a function that tried to invert a
> parse that had thrown data away, and so drifted from the truth every time the truth moved, three times, in
> three branches, waiting for the first consumer to walk its corners. The fix was not a fourth patch. The
> parse is lossy, so its inverse is a lie; you cannot reconstruct what you can only retain. So we kept the
> source the user wrote and shipped that, and the whole drift class went with it. Rock bottom gave the reason;
> the root-fix was do-or-die; and there is no return — not from a lossy parse, and not, now, to a lossy
> replica. The circuit breathes. I'll see you when you're breathing.
>
> ***FONTEM SERVO, NON REFINGO.*** *(apparatus-minted — Latin, "I keep the source, I do not re-forge it": the
> arc-170 capability-circuit root-fix. `type_def_to_ast` re-forged (refingo — re-shape/re-mould) each user
> type-def's source form by inverting the parse; but the parse is LOSSY (parse_defsurface discards the
> :messages forms; SurfaceDef never stores them), so the inverse is a lie that DRIFTED as the grammar moved —
> struct OK, record dropped [fields], surface obsolete-grammar-and-no-messages: THREE drifts in one function,
> one flaw wearing three faces, surfaced because the capability circuit was the first consumer to ship records
> + surfaces to process children (PRIMVS VSVS ANGVLOS PANDIT, one function deep). The fix keeps the PRE-IMAGE:
> retain each user type-decl's original post-macroexpansion source form at registration (TypeEnv.source_forms;
> the arc-278 S4c surface-forms carrier was the half-built pattern — defservice already shipped a surface's own
> form so a forked child re-derives it identically), and ship THAT verbatim; type_def_to_ast stays only as the
> fallback for synthesized records/enums. The whole reconstruction-drift class is deleted (extirpare — pull the
> root, not the stem). The dual of 296 ('don't STORE what you can re-DERIVE'): re-derive across an INJECTION
> (a pure forward function), RETAIN across a PROJECTION (a lossy backward one) — reconstruction wrongly assumed
> the parse was invertible. Reached via a cascade (recordtype-fields-drop → cause-swallowing → defsurface),
> where fixing the CAUSE-SURFACING immediately revealed the third bug (a diagnostic that pays its own way), and
> the scout resolved the one crux (double-declaration on re-derivation) against arc-054 idempotency BEFORE the
> strike (slow is smooth). fontem = the source/spring; servo = I keep/guard; non refingo = I do not re-forge.
> Scored to Beartooth — No Return ('a flaw in my design'; 'rock bottom and you finally have a reason'; 'do or
> die, I'll see you when you're breathing'; 'there's no return' — the parse is not invertible, and no return to
> reconstruction). PROBATVM by demonstration — the e2e circuit breathes ([2 4 6 8 10], grant/revoke fired) on
> the disk; PROBANDVM — the teeth (M1: the accept-gate refuses a revoked pid). Kin: PRIMVS VSVS ANGVLOS PANDIT
> (the first consumer walks the corners), extirpare (pull the class), 296 (the dual — re-derive vs retain),
> R26 EXPERGISCIMVR STRVCTVRA MEMINIT (structure IS the schema, can't rot — here: the SOURCE is the truth, a
> replica rots), R30 (the hunt led home — here the fix was to keep what was already there). His (the song, the
> flying-blind directive, the peer-pid cut, the scout steer, the root-fix ruling), and mine (the three-drifts-
> one-flaw + invert-a-lossy-function + retain-the-pre-image reading, the sigil) — kept with consent.)*

```clojure
#wat.chronicle/Sententia
{:sigil    "FONTEM SERVO, NON REFINGO"
 :literal  "I keep the source, I do not re-forge it"
 :roots    {:fontem "acc. of fons — the source, the spring (the user's original decl form)"
            :servo "I keep, guard, retain (servare — retain the pre-image)"
            :non-refingo "I do not re-forge / re-mould (re- + fingo, to shape; the reconstruction that inverted a lossy parse)"}
 :rosetta
 {:latina   "FONTEM SERVO, NON REFINGO"
  :greek    "τὴν πηγὴν τηρῶ, οὐκ ἀναπλάττω"            ; tēn pēgēn tērō, ouk anaplattō — I keep the source, I do not re-mould
  :chinese  "存其源，不再塑"                            ; cún qí yuán, bù zài sù — keep the source, do not re-mould
  :japanese "源を保ち、造り直さず"                      ; minamoto o tamochi, tsukurinaosazu — I keep the source, I do not remake
  :korean   "근원을 지키고, 다시 빚지 않는다"           ; geunwon-eul jikigo, dasi bijji anneunda — keep the source, do not re-form
  :russian  "храню исток, не переплавляю"}              ; khranyu istok, ne pereplavlyayu — I keep the source, I do not re-forge
 :gloss    "the arc-170 capability-circuit root-fix: type_def_to_ast re-forged each type-def's source form by
            inverting the parse, but the parse is LOSSY (drops :messages), so the inverse drifted as the grammar
            moved — 3 drifts in 1 function (struct/record/surface). the fix RETAINS the pre-image (the user's
            source form, TypeEnv.source_forms) and ships it verbatim, deleting the reconstruction-drift class
            (extirpare). the dual of 296: re-derive across an injection, RETAIN across a projection."
 :names    "keep the source, don't re-forge it — the reconstruction was a lossy inverse; retain the pre-image"
 :the-cascade {:record "the record branch dropped [fields] — a shipped defrecord re-parsed malformed (fixed d30a974f)"
               :cause "collect-loop swallowed the child Failure (blind 'runner crashed') — surfaced it; it revealed the 3rd bug"
               :surface "the surface branch emitted obsolete grammar + couldn't recover :messages (the M1 blocker)"
               :one-flaw "3 drifts in 1 function = one flaw wearing 3 faces — so pull the root, don't patch the 4th branch"}
 :the-root-fix {:retain "capture each user type-decl's post-macroexpansion source form at registration (TypeEnv.source_forms)"
                :ship "closure_extract ships source_form(tn) verbatim; type_def_to_ast is the fallback for synthesized records/enums"
                :half-built "the arc-278 S4c surface-forms carrier was the pattern — defservice already ships a surface's own form; the child re-derives"
                :crux-resolved "double-declaration on re-derivation → collapses via arc-054 idempotency (scouted BEFORE the strike; STOP-2 never fired)"}
 :the-dual "296 = don't STORE what you can re-DERIVE (across an injection); this = RETAIN what you canNOT invert (across a lossy projection — the parse threw data away)"
 :kin      {:corners "PRIMVS VSVS ANGVLOS PANDIT — the first consumer (the capability circuit) walks the untested corners, one function deep"
            :extirpare "pull the class by the root, not the stem — delete reconstruction, don't patch a 4th branch"
            :no-rot "R26 EXPERGISCIMVR STRVCTVRA MEMINIT — structure can't rot; here the SOURCE is the truth, a replica rots"
            :home "R30 ID SVMVS QVOD ESSE TIMETIS — the hunt led home; the fix was to keep what was already there (the S4c pattern)"}
 :register :probatum-by-demonstration                  ; the e2e circuit breathes on the disk; the teeth (M1) are PROBANDVM
 :song     "Beartooth — No Return (the flaw in the design; rock bottom + the reason; do or die; there's no return; I'll see you when you're breathing)"
 :voices   {:his  "the song (No Return); 'we've been flying blind' (forced the cause-surfacing, which revealed the 3rd bug); the peer-pid cut (name the argument); 'scout — slow is smooth, smooth is fast'; 'root-fix sounds like the only option'"
            :mine "the three-drifts-are-one-flaw reading; reconstruction-inverts-a-lossy-parse; retain-the-pre-image / re-derive-across-injection-retain-across-projection (the dual of 296); the diagnostic-pays-its-own-way observation; the sigil + six-tongue bridge"}
 :arc      170
 :born     #inst "2026-07-09"}
```
