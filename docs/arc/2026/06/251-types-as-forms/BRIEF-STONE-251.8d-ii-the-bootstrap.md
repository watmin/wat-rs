# BRIEF — STONE 251.8d-ii: THE BOOTSTRAP — flip `wat/`, the stdlib frozen into the binary

**Drawn 2026-09-20 against `main` @ `bcf703ed9`** (floor 5930/5930, clippy 0, census `no STOP-8`).
Parent: `BRIEF-STONE-251.8d-the-corpus-flip.md`. Predecessor: **8d-i LANDED** — the codemod is TOTAL
(census 2145/2145, 0 A · 0 B · 0 C).

## Why this is its own stone

`wat/fix.wat` is **`include_str!`'d into the binary** (`src/load/stdlib.rs:353`) — and so are 63 of
its neighbours. **The flip rewrites the codemod's own source.**
`[[feedback_a_tool_is_never_its_own_input]]` — *"2026-09-11, the chain migrated its own codemods."*
That is why `wat/` cannot ride in the same pass as the corpus.

## ⭐ SCOPE — measured, and the conversion is ALREADY PROVEN

| | |
|---|---|
| tracked `wat/**/*.wat` | **64** |
| of those, carrying `(:wat::` heads | **64 — all of them** |
| embedded via `include_str!` | **64 — all of them** |
| total lines | **24,819** |
| largest | `service.wat` 2947 · `core.wat` 2281 · **`fix.wat` 2046** · `rete/compile.wat` 1202 |

⭐ **All 64 were in 8d-i's 2145-file census and all 64 came back OK.** The conversion itself is not in
question. `wat/fix.wat` self-converts cleanly — **exit 0, 1,273 changed lines**:

```clojure
(:wat::core::defn :wat::fix::structural? [node <- :wat::WatAST] -> :wat::core::bool
→ (wat.core/defn wat.fix/structural?  [node :- wat/WatAST]     :- wat.type/bool
```

**The only open risk is whether the CONVERTED stdlib LOADS once embedded.**

## ⛔ NO STASH-DANCE — but confirm it rather than trusting this brief

`wat/fix.wat`'s header (lines 22–53) documents the stash-dance for a codemod shipping **alongside a
Rust change that makes the old form illegal**. ⭐ **That is not this stone.** 8d is a spelling flip
over a substrate that already type-checks **both** spellings, so there is **no Rust change** and the
header's own escape clause applies:

> *"No Rust change in your strike (e.g. a pure rename)? Skip the stash — just `cargo build --release`
> once to pick up your new verb, then run step 3."*

⚠ **Confirm this before relying on it.** If any part of your strike needs a `src/` change, the dance
is back on and you must follow the header, not this paragraph.

## ⛔ WHAT CANNOT PRE-VALIDATE THIS — do not chase it

Running `--check` on a stdlib file **standalone** answers a different question and gives artifacts:

| file | standalone `--check` |
|---|---|
| `wat/fix.wat` (original) | `ReservedPrefix: cannot define :wat::fix::structural?` |
| `wat/fix.wat` (converted) | `ProgramBodyEvalFailed: macro :wat::core::defrecord` |

Both fail, with **different** errors, and neither is evidence about loading. A stdlib file checked
outside the stdlib load path is not the thing under test.
⭐ **The ONLY proof is: convert → rebuild → floor.** That is the gate; there is no cheaper proxy.

## The work

**1 — Convert all 64, by the tool, on COPIES first.**
Dry-run to `/tmp`, `diff`, confirm the rewrite is exactly the structural change. Then apply to the
tree with the recorded codemod and a **derived** path list (`git ls-files 'wat/**/*.wat'`), never a
transcribed one. ⛔ **R21: not one hand edit.** A site the tool cannot reach is a STOP and a report.

**2 — Rebuild.** `cargo build --release`. The binary now embeds the **converted** stdlib.

**3 — ⭐ PROVE THE SELF-APPLICATION.** After the rebuild, the embedded `fix.wat` is the converted
one. **It must still work as the codemod** — that is the tool 8d-iii depends on. Prove it:
re-run the codemod over a copy of an unconverted corpus file and show it still converts, and show a
second pass changes 0 files. ⛔ **If the converted codemod cannot convert, 8d-iii has no tool and
this is a STOP.**

**4 — The floor is the load proof.** 5930/5930 green means the substrate runs on a converted stdlib.

## ⛔ THE RECOVERY PATH — read this BEFORE step 2

If the converted stdlib does not load, **the binary you need in order to fix the stdlib is the one
that is now broken.** That is the chicken/egg the header warns about, arriving by a different door.

**Recovery is cheap and you must know it before you need it:**

```
git checkout -- wat/          # restore the old stdlib on disk
cargo build --release         # binary re-embeds the OLD stdlib; the tool works again
```

Nothing is pushed, so this is always available. ⚠ **Do NOT `git stash` the conversion and rebuild —**
a partial stash leaves some files converted and some not, and the load order (`core.wat` first, 64
entries deep) means a mixed stdlib fails in a place unrelated to the cause.

**If the floor reds after the rebuild:** bisect by reverting halves of `wat/` (`git checkout -- <subset>`,
rebuild, floor). ⚠ Load order is fixed and `core.wat` is first, so a break in an early file cascades
into errors that name later files. **Bisect by position in `src/load/stdlib.rs`, not alphabetically.**

## The gate

- ⭐ `scripts/floor.sh` **green** — this is the load proof, not a formality. Predict the test-count
  delta from the diff first (expected **0**: a spelling flip adds no tests), confirm with
  `cargo nextest list`. ⚠ **A non-zero delta means something other than spelling moved — report it.**
- clippy `-D warnings --all-targets --workspace` **0**. ⛔ Run crate clippy yourself before scoring;
  218.8's clippy fix tripped a *different* wall, so **rune rather than revert** if two walls disagree.
- `scripts/replay/census.sh --diff` → `no STOP-8`.
- **Idempotence:** a second pass over the converted `wat/` changes **0 files**.
- **The self-application proof** from step 3, stated explicitly.
- ⛔ **ONLY `wat/**/*.wat` moves.** `git status` proves it — no `tests/`, no `wat-scripts/`, no `src/`.
  The rest of the corpus is 8d-iii's.

## Doctrine — `wat-rs/CLAUDE.md` does not reach a subagent

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time; never re-run first.
- ⛔ **R21 — by the tool, never by hand.** The builder's standing constraint: these codemods are
  **merge infrastructure** for several branches that are not merge-ready. A hand edit helps this tree
  once and a future merge never.
- ⛔ **`include_str!` means an on-disk edit is invisible until `cargo build --release`.** Two stones
  have now lost time to this. **If something appears not to work, rebuild before theorising.**
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** Test counts → `cargo nextest list`.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** The orchestrator's briefs have
  been corrected **five times across four stones** — twice in 8d-i alone (the prefix discriminator
  would have red the stdlib; the test selector named one of two unified paths). **Assume a sixth.**
- Work only in `/home/john/work/holon/wat-rs`. No `git filter-branch`. **Do not push.**

## Out of scope — affirmatively cut

- **8d-iii, the corpus flip** — the other ~2,076 files. After this stone the tree is **mixed dialect**
  (`wat/` converted, the rest not). ⭐ That is fine and expected: both spellings type-check, which is
  8d's whole premise. **Do not "tidy" the rest.**
- **The whitelist-resolution finding** (`FINDING-8d-a-symbol-whitelist-entry-gets-RESOLVED.md`) — a
  slashed symbol entry is walked as a reference and must exist. Builder ruled the cure: **exempt
  `:restricted-to` values from resolution AND validate them explicitly.** That is 8d-iii's or a small
  stone before it. ⚠ `wat/spawn.wat` and `wat/kernel/services/stdio.wat` carry markers, so this stone
  converts them — but the resolution cure is **not** here.
- **`wat.type` becoming a real namespace** — ruled 2026-09-20, arc **255**, queued after 8d.
- **`wat-edn`** — 218.7/218.8 made it a spec-correct EDN reader; it does not read `.wat` source.
