# SEAM — the ONE live breadcrumb. 2026-09-08 (late). **GREEN · CLEAN · PUSHED · NO PEER IN FLIGHT.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS.

```bash
git status --porcelain          # expect EMPTY
git log --oneline @{u}..HEAD    # expect EMPTY (0 unpushed)
grep -aE "^ +Summary" .floor/latest/raw.log
```

```
floor ... 5277 tests run: 5277 passed, 21 skipped   FLOOR EXIT=0   HEAD 927d94d2b
```

## ⛔⛔ THE PEER CHANNEL IS OUT OF CREDITS. DELEGATE LOCALLY.

Builder, 2026-09-08: *"we need to delegate to opus/sonnet locally - out of credits."* **`pulsare_yield`
is no longer the channel.** The BRIEFs transfer unchanged — they are already written tier-agnostic
(*"you edit and report; the orchestrator runs the floor centrally, once"*), which is FM 18/19's rule.
What changes is the spawn:

```
model: "sonnet"     EXPLICIT on every Agent call. FM 12 — omit it and the spawn silently inherits
                    OPUS while every document says sonnet. Nine agents shipped that way once.
isolation           OMIT IT. Never "worktree". FM 7-bis, doctrine not preference.
cwd                 anchor /home/john/work/holon/wat-rs absolutely in the prompt; any path with
                    `.claude/worktrees/` is harness state and illegal to operate on.
no tool preamble    FM 16 — mentioning Bash/cargo availability AT ALL triggers the hallucinated
                    denial. State the work; trust the tools.
the rider           runs targeted probe binaries + per-fixture `--check`. NEVER the floor.
```

⚠ A local rider spends THIS session's context, not a separate budget. Weigh what is delegated
against what is a genuinely small inline fix.

⚠ **`scripts/floor.sh` DOES NOT RUN CLIPPY.** Every "clippy N" in this record came from an ad-hoc
`cargo clippy | grep -c` in the orchestrator's command line, and across three invocations that grep
returned **0, then 7, then 8** for a tree whose real answer never changed. The honest gate is
`cargo clippy --release --all-targets -- -D warnings`; it EXITS 101 on **5 pre-existing `dead_code`
items** that are already on `origin/main`. They are a `purgare` stone, not a regression.

## ★★★ WHAT LANDED — the type authority is one authority, and so is assignability

```
296 P-1     an annotation may not name a type that does not exist. It caught THREE real phantoms
  +RELAND-1 nothing could see before: :wat::core::Int (16 sites), ::Keyword (7), and
            :wat::kernel::ExitCode — RETIRED by arc 170 on 2026-05-10 and still annotated.
            Both `is_reserved_prefix` blankets deleted; the union is FOUR stores.
296 P-2prq  `use!` seeds its imported name into TypeEnv, so is-type? agrees with resolve.
296 P-3     the wall asks each DECLARING SCOPE about its own use!; is-type? asks subtype_edges.
            MEASURED, both directions:
              no user use!    resolve check=1 · wall check=1 · is-type? false
              with user use!  resolve check=0 · wall check=0 · is-type? true
296 A-1     `if`, `send`, `try-send` decide fit the way a PARAMETER always has. Same pair of
            types, both positions, same answer. is_subtype count unchanged — assignable reused.
296 P-1b    a PARAMETRIC annotation's HEAD is validated. It was not: P-1 reused a walk built for
            free-VARIABLE collection, where skipping a head is correct, and inherited its blind
            spot. `(:usr::MadeUp :- [i64])` checked CLEAN while the bare form refused.
```

## ⛔ P-2 IS NOT BLOCKED ON VARIANCE — THAT WAS MY HASTY RETREAT, AND THE BUILDER CAUGHT IT

The previous seam said *"blocked on VARIANCE, a type-system decision nobody has made."* **Measured
false.** 299 errors was a WORKLIST, not a wall (FM 15, which I had just written into this file).
What the measurements actually found:

```
the erasure          src/declare/register.rs  `ret_type: enum_type.clone()`  — ONE LINE.
                     `infer_enum_map_ctor`'s fallback is NOT the live path; a registered SCHEME is.
head-level edges     ALREADY WORK: (derive Child Parent) → a Parent<i64> slot accepts Child<i64>,
                     and args-differ is correctly refused. A VARIANT edge is head-level, so A-2
                     needs NO new assignable arm. My "one remaining unknown" did not exist.
{:keys} on a variant NOT free. It asks `TypeDef::Aggregate`. ⛔ The builder REFUSED my fix of
                     registering variants AS aggregates — "why are we forcing enums to be
                     aggregates?" — that is shaping the TYPE to fit a PREDICATE. The predicate is
                     what is too narrow, exactly as Stone O's `nature == Struct` was.
the builder's match  ALREADY GREEN today. Only the annotation and the ctor are subjects.
```

## ⛔⛔ P-2 IS NOT UNBLOCKED. THE PREVIOUS SEAM SAID "4/4, UNBLOCKED" — THAT IS SUPERSEDED.

`296/NOTE-P2-is-blocked-on-VARIANCE-and-my-fence-was-on-the-wrong-axis.md`. Struck as P-2a,
**reverted**; the rider's work is preserved at `7292a2c2a`, the revert is `a6641ab45`.

```
the ctor half   299 type-check errors, two clusters:
  cluster 1     infer_send_prime (check.rs:11619) UNIFIES payload against I and never calls
                assignable — while ordinary parameters DO (check.rs:16992).
                TWO ARGUMENT POSITIONS, TWO RULES.
  cluster 2     RecvOutcome :- [ScanResponse::RequestTooLarge] vs RecvOutcome :- [ScanResponse].
                TYPE ARGS ARE INVARIANT. `Variant <: Enum` does not lift through a container.
the reg. half   cannot stand alone: the annotation is accepted and NOTHING satisfies it —
                "(:user::takes-red (:usr::Colour::Red {:shade 7})) -> expects Colour::Red; got Colour"
```

**P-2 needs, in order: (1) one rule for argument positions · (2) a VARIANCE story · (3) then the
ctor stops erasing.** (2) is a type-system decision nobody has made. It is the builder's.

## ⛔ QUEUED

```
A-2  a variant is a type        NEXT. Fixtures already written (untracked, tests/types/
                                probe_arc296_A2_*.wat) FROM THE BUILDER'S OWN EXAMPLES. Room mapped:
                                  register.rs `ret_type`      the ctor stops erasing
                                  {:keys} predicate           widen by SHAPE, not nature
                                  defclause routing           falls out of the 72-site router
                                Baseline: process-full-box check=1 · posterity ctor check=1 ·
                                match check=0 (already green).
the :wat::* call-head blanket   src/resolve/walk.rs:272. Arc 255's founding defect. Gates the dot
                                flip. ⚠ MY 92%/470 CENSUS WAS A DIFFERENT EXPERIMENT than 255's
                                17%/35: they REPLACED the blanket with a registry lookup, I deleted
                                it and put nothing there. The real finding: `is_resolvable_call_head`
                                has NO path to the intrinsic registry at all — which is 255's
                                founding sentence, measured.
extend-type full-spelling       a DECLARED edge between two parametric types is ignored:
                                `(extend-type (Child :- [i64]) (Parent :- [i64]))` then a
                                Parent<i64> slot REFUSES a Child<i64>. Head-level edges work.
                                Real defect, off A-2's path.
derive's MARKER                 `(derive :usr::A :usr::Typo)` mints a new marker silently.
5 dead_code items               pre-existing, `-D warnings` only. A purgare stone.
```

## ⚠ RULINGS — do not re-litigate

- **An enum ctor is a MAP.** declare `:None []` · construct `(…::None {})` · match `[…::None {} b]`.
- **`{:keys}` is for ONE-SHAPE aggregates; match is for many-shape.**
- **Natures differ in PURITY, not SHAPE.**
- **`type-of` is STRUCTURE; `is-type?` is MEMBERSHIP** — and membership is over the UNION, which
  turned out to be four stores, not two. `HAERESIS EST ITERVM ROGARE`.
- **⛔ SIDE BRANCHES DO NOT SERVE US** · **COMMIT LOCALLY OFTEN; PUSH ONLY GREEN.**
- **`git commit <paths>` — NEVER `git add` then commit.** Six occurrences of sweeping a peer's
  in-flight work into a mislabelled commit; the sixth was the fix for the fifth.
- **Colon-quoted symbols are ACCEPTABLE transitionally.** `:wat::core::+` IS `wat.core/+`.

## ⛔ THE FAILURE PATTERNS — all four fired again today

**① A NAME CHECKED AGAINST A PARTIAL SET.** Now at SIX positions. Every stone this session was
this defect at a new address, and the cure for one exposed the next.

**② I MEASURE THE DECLARATION AND NOT THE CONSUMERS.** P-2a's fence held perfectly and described
the wrong boundary — I fenced generic ENUMS out, and a MONOMORPHIC variant hit the same class by
being wrapped in a generic container.

**③ A PROBE THAT ANNOTATES IS NOT A PROBE THAT USES.** P-2a's five rows all passed in exactly the
state the DESIGN called "strictly worse than today's refusal." **The rider found it, not me.**
A probe that proves a type is USABLE must construct a value INTO it.

**④ AN INSTRUMENT ANSWERING A NARROWER QUESTION.** `| tail -6` on a 27-failure floor; a JSON grep
against EDN output returning a confident 0; `pgrep -f "cargo"` matching `~/.cargo/bin/wat --mcp` and
reporting a phantom build TWICE, which I then wrote onto disk as the reason a measurement went
unrun. **Run the gate unpiped and read `$?`; read `git show --stat` BEFORE writing the message.**

★ **THE RIDER CAUGHT ME THREE TIMES TODAY** — a mislabelled commit, a probe that could not fail,
and a design whose contract its own strike disproved. Weigh its report against the disk, and read
its "what surprised" section: that is where the findings are.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⛔ **GREEN AND QUIET IS THE MOST DANGEROUS STATE THIS FILE DESCRIBES.** `git status` first.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
