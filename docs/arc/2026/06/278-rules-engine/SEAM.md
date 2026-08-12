# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD 89c6d355   pushed   floor 4391 passed / 0 failed / 262 skipped   clippy 0
```

`git status` clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` STILL HOLDS THE LIFECYCLE STRIKE — do not `git stash drop`.** Made with `-u`, so it
has **three parents**; `git stash show --stat` sees only the tracked one and **cannot see the
untracked payload**. Read it with `git show 'stash@{0}^3:<path>'`. Its `.wat` is **STALE** —
`--check` exit 1, five errors (four are one root: a `DisconnectReason` scrutinee resolving to an
unbound type var; the fifth is R57's recv-outcome wall on a dropped `recv`). Restoring it turns the
floor red, so it stays stashed — and the lifecycle strike now opens with a **located worklist**.

⚠ **`--check <f> | tail` returns TAIL's exit code.** Both files "passed" until re-run without a pipe.

## ★ WHAT LANDED THIS STRETCH — five commits, each weighed by own `--release` re-run

| commit | |
|---|---|
| `8e661362` | the union-closure gate goes **`VERDICT MEANINGFUL`** — a `fn-forms` union boots a real forked child; plus the probe that **refuted** the seam's own prescribed first act |
| `310f8050` | **`Peer' derives ThreadSelfPeer'`** — the safe edge, stated ONCE, one-way, with the negative gate |
| `2fff3749` | the child-entry stone drawn + briefed + its disconfirming probe (**PROVEN BY RUN**) |
| `c44873e6` | the strike **struck, failed, REVERTED** — and the failure was worth more |
| `076b2a2c` · `89c6d355` | **`fn-forms` stops reading DATA as CODE**; the unbound-`?` NOTE |

## ⛔ THE LIVE QUESTION — rete is PRIVILEGED in the boundary door, and it must not be

**Builder's cut, and it is the standing frame:** *"this must be agnostic to rete — any user defined
dsl must be tolerable… we cannot make ourselves special."*

`src/resolve/boundary.rs::quote_boundary` — by its own doc *"the ONE place the boundary-head set is
encoded"* — mixes two kinds of entry:

- **the language declaring its own grammar** — `quote`, `quasiquote`, `match`, `define`, `forms`.
  Legitimate. Every compiler knows its own special forms. (`match` on an enum **is core** —
  builder-confirmed.)
- **a LIBRARY's grammar inside the compiler** — `:wat::rete::make-rule` → `MakeRule`, plus
  `is_where_form()`, an entire function for one library form. **rete got to edit the compiler's
  list; a user's DSL cannot.** That is what makes a user DSL second-class.
  (`:wat::holon::literal` is NOT this class — holon-rs merges INTO wat-rs and stops being a dep,
  builder's call. **rete is the only one left.**)

`closure_extract` therefore honours the language facts and **REFUSES the library ones**
(`MatchesSubject | MakeRule => {}`). That arm is a **self-deleting marker**: exhaustiveness makes it
a compile error the moment those variants die. It cannot outlive the defect it marks.

### ▶ THE FORK, POSED AND UNRULED — and my proposed shape is NOT yet grounded

| | |
|---|---|
| **(A)** rete re-expresses `:when` in existing machinery — delete `MakeRule` + `is_where_form` | smallest, no new concept |
| **(B)** a form **DECLARES** its own boundary — a registration any DSL uses, rete included | the general answer; removes the *possibility* of privilege, not just this instance |

⚠ **GROUNDING OWED BEFORE (A).** I claimed "express `:when` as a quasiquote with `where` bodies as
unquote escapes." **That is unproven and may be wrong.** For RESOLUTION an unquote escape means
*"resolve this subtree in place"* — fits. For EVALUATION `~x` means *"evaluate x and splice the
VALUE"* — but rete needs the `where` body to stay a FORM inside the rule data. If those diverge,
quasiquote is right for one pass and wrong for the other, and reusing it trades a visible privilege
for a subtle semantic bug. **One probe decides it:** does a quasiquoted `where` body survive both
passes intact? If yes, (B) may be unnecessary; if no, the failure tells you what (B) must carry.

## What the `fn-forms` fix actually was (do not re-derive)

The walker had **no concept of `quote`** (`grep -n quote` → zero hits) and so **read quoted data as
code**, raising `UnresolvedSymbol` on symbols that were never references. Isolated rete-free: two
arms differing in one thing, the subject raising on `mystery-symbol` — a plain bare Symbol,
deliberately not `?`-prefixed — **inside the quote**. The rete symptom (`?c`) was incidental:
`defrule` QUOTES its `:when`/`:then`.

**MEASURED, and it killed my own recommendation twice:**
- refusing `MakeRule` cost **nothing** on the raise path — `make-rule`'s `:when` *is itself a quote
  form*, so `AllData` stops the walk anyway. The special case only ever bought dep-collection inside
  `where` bodies. **It looked load-bearing and was mostly ornamental.**
- `fn-forms` over a rule fn returns **`closure forms=5`** — the child-entry blocker, measured
  directly. The floor could NOT have shown this: it was green with that strike reverted.

## ⛔ ALSO OPEN

**The child-entry strike** — `DESIGN-STONE-the-child-entry-kills-the-manifest.md` + BRIEF +
EXPECTATIONS, all correct about WHAT to build, all still on disk. **REVERTED.** Its remaining reds
are untouched by the quote fix: the rotted `wat-scripts` consumer of `service-forms`, and
`a_forked_service_that_cannot_decode_a_message…` which is **UNCHARACTERIZED** (saw `Outcome/Message`,
requires `Outcome/Lost`) — *"probably the same root"* is not a disposition.

⚠ **A rider scored "thread tier untouched" GREEN off one passing thread test + an empty
`spawn.wat` diff, while four thread tests were red.** Both facts true; neither could see the
violation. Any row of the form "tier X untouched" must be measured by that tier's **whole set**.

**`NOTE-a-rule-may-reference-an-unbound-variable-and-compile-clean.md`** — builder-ruled "we'll deal
with it in time." **Tier 0 is an owed measurement**: does *firing* the broken rule raise, or derive a
corrupt fact? Every other tier is gated on it.

**Filed, not scheduled:** `109/NOTE-two-resolvers-over-the-five-registries.md`.
**Owed intueri casts:** the admission type; the correlation surface.
**Older:** #87 · #49 · #7 · #17 · #19 · #20 · #50 · #58 · #60 · #64 · #67 · #81.

## The rules this stretch paid for

- **Routing through a shared "one door" INHERITS whatever the door encodes.** I adopted
  `quote_boundary` to kill a privilege and imported a different one in the same motion. Audit the
  door before you route a third consumer through it.
- **A special case can be earning far less than it appears.** `MakeRule` looked load-bearing;
  measured, it bought nothing on the path that mattered.
- **A text-range deletion does not know about attachment.** My python surgery cut BETWEEN a doc
  comment and its function — `walk_match_form` lost its doc and `walk_quasiquote_template`
  inherited one about *match patterns*. **Clippy caught it; no test could.**
- **When a check comes back CLEAN, ask what it cannot SEE** — the fourth instance this arc.
- **The instrument that reports a defect may have CAUSED it** — the union gate's own dedup deleted
  the declaration whose absence it then reported.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> This stretch, the record's own doubts did the work: a written-down *"do not assume it transfers"*
> cancelled a day aimed at the wrong file, and a defence-objection — *"who catches a broken rule?"* —
> was measured and came back **nobody**, which is now a NOTE.
>
> The line that cost the most: **my own recommendation was refuted three times by measurement, and
> every refutation made the answer better.** Pose the fork, run the probe, and let the disk rule.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
