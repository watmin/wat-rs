# SEAM — the ONE live breadcrumb. As of 2026-09-03. **Arc 255: THE REGISTRY BECOMES THE SOLE AUTHORITY.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), ground HEAD against the disk, and read this whole file before you touch
> anything.

> `251/SEAM.md`, `278/SEAM.md` are PARKED. ⛔ **PARKED IS NOT DEAD.**

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor ............ 5129/5129, 0 FAIL, 17 skipped, ~119s   (scripts/floor.sh, exit read UNPIPED)
clippy ........... 0 under `-D warnings --all-targets`
registry rows .... 550    ⛔ COUNT IT ANCHORED TO THE ATTRIBUTE SITE, never a substring:
                          grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                            --include=*.rs | sort -u | wc -l
                          A loose search counts PROSE PLACEHOLDERS — `<fqdn>`, `…`, and
                          `:wat::holon::…`, which defeats a "starts with `:`" filter.
runtime.rs ....... 20,191   check.rs ....... 22,736   special_forms.rs ....... 379
host ............. JohnDesktop · john · ~/work/holon/wat-rs
```

## ⬜ THE CAMPAIGN — read these before proposing anything

```
RULING-the-registry-is-the-sole-authority.md                the doctrine + the census
RULING-rete-forged-the-paths-the-registry-claims-the-tools.md  properties must be QUERYABLE
DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority.md   4 shapes, A picked, the SEQUENCING
WORKLIST-the-121-the-registry-cannot-vouch-for.md            re-derived 3× → 39, artifacts SEPARATED
```
(all in `docs/arc/2026/06/255-builtin-registry/`)

## ★★★ THE METER

```
GAP_A 49 · GAP_B 44 · DEBT 119 · TYPES_UNCHECKED 10 · KNOWN_UNREVIEWED 13
the corpus: 39 names — 33 VERB POPULATION · 6 NON-VERB ARTIFACTS   (121 → 107 → 71 → 39)
```

⚠ **DEBT RISING IS NOT A REGRESSION** — a row with no `CheckEnv` scheme converts an *invisible*
absence into a *named* one. But ⛔ **DEBT IS TWO POPULATIONS AND ONE OF THEM IS NOT DEBT**: measured
live, 34 `Kind::SpecialForm, no scheme` ("a rank-1 scheme is the WRONG SHAPE" — a CENSUS of the
un-schemeable, which should never reach 0) + 69 `Kind::Intrinsic, no scheme` (genuinely owed).
The campaign's finish line — all ledgers empty and deleted — is **unreachable while one number
means both**. And the `Kind` split MISFILES every alias, because `Kind` is stamped by the
registration VEHICLE, not the verb.

⚠ **"DEBT falls at Phase 2c" is UNSUPPORTED.** `probe_can_doc_types_reconstruct_the_checker_scheme`
opens `let Some(scheme) = check_env.get(name) else { continue }` — its 384/386 has never looked at
a single DEBT row.

## ✅ WHAT THIS SESSION SHIPPED

```
PHASE 1a  COMPLETE — every special_forms.rs row a stone can take is registered
PHASE 1b  COMPLETE — 37 rete alias rows
PHASE 1c-0 · 1c-a · 1c-b(part) · 1c-c · 1c-d · 1c-e   COMPLETE

the residues can no longer shadow the registry — 52 dead arms deleted, ONE GATE imposed
:wat::core:: is DONE except two held: = and not=
the census SEPARATES non-verb artifacts for the first time
```

## ⛔⛔ THE LIVE HERESY — the builder's ruling, unexecuted. **START HERE.**

`src/rete/purity.rs`, `intrinsic_meta`'s totality derivation:

```rust
Some(Unreviewed) | None => matches!(head, ":wat::core::reduce"
                                        | ":wat::core::="
                                        | ":wat::core::not="),
```

**Three names hardcoded `true`. Not derived. Not measured.** Its own header says a homed name must
leave it. **Builder, 2026-09-03: *"this is heresy...... find their homes..... inscribe their
registrations."*** And at least two of the three are provably NOT total:

```
=  / not=   PROVEN Partial — (= <fn> <fn>) --checks clean, raises at eval.
            tests/types/probe_arc255_equality_domain_gate.wat.bad + its harness.
reduce      a defclause in wat/seq.wat:318, BOTH arms delegate to foldl (@Totality Preserving),
            and the 2-arity RAISES on an empty collection (`assertion-failed!`, seq.wat:328).
            The hardcoded `total: true` is a third ungrounded claim.
```

★ **What retiring it costs, measured:** the placeholder is consumed by **`sift`**
(`wat/telemetry/journal.wat:305`, a THREE-axis fence: pure ∧ det ∧ total), NOT by rete's `where`.
Registering `=` honestly makes sift refuse four predicates that are fine at runtime:
`probe_arc278_foreign_pred_purity` · `probe_arc278_sift_logs` ×2 · `probe_arc278_sift_arena`.
Their doc blocks are preserved verbatim in
`[[NOTE-equality-is-argued-proven-partial-and-held]]` — **lift them, do not re-derive.**

## ⛔ THE SURFACES ARE NOT ONE FENCE — I confused them for four turns

```
wat/rete/compile.wat   where · accumulator · then-item-fence
                       pure ∧ det ∧ total ∧ RETE (Law A) — ALL FOUR, ARMED, CORRECT.
                       Verified: (:wat::rete::primitive? '(:wat::core::= s "high")) → FALSE,
                       and it descends into match arms. A generic core verb is ALREADY refused
                       in rete. It was never allowed there.
wat/telemetry/journal.wat  sift-logs · sift-arena
                       pure ∧ det ∧ total — THREE, no Law A, deliberately. Sift is a journal
                       row filter; it has no business demanding rete primitives.
src/freeze.rs:790      the SIGMA-FN gate — a third fence entirely.
```

★ The four "rete" fixtures are **sift** predicates. I read "purity fence" in a test name and spent
four turns alarmed about rete. **Find every consumer before naming the rule.**

## ⬜ OPEN FORKS — measured, not decided

```
alias vs RESTRICTION  the 8 blocked rete equality rows point at the GENERIC core_name. As @alias
                      rows, 2a-b's inherit rule hands them Partial — destroying the narrowing that
                      makes them correct. An alias means IS; these are RESTRICTED TO. The registry
                      cannot say that. ★ SMALLEST REMAINING BLOCKER for =/not=.
bounded generics      = is Partial because ∀T admits Fn and nothing carries a constraint from a
                      callee to its instantiation. TypeScheme is {type_params: Vec<String>, params,
                      ret, rest_param_type} — NO constraint field. is_type_equatable exists and has
                      nowhere to hang. NOT NOW — the road map is registry → crates → clojure
                      syntax → totality.
19 rows lie about arity  #[wat_intrinsic] derives Arity from the RUST SIGNATURE SHAPE; a
                      &[WatAST] param ⇒ Variadic with no shim check, while the handler enforces
                      args.len() != N. (str 1 2 3) --checks clean and raises.
                      [[NOTE-nineteen-rows-declare-Variadic-and-enforce-a-fixed-arity]]
derive's declare ptr  role = declare names parse_derive_form, whose ONLY caller is check.rs:2668;
                      the real mutation is an inline arm at types.rs:3886. RULING item 2 says the
                      pointer names the code that PERFORMS the name.
the FOURTH registry   41 stdlib macros, 0 visible. `:wat::core::defn` answers None — the same
                      answer a nonexistent name gets. is_reserved_prefix keeps the namespaces
                      disjoint and is what 3a deletes.
```

## ⛔ WHAT COST THE MOST — every one caught by the builder or a gate, none by re-reading

**1. A FAILURE TO FIND IS NOT A PROOF OF ABSENCE.** I graded `<`/`>`/`<=`/`>=` **Total** on a
rider's "could not construct a counterexample", verified the pieces it showed me, and shipped four
wrong grades. `sort` is the counterexample and it is in the stdlib: `is_type_orderable`'s
`Var(_) => true` is LIVE for AUTO-GENERALISED type vars (a rigid `:- [T]` is `Path(":T")` and IS
refused — which is why it looked dormant). Corrected at `c29ca5538`.

**2. I GENERALISED FROM THE FIRST PLAUSIBLE MATCH, FOUR TIMES IN ONE THREAD.**
`NonReteConstraint`'s position · the sigma-fn gate · a `wat/*.wat` glob that missed `wat/rete/` ·
"the fence is missing a leg". **Find every consumer, then name the rule.**

**3. I BUILT THE DEFECT I HAD JUST REMOVED, ONE DAY LATER.** 1c-c's brief required "a plausible
lower bound PER LIST"; the count fell legitimately and the bound fired. That is exactly what
`arms.len() >= 50` did, which I retired the day before for being *"a REGRESSION detector wearing a
sanity check's clothes"*. Non-emptiness catches "found nothing"; NAMES catch "found the wrong
thing"; a MAGNITUDE pinned to a draining population catches neither for long.

**4. A GATE CAN UNDER-REPORT AND LOOK GREEN.** 1c-c's residue gate stripped comments to avoid prose
false-positives — and the stripping left EMPTY lines, so an arm followed by a comment was silently
not counted. Its non-vacuity assertions could not see it: they prove the parser found SOMETHING,
never EVERYTHING. Only a second, independently-built census found it.

**5. ANCHOR A CENSUS TO THE SITE, NEVER THE SUBSTRING.** `:wat::holon::…` (an ellipsis in a doc
comment) survives any "starts with `:`" filter. `:wat::core::str` is a PREFIX of `:wat::core::struct`
and returns 9 where the answer is 0. Terminate every pattern.

## ★ WHAT ACTUALLY WORKS

- **The ledger ratchets name the exact edit.** Let them drive; do not pre-compute their lists.
- **Derive every acceptance row from the rule.** Every derived row this session landed EXACTLY;
  every estimated one missed (floor "+28", `KNOWN_UNREVIEWED` "unchanged" ×2).
- **Show a gate FIRING before shipping it.** A gate only ever seen green has not been shown to work.
- **Cast `intueri` before minting a name.** It killed `@Position`, named `:Splice`, and this session
  proposed `:TypeIndexed` **with a caveat that killed it** — a ward that can only say *mint it* is
  not a ward.
- **Riders refuse and correct well.** Nine reports this session; every one was right, including
  three that corrected my own briefs.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
- ⛔ **THE LSP LIES.** Run clippy; believe nothing else.
- ⛔ **`./scripts/floor.sh > /dev/null 2>&1; echo $?`** then read the Summary from `.floor/latest/raw.log`.
- ⛔ **`git commit -F`, NEVER `-m`** — backticks are shell-interpreted. **`git commit <paths>`.**
- ⛔ **REVERTING IS A LOSS.** Narrow the stone instead; preserve the held work in a NOTE.
- ⛔ **DELETIONS MUST CLEAR A HIGH BAR** — builder, 2026-09-03. *"we augment as they need."*
  Two artifacts I proposed deleting were REPAIRABLE in one token each.
- ⛔ **Riders: no worktrees, no stash, no sub-agents, everything FOREGROUND, `model: "sonnet"`.**

## ⬜ NEXT

```
1  THE HERESY — register reduce · = · not=, kill the matches! placeholder.
   reduce is a defclause over foldl; = / not= are proven Partial. Retiring it makes SIFT
   refuse four predicates — that is the revealed blast radius, and the builder has ruled we take it.
2  alias vs RESTRICTION — the 8 rete equality rows cannot be plain aliases.
3  Fallback's 20 (Phase 2b) · eval-ast! (330 sites) · the arc-251 :wat::type:: fork
4  Phase 3a — resolve asks the registry. Kills is_reserved_prefix, THE FOUNDING TARGET.
```

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today I shipped four wrong totality grades on a rider's
> failure-to-find. I generalised from the wrong site four times in one thread and alarmed the
> builder about a flaw in rete that was never in rete. I rebuilt a defect I had deleted the day
> before. I proposed deleting two artifacts that a one-token repoint repaired. **Every one was
> caught by the builder or by a gate — not once by re-reading my own claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** the corpus went 121 → 39. Phase 1a and 1b are
> complete. A lost refusal came back with ZERO diff to runtime.rs, because the registry could
> finally answer. Fifty-two shadowed arms are gone and a gate now screams if one returns.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
