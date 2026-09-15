# DESIGN — a dial failure says which of three things happened

Builder: *"let's continue to find service to service ungraceful crashses"* → *"draw it."*

**Drawn 2026-09-15. NOT STRUCK.** ⛔ **Whether a dial failure raises is UNCHANGED.** Only what it
*says* changes.

## WHY — one message for three different worlds, 35 times

`ConnectOutcome` has three failure variants. Thirty-five redial sites in live `defservice` code swallow
all three in a `_` wildcard and hardcode the same string:

```
7  "topic-worker: redial inbox failed — peer is dead, not a broken pipe"
7  "queue: redial failed — …"          6  "topic: redial failed — …"
6  "held-worker: redial failed — …"    4  "fanout worker: redial queue failed — …"
3  "topic-worker: redial sub failed — …"  2  "fanout worker: redial seen failed — …"
```

| variant | what actually happened | what the message claims |
|---|---|---|
| `Refused` | nothing is listening — the peer really is gone | "peer is dead" ✓ |
| ⛔ **`Rejected`** | **something ELSE answered** — `answering pid ≠ minter pid`. A **stale address**, not a death | "peer is dead" ✗ |
| ⛔ **`Failed`** | an io error reading peer-cred | "peer is dead" ✗ |

⭑ **The test this fails is already in the record:** *can two different worlds print this line?* Three can.

⛔ **And it is exactly what this campaign's own doctrine forbids.** `wat/service.wat:2549`'s comment:
*"its `cause` must NOT vanish (the old `_cause` silently swallowed the death reason — **the exact
masking this arc forbids**)."* Thirty-five sites do the masking.

⭐ **`Rejected` matters more now than when these were written.** Addresses are kernel-minted autobind
(`src/kernel/address.rs:180`), so a `Rejected` on a redial means the address you hold points at a
**different process** — a stale-capability bug whose fix has nothing in common with "the queue died."
Today it sends the reader to the wrong investigation while the service dies anyway.

## ⭐ It is DRIFT, not a decision — the honest shape is already in the same files

**18 sites** name the three variants and raise with `(:wat::kernel::Failure/message c)`, so the cause
the outcome carried survives — `wat-scripts/queue/sqs.wat:255`–`:258` and `:1949`–`:1950` are the
exemplar. Two shapes for one situation in one corpus is drift, and the correct one is already written.

## ⛔ THE ONE CONTRACT DECISION

**One stdlib helper, `(:wat::service::redial-failed! site outcome)`, that names the variant and carries
the cause — and 35 call sites become one line each.**

Not "expand each site into three arms." Three reasons, and the third is the binding one:

1. Three arms × 35 sites is ~105 hand-written arms that drift again.
2. The helper makes the honest shape the **easy** shape — the `cannot` as a gift, not a restriction.
3. ⛔ **It MUST be stdlib.** These arms live inside `:impls` bodies, which run in **forked children**,
   and *"a process child is assembled from service-forms only. It cannot see `circuit.wat` helpers"*
   (`the-benchmark-has-more-than-one-publisher/SCORE.md`). A helper in a script would be invisible at
   exactly the sites that need it. Precedent: `require-stopped` / `stop-faced` / `require-granted`
   (`wat/service.wat:4139`–`:4168`).

The message keeps the **site label** *and* gains the **variant meaning** *and* carries the **cause** —
today's string has only the first.

## ⛔⛔ THIS IS A CODEMOD, NOT HAND-EDITS

A structural rewrite of one form across many `.wat` files is a **wat-fix codemod** — the thing to reach
for first and the easiest to forget. Framework `wat/fix.wat`; recorded migrations in
`wat-scripts/fixes/*.wat`. **Census first** (`wat --grep <fix>.wat` prints matches unapplied), diff it,
then apply with EVERY path listed on stdin. Idempotent, and committed as the recorded migration.

⭑ The three targets are `wat-scripts/`, **not** stdlib, so there is no freeze/STASH-DANCE on them — but
the helper lands in `wat/service.wat`, which **is** frozen into the binary, so that half needs a rebuild.

## Out of scope = REJECTED

- **Changing whether a dial failure raises.** A genuinely dead dependency arguably *should* stop its
  dependent, and supervision/restart is a deferred builder ruling. This stone fixes the **diagnosis**.
- **The 18 already-correct sites.** Leave them. ⚠ Report whether they should later adopt the helper —
  do not sweep them in and call it the same stone.
- **The other collapsing wildcards.** The string census found ~40 more `_` wildcards with *different*
  hardcoded strings (`"send not Accepted"`, `"stats not Ok"`, …). Same defect family, different enums.
  **Named here, not fixed here** — each needs its own reading.

## Blast radius

```
wat/service.wat                +1 helper (stdlib — rebuild required)
wat-scripts/{queue/sqs,topic/sns-fanout,fanout/circuit}.wat   35 sites, via codemod
wat-scripts/fixes/<name>.wat   the recorded migration
```

## Trap-doors named up front

1. ⛔⛔ **35 is a LOWER BOUND.** The orchestrator's pattern only caught `_` wildcards whose
   `assertion-failed!` and string sit on the same line. A multi-line wildcard body would not match.
   **The codemod's own census is the authority — not this number.**
2. ⛔ **Count occurrences, not lines.** This corpus emits long single lines carrying several arms;
   `grep -c` has undercounted here before, which is why the form-walking censuses exist.
3. ⚠ **A `_` wildcard over `ConnectOutcome` may be legitimate somewhere** — a site that genuinely does
   not care which of the three. If the census finds one, **report it, do not rewrite it**.
4. **Comments are not rewritten by the codemod** (it walks the form tree). Any prose still saying
   "peer is dead" is a separate manual pass — say which you did.
5. **`cargo build --release` does not compile tests.**
