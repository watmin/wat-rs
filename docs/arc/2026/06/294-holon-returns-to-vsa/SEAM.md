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
floor ... 5267 tests run: 5267 passed, 21 skipped   FLOOR EXIT=0   HEAD a6641ab45
peer .... idle. Last exchange: P-2a scored, then REVERTED by me.
```

⚠ **`scripts/floor.sh` DOES NOT RUN CLIPPY.** Every "clippy N" in this record came from an ad-hoc
`cargo clippy | grep -c` in the orchestrator's command line, and across three invocations that grep
returned **0, then 7, then 8** for a tree whose real answer never changed. The honest gate is
`cargo clippy --release --all-targets -- -D warnings`; it EXITS 101 on **5 pre-existing `dead_code`
items** that are already on `origin/main`. They are a `purgare` stone, not a regression.

## ★★★ WHAT LANDED — the type authority is now one authority

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
the :wat::* call-head blanket   src/resolve/walk.rs:272. Arc 255's founding defect, still standing
                                for CALL HEADS. Gates the dot-notation flip. STILL THE HEAD.
derive's MARKER                 `(derive :usr::A :usr::Typo)` mints a new marker silently.
                                Declared-first vs open-minting — the builder's ruling.
5 dead_code items               pre-existing, `-D warnings` only. A purgare stone.
P-2                             behind variance, above.
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
