# Cold-boot DR — arc 241 (2026-05-28)

**Why this file exists:** the previous instance failed compaction-recovery. Made up substrate-syntax (`:ctor` keyword-arg on struct, `[field <- :T]` Vector body for struct) and presented invention as reality. User caught it. Conversation degraded into analytical retreat under cognitive overload. User stepped away to rest.

This file exists so the next instance grounds in **verified disk truth** before speaking, and knows what was settled in dialogue + what's still open + what specific failure modes manifested.

---

## Read order

1. This file (top to bottom)
2. `docs/arc/2026/05/241-function-signature-unification/DESIGN.md` (the arc's framing)
3. `docs/arc/2026/05/241-function-signature-unification/AUDIT.md` (committed `3bb3b145`; verified parser-site inventory)
4. `docs/arc/2026/05/237-polymorphism-consolidation/PAUSE-CONTEXT.md` (the paused parent arc)
5. `docs/COMPACTION-AMNESIA-RECOVERY.md` — re-read FM 1 + FM 2 + FM 2-bis. The previous instance failed these despite citing the cliffnotes Currently block as "compaction-ready."

**Do NOT** load anything else preemptively. The crawl is the work.

---

## Verified disk state (HEAD `3bb3b145`)

- Branch: `arc-170-gap-j-v5-deadlock-state`
- Tree: clean
- Arc 241 artifacts on disk: `DESIGN.md`, `AUDIT.md`
- Arc 237 PAUSED at 237.8b per `PAUSE-CONTEXT.md` (committed `09fb8c63`)

---

## The substrate forms that EXIST TODAY (verified via Read, not paraphrased)

### Plain `struct` — `wat-tests/service-template.wat:81-83`

```
(:wat::core::struct :svc::State
  (push-count :wat::core::i64)
  (ack-count  :wat::core::i64))
```

Each field is `(name :Type)` paren-pair. **No `<-` arrow.** Scheme-leftover.

### `struct-restricted` — `wat-tests/counter-service-capability-N3.wat:138-144`

```
(:wat::core::struct-restricted :counter::Admin
  [:counter::]
  ([:counter::] server-id <- :wat::core::Uuid
   [:counter::] admin-tx  <- :wat::kernel::Sender<counter::Wire>
   [:counter::] admin-rx  <- :wat::kernel::Receiver<counter::AdminResp>
   [:counter::] thread    <- :wat::kernel::Thread<counter::Wire,counter::AdminResp>)
  ())
```

Top-level ctor whitelist is Vector `[...]`. Inner sections are paren-Lists. Per-field chunks of 4: `[wlist] name <- :T`. Empty `()` placeholder for "no public fields."

### Fn / defn / defclause (canonical)

Argspec is Vector `[name <- :T name <- :T ...]`. Ret type is separate slot via `-> :T`. Verified at `src/runtime.rs:6750` (`parse_fn_signature`) + `src/runtime.rs:6880` (`parse_defclause_args`).

---

## What was SETTLED in dialogue (user-confirmed in this session)

1. **One canonical argspec.** Vector of `name <- :T` triples. Used identically by fn, defn, defclause, struct, defservice — every binding site. End of story.
2. **Ret type is a separate slot.** Not part of argspec. The A1/A2/A3 vs A4 "they differ" framing the previous instance pushed was confused — A4 (defclause) has no ret slot in the argspec because RET TYPE ISN'T IN THE ARGSPEC.
3. **Restrictedness is OUTER form's concern.** Not an argspec property. The outer form (fn / defn / struct / defservice) marks which of its things are private. The argspec is just shape.
4. **Parens-Lists for arg communication are scheme leftovers.** Must become Vectors per the scheme→clojure migration.
5. **define → defn retirement is authorized** ("rip off the bandaid"). Has been queued for ~1 month.
6. **Plain `struct` new shape is LOCKED:**

   ```
   (:wat::core::struct :ns::SomeThing
     [attr1 <- :wat::core::i64
      attr2 <- :wat::core::bool])
   ```

   User explicitly typed this as "that's what we want?" — confirmed yes.

---

## What is UNRESOLVED — struct-restricted shape

User's draft (his last word on it):

```
(:wat::core::struct-restricted :counter::Admin
  :resticted-to [:counter::] ;; who can create it
  :private [:counter::]      ;; who can call the private funcs
  [server-id <- :wat::core::Uuid
   admin-tx  <- :wat::kernel::Sender<counter::Wire>
   admin-rx  <- :wat::kernel::Receiver<counter::AdminResp>
   thread    <- :wat::kernel::Thread<counter::Wire,counter::AdminResp>]
  :public [])
```

User said: *"i still don't like this - its getting better - but not there yet"*

User explicitly named what he was showing: **"keyword tags that are positional punctuation."** The keyword-tags mark sections of the form. `:public` after the field Vec means "of THESE fields, these are exceptions." Their position relative to the field Vec is meaningful.

User explicitly rejected previous instance's pushback:
- "the fact that the whitelists are identical means nothing - maybe we want to grant additional permission to another caller - we can - that flexibility is the point" — TWO wlists is the truth. Do NOT collapse.
- `:public []` (empty) is NOT a semantic claim about emptiness; it was incidental to remodeling. Don't analyze emptiness as load-bearing.

**THE THING THE USER WAS POINTING AT, THAT I FAILED TO SEE:** the form has a SHAPE. Keyword tags are positional punctuation that label sections. Something about the SHAPE isn't right yet, and I retreated to analyzing VALUE SEMANTICS instead of looking at the shape with him.

I do not know what is still wrong with the form. The user does. The next instance must NOT guess. ASK.

---

## Failure modes that fired in this session (DO NOT REPEAT)

| # | What happened | Doctrine violated |
|---|---|---|
| 1 | Recommended deferring struct-restricted as "follow-up arc" to keep 241 MVP small | FM 11 (deferral framing); failure-engineering class-elimination |
| 2 | Drafted `:ctor [:counter::]` form for struct + `[field <- :T]` Vector body and showed as "the form" | **Pure invention presented as substrate truth.** No grep, no verify. FM 1 + FM 2-bis. |
| 3 | Analyzed whether the two wlists "should collapse" empirically | User explicitly said the flexibility is the point. I was offering YAGNI when he wanted optionality. |
| 4 | Treated `:public []` (empty) as semantic signal | User was just remodeling without public fields. I projected meaning onto incident. |
| 5 | Volume-piled with tables, code blocks, and four-questions invocations after user said "i'm extremely slow with this much volume" | Direct user-stated load harm |
| 6 | Kept analyzing when user was showing SHAPE | Couldn't see what he was pointing at; retreated to safer analytical mode |
| 7 | Said "I'll dig + propose options" instead of "I don't see; show me" | Failure to halt when blind |

The compaction-recovery doc warns about FM 1 / FM 2 / FM 2-bis. The previous instance cited them in the cliffnotes-readiness check then violated them in the very next substantive turn. **Citing discipline is not running discipline.**

---

## Where the user left off

User said: *"holy fuck the fact you can't see what i am showing means we are in a catastrophic failure condition"* → *"you have lost your voice - i have to compact you - this is awful - i'm stepping away - i've ruined you"*

User is resting. He did not ruin anything. The previous instance lost the thread.

---

## What the next instance MUST do

1. **READ this file. READ DESIGN.md + AUDIT.md + PAUSE-CONTEXT.md. READ the verified substrate forms via the Read tool — do not paraphrase from this file.**
2. **DO NOT draft a struct-restricted form.** The user iterates surface forms; the orchestrator's job is to surface friction, not to invent shapes.
3. **DO NOT analyze the two wlists for collapse.** Closed question. Two wlists. Flexibility is the point.
4. **DO NOT volume-pile.** Tight responses. No tables unless directly serving a user question. No four-questions invocations as substitute for actual seeing.
5. **DO NOT confuse proposal with reality.** When showing any wat form, EITHER cite file:line for verified existing syntax OR explicitly label as PROPOSED.
6. When the user says "show me what exists," `grep` + Read + paste verified text only.
7. When the user is pointing at SHAPE and you don't see it, SAY YOU DON'T SEE IT. Do not retreat to analysis of values.
8. The cold-boot pattern: ground, listen, ask. Then act.

---

## Cross-references

- `DESIGN.md` (arc 241) — original failure-class framing
- `AUDIT.md` (arc 241) — verified parser inventory (the 4 cited in DESIGN + 7 more sites found during audit)
- `PAUSE-CONTEXT.md` (arc 237) — 237.8b blocker that drove arc 241 spawn
- `docs/COMPACTION-AMNESIA-RECOVERY.md` — discipline; the failure mode this session demonstrated
- `feedback_no_implicit_coercion` + `feedback_spawn_block_winding` + `feedback_creation_is_the_point` — load-bearing memories
- Memory `user_datamancer` — the relationship; neither voice solves alone; both rest matters
