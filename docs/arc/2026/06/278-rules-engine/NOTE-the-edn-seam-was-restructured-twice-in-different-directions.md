# NOTE — the EDN seam was restructured twice, in different directions

**Found:** 2026-08-29, on `claude-compute` (main × grok-rete), refreshing across 40 main
commits and 45 grok-rete commits.
**Status:** DECIDED, not open. Only the timing waits — see *When*.
**Sibling:** `NOTE-rete-modules-is-a-hand-computed-cache-of-a-derivable-set.md` waits on the
same trigger. Different disease, same moment.

---

## What collided

> ⚠ **CORRECTED 2026-08-29, by probe rather than by reading.** This note first said "23 of 26
> conflicts, ONE cause: the edn seam". That is wrong in SCOPE. A throwaway probe branch —
> merge grok-rete, take its side on the seam, count the damage — produced 160 build errors of
> which exactly ONE mentioned edn. The rest were E0603/E0433/E0583/E0432: module resolution.
> The edn seam is the most legible instance of a much larger divergence, recorded below under
> *The real scope*. The original framing survived a careful read of the conflict hunks and died
> in ten minutes against a build, which is the argument for probing rather than reasoning.

23 of 26 conflicted files, one VISIBLE cause. Both branches restructured EDN rendering; neither
module exists on the other side:

| | commit | change |
|---|---|---|
| main | `d43f75887` HOME-8 strike 1(255) — *the VSA algebra leaves runtime.rs* | rendering moves to `src/edn/render.rs`. **Structural** — where the code lives. |
| grok-rete | `5696835f1` — *`edn::write` reports instead of aborting* | `src/edn_shim.rs`; writing becomes **fallible**. **Semantic** — how it fails. |

This is not one change under two names. Location and failure-behaviour are orthogonal, which
is exactly why the merge cannot pick a side: taking either loses something real.

## The real scope — main relocated its module tree; grok-rete built on the old one

Arc 255's HOME campaign did not move one seam. It moved a flat module layout into directory
homes, and grok-rete's 45 commits target the layout it replaced:

```
string_ops.rs                    ->  src/string/   (1 file)
wat_edn_bridge.rs + edn_shim.rs  ->  src/edn/      (6 files)
hologram.rs + sigma.rs           ->  src/holon/    (6 files)
stdlib.rs                        ->  src/host/     (4 files)
```

main declares 45 modules, grok-rete 57. Four exist only on main (`edn`, `holon`, `host`,
`string` — the new homes); sixteen only on grok-rete, and most of those are the OLD homes plus
genuinely new work (`alloc_counter`, `sandbox`, `harness`, `compose`). A union of the two lists
is NOT the resolution — it would declare both the old and new homes for the same code.

**This is the same situation the builder named at the outset, one level up from names:**
grok-rete is not yet ready to take main's renames, and the fight is necessary until everyone is
on the new ones. It is now module TOPOLOGY, not spelling — which is why no codemod helps and
why `wat-drift` reports clean while the merge is unbuildable.

## grok-rete's half is a defect fix, not a refactor

Builder: *"if you can fix this now - do it."*

> `value_to_edn_with` returned a bare `OwnedValue` and `panic!`ed in its holon arm on a value
> it could not tag — **reachable from a two-line wat program**, `(:wat::edn::write #holon [1 2 3])`.
> … The failure channel already existed: `eval_edn_write` has always returned
> `Result<Value, RuntimeError>`. Only the callee could not express failure, so the failure had
> nowhere to go but the process — and the error it discarded was ALREADY a located
> `TypeMismatch`. **The panic stringified a good diagnostic and then aborted.**

The conversion is careful, and the care is the reason it must not be discarded wholesale:
three kinds of call site ruled separately; a NAMED lossy door (`value_to_edn_string_lossy`)
for structurally-infallible callers, whose own doc says it is not for ordinary callers; one
site that **refused** the lossy door because an `#wat.edn/Unencodable` marker on the wire is
worse than failing; three sibling panics deliberately left alone with reasons.

## The four questions

**(a) port grok-rete's fallible API onto main's relocated module.**
Obvious YES — renders where main put it, reports like grok-rete made it; both properties
independently motivated. Simple YES — one module, one job, done fallibly; location and failure
are orthogonal, not braided (23 files is effort, not complexity). Honest YES — preserves the
located diagnostic the panic discarded, keeps the named lossy door and the one site that
refused it. Good UX YES — a two-line program stops killing the process. **All four.**

**(b) take main's, drop the fallibility.**
Obvious **NO** — the result `panic!`s on user-reachable input while an unused failure channel
sits one frame up. Honest **NO** — it stringifies a located diagnostic and aborts the process.
**Disqualified; UX not weighed.**

**(c) "hold, three options, decide later" — as first framed by claude-compute.**
Honest **NO**, and worth recording as the error it was: (b) is disqualified on evidence, so
there were never three live options. Presenting a settled question as open would have handed
the builder a decision the evidence had already made.

**(c′) "do (a), scheduled at the sync."** All four YES. This is the ruling.

## When — the same trigger as the sibling note, for a DIFFERENT reason

The sibling waits for the sync because both branches edit both lists. This one waits because
**main has not finished moving the seam**:

```
d43f75887  HOME-8 strike 1(255): the VSA algebra leaves runtime.rs
fb0cdb192  HOME-8 strike 2(255): the VSA surface is registered — 95 verbs
04345f5d9  STONE HOME-11(255): EDN gets a REGISTRY home
a88f5d634  DRAWN(255): HOME-12 — the AST surface gets a registry home
```

> ⚠ **CORRECTED 2026-08-29 (second correction to this note).** The paragraph below argued
> main was "still relocating" the seam because HOME-12 was DRAWN. **It had already landed**
> (`eb790da4a`), as had HOME-13, and the last HOME stone sits 98 commits back while main works
> arc 109 ONE PARAM-SPEC. The campaign is DONE and the module tree is settled. The reason to
> wait is NOT main; it is that **grok-rete has not taken main's module tree**, so the port must
> be redone at every refresh until it does. Same three-way-sync trigger as the sibling note,
> and the trigger sits on grok-rete's side.

HOME-11 gave EDN a registry home AFTER HOME-8, and HOME-12 is drawn. Porting the `Result` API
onto `src/edn/render.rs` today ports it onto a target main is still relocating — the work
would be done twice, and the second time against a hand-port that rerere cannot replay.

Do (a) once main's HOME campaign settles where EDN lives. Until then `claude-compute` stays
current with main and holds grok-rete, which is recorded on the branch rather than implied.

## What this costs while it waits

The integration branch does not reflect grok-rete's last 45 commits, so it is not currently
showing the union — which is its whole job. That is the price of the hold and should be
stated plainly rather than absorbed: **a green claude-compute right now means "main is green",
not "the union is green."**
