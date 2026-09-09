# SCORE — the topic forgets its subscriber count

**SCORED.** Executor: claude, 2026-09-09. Did not commit. **No STOP fired.** Seven `.wat` files
migrated by one recorded codemod, `wat-scripts/fixes/topic-record-drop-nsubs.wat`. **21 edit sites**,
all of them found by the codemod's own finder, none by hand. **No `wat/`, no `src/`, no `mem.wat`, no
store.**

```
     Summary [ 470.395s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T09-00-34Z/` — `scripts/floor.sh` exit `0`, **no `ARM.txt`**, and
`grep -cE 'FAIL|TRY|TIMEOUT|ABORT|SIGSEGV' clean.log` → **0**. **5237** — unchanged, none added or
removed. `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`
**PASS [470.388s]**. Run **once**, after the migration was complete. **Nothing was re-run.**

---

## ⭑ THE HEADLINE — the finder found 21 sites, and the corpus contains no other `:nsubs`

`wat --grep` was run over **every `.wat` file in the repository** — 1809 paths, `find . -name '*.wat'
-not -path './target/*'`, in 19 chunks of 100, **all 19 exit 0** — not over the seven files the
DESIGN's exemplars named. Two facts fell out that a file-list census could not have established:

★ **`token-nsubs-keyword` and `record-kwarg-nsubs` return the SAME 20 sites.** Set difference in both
directions is empty. So in 1809 files there is **not one `:nsubs` keyword that is not a
`:demo::topic::Record` kwarg** — no `:nsubs` on another record, none in a map, none in a quasiquote.
The form filter and the token census agree, which is the strongest statement available that form 1 is
complete.

★ **`token-nsubs-symbol` returns 16 sites; the binder rule claims exactly 1.** The other **15 must
survive**, and they did. The DESIGN's trap list named **6** of those 15.

---

## THE CENSUS — before any file was rewritten

Run at HEAD `f9d1684ea`, tree clean, **before** the applier touched anything. Classified from the
raw `#wat.grep/Match` lines (kept at `census-full.txt`; a verbatim sample is at the bottom of this
section).

```
== match count by rule ==
    1  durable-binder-nsubs
    1  record-accessor-nsubs
   20  record-kwarg-nsubs
   20  token-nsubs-keyword
   16  token-nsubs-symbol

== durable-binder-nsubs ==            (form 2 — the field itself)
   ./wat-scripts/topic/sns-fanout.wat:71:15

== record-accessor-nsubs ==           (form 3 — SUBSUMED: the value of the :220 kwarg)
   ./wat-scripts/topic/sns-fanout.wat:220:13

== record-kwarg-nsubs ==              (form 1 — the rewrite population)
   ./wat-scripts/fanout/circuit.wat:2190:41
   ./wat-scripts/fanout/circuit.wat:2718:42
   ./wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat:44:41
   ./wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat:75:41
   ./wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat:104:41
   ./wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat:64:41
   ./wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat:133:41
   ./wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat:158:41
   ./wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat:43:41
   ./wat-scripts/topic/sns-fanout.wat:220:5
   ./wat-scripts/topic/sns-fanout.wat:888:41
   ./wat-scripts/topic/sns-fanout.wat:959:41
   ./wat-scripts/topic/sns-fanout.wat:1014:41
   ./wat-scripts/topic/sns-fanout.wat:1033:41
   ./wat-scripts/topic/sns-fanout.wat:1050:41
   ./wat-scripts/topic/sns-fanout.wat:1071:41
   ./wat-scripts/topic/sns-fanout.wat:1100:41
   ./wat-scripts/topic/sns-fanout.wat:1115:41
   ./wat-scripts/topic/sns-fanout.wat:1140:41
   ./wat-scripts/topic/sns-fanout.wat:1182:41

== token-nsubs-symbol ==              (the HYPOTHESIS — 15 of these 16 must survive)
   ./wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat:36:6      ← let binding
   ./wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat:44:48     ← its only reference
   ./wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat:67:6       ← let binding
   ./wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat:75:48      ← its only reference
   ./wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat:149:4   ← fn PARAMETER
   ./wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat:158:48  ← its only reference
   ./wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat:35:6    ← let binding
   ./wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat:43:48   ← reference
   ./wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat:56:11   ← reference (format label)
   ./wat-scripts/topic/sns-fanout.wat:71:15    ← ⚑ the ONLY one the binder rule claims
   ./wat-scripts/topic/sns-fanout.wat:425:19   ← ⛔ the worker's let binding. MUST SURVIVE
   ./wat-scripts/topic/sns-fanout.wat:506:65   ← reference to it
   ./wat-scripts/topic/sns-fanout.wat:527:65   ← reference to it
   ./wat-scripts/topic/sns-fanout.wat:528:880  ← reference to it
   ./wat-scripts/topic/sns-fanout.wat:533:56   ← reference to it
   ./wat-scripts/topic/sns-fanout.wat:534:44   ← reference to it

== token-keyword sites the kwarg rule does NOT claim ==   (empty)
== kwarg sites the token census missed ==                 (empty)
```

Verbatim, as the finder printed it:

```
"#wat.grep/Match {:file \"./wat-scripts/fanout/circuit.wat\" :line 2190 :col 41 :end-line 2190 :end-col 47 :rule \"record-kwarg-nsubs\" :captures #wat.core/PersistentVector [#wat.grep/Capture {:name \"old\" :value \":nsubs\"}]}"
"#wat.grep/Match {:file \"./wat-scripts/topic/sns-fanout.wat\" :line 71 :col 15 :end-line 71 :end-col 20 :rule \"durable-binder-nsubs\" :captures #wat.core/PersistentVector [#wat.grep/Capture {:name \"old\" :value \"nsubs\"}]}"
"#wat.grep/Match {:file \"./wat-scripts/topic/sns-fanout.wat\" :line 220 :col 13 :end-line 220 :end-col 39 :rule \"record-accessor-nsubs\" :captures #wat.core/PersistentVector [#wat.grep/Capture {:name \"old\" :value \":demo::topic::Record/nsubs\"}]}"
```

---

## ⚠ THE ONE THING THAT SURPRISED ME — a five-way join OOM-KILLED the finder

My first draft of `durable-binder-nsubs` carried **five** `Node` patterns: the symbol, its vector,
the defservice head (`?hi 0`), the service name (`?si 1`), **and** the `:durable` keyword, with
`(:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?di 1 :undefined 0)))` proving the
vector sat immediately after `:durable`. That predicate has committed precedent
(`wat-scripts/scratch-pad/rules-corpus-01-node-facts.wat:85`) and it **matched correctly** —
`sns-fanout.wat` alone gave 31 matches, exit 0.

It was **SIGKILLed on `wat-scripts/fanout/circuit.wat`**:

```
$ printf '["…sns-fanout.wat" "…circuit.wat" …7 paths]\n' | ./target/release/wat --grep <fix>.wat
/bin/bash: line 1:  77061 Killed   | ./target/release/wat --grep …
exit=137
```

Four `Node` patterns sharing one `?parent` variable is an O(children⁴) self-join, and `circuit.wat`
holds forms wide enough to make that fatal.

⛔ **AND IT HAD A MACHINE COST, NOT JUST A PROCESS COST.** The 1809-path invocation of that draft
exhausted RAM **and swap**. `scripts/capped.sh` appeared in the tree, untracked, at 02:12 local while
my floor was running — **it is not mine and I have not touched it** — and its header records the
accounting: `user.slice memory.peak 31G of 31.8G`, `memory.swap.peak 25G of 25.4G`, *"sshd went down
and took ~30 minutes to come back."* That is my census run. I am naming it because it is the real cost
of the finding, and because the same header names the mechanism I hit twice: my `time (...)` wrapper
reported **exit 0** for a run that had been SIGKILLed — *"a pipe returns the PAGER's status"*, the
exact trap CLAUDE.md warns about, and I walked into it before the 7-path run showed me `137` directly. **This is a real cost fact about the rete finder that no
recorded fix in `wat-scripts/fixes/` documents**, and it is the reason the census over 1809 files was
run in 19 chunks of 100 rather than in one invocation.

**Resolution, and it is inside the BRIEF's sketch, not outside it.** I dropped the fifth pattern,
leaving *exactly* the `alarm-after-to-delay.wat:78–97` shape and *exactly* the predicate the BRIEF's
rule 2 specified — symbol, parent kind `vector`, grandparent head `:wat::service::defservice`,
index-1 `:demo::topic`. The `:durable` adjacency moved into the **applier**
(`:tn::defservice-edits` locates the `:durable` keyword and takes the child at its index + 1), which
is therefore **stricter than the finder**: the finder would report a `nsubs` symbol sitting in this
defservice's `:ephemeral` or `:peers` vector; the applier would not touch it. There is no such
symbol in the corpus. The asymmetry is stated in the codemod's own header so a future one is not
rewritten silently. ⚠ **I did not promote anything to `wat/` to fix this** — see STOP-3.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ⛔ **the worker's `nsubs` survives** | ✅ **PASS.** `grep -n 'nsubs (:wat::core::count subs)' wat-scripts/topic/sns-fanout.wat` → `423: [nsubs (:wat::core::count subs)`. The line **moved 425 → 423** (two lines were deleted above it, at `:71-72`); its text is byte-identical. All five references (`:504 :525 :526 :531 :532`, formerly `:506 :527 :528 :533 :534`) survive untouched — `git diff` contains **no line** from the worker |
| 2 | ⛔ **the corpus loads** | ✅ **PASS.** `every_wat_scripts_file_loads_on_the_current_runtime` **PASS [470.388s]** — it parses and type-checks all seven migrated files **and the new codemod**, on the current runtime |
| 3 | ⛔ **the field is gone from the record** | ✅ **PASS.** `grep -n 'nsubs <- ' wat-scripts/topic/sns-fanout.wat` → **no output**. `:durable` is now `[inbox-addr … inbox-lost … inbox-closed … inbox-timedout]` |
| 4 | ★ **no `:nsubs` kwarg survives** | ⚠ **PASS in the corpus, with a delta on the wording.** `grep -rn ':nsubs' --include='*.wat' .` returns **11 lines, every one of them inside `wat-scripts/fixes/topic-record-drop-nsubs.wat`** — 8 comment lines and **3 code lines** that are the finder's own string literals (`(:wat::rete::where (:wat::rete::string::= ?n ":nsubs"))` ×2 and the applier's `(= (:tn::kw-name …) ":nsubs")`). **Zero `:nsubs` anywhere else in 1809 `.wat` files.** The row expected "only comment lines, if any"; a recorded codemod must name the token it deletes, exactly as `declare-queue-drop-knobs.wat` still names `:drop-recv-bp`. The previous stone's proposed gate `grep -rn ':nsubs' --include=*.wat . \| grep -v '^./docs/' \| wc -l → 0` therefore needs `\| grep -v wat-scripts/fixes/` to be the check it intends |
| 5 | ★ **the codemod is recorded and idempotent** | ✅ **PASS.** `wat-scripts/fixes/topic-record-drop-nsubs.wat` (351 lines). Second run on the **migrated corpus**: `md5sum -c` **OK on all 7 files**, `git diff --numstat` byte-identical before and after. Also verified idempotent on the `/tmp` pilot copies before the corpus was touched |
| 6 | ★ **census before apply** | ✅ **PASS, and wider than the row asked.** The full census above was taken over **1809 `.wat` files** — the whole repo outside `target/`, not the DESIGN's seven — **before** the applier ran, then diffed on `/tmp` pilot copies, then applied with all 7 paths on stdin in one invocation |
| 7 | **behaviour is unchanged** | ✅ **PASS.** `./target/release/wat wat-scripts/topic/run.wat` → `"3 3"`, exit 0 |
| 8 | **the circuit is unchanged** | ✅ **PASS.** `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000`, `dup=0`, inbox `accepted=2000`. Also `total=8000`, `seen-recorded=8000`, `seen-skipped=0`, `empty=1`, `visible=0`, `unacked=0`, subscriber `refused=0` on all four subs, `fill-depth=[2000/0]×4`, `inbox-lost/closed/timedout=0`, `redeliveries=0`, exit 0, **zero bytes on stderr** |
| 9 | **the declared surface is untouched** | ✅ **PASS.** `:demo::Topic::StatsResponse::Ok` is still `[depth ticks inbox-lost inbox-closed inbox-timedout]` — the enum is not in the diff at all. `:cap`, `:max-entries [msgs 10]` and `:max-request-bytes` untouched |
| 10 | **blast radius** | ✅ **PASS on everything I touched.** 7 modified `.wat` + 1 new codemod + this SCORE. ⚠ `git status` also shows `?? scripts/capped.sh` and `M scripts/floor.sh`, **neither of which is mine** — both appeared at 02:12/02:13 local while my floor was running; see BLAST RADIUS. **No `wat/`, no `src/`, no `tests/*.rs`, no `mem.wat`, no store.** `git diff --numstat` totals **20 insertions / 22 deletions**; of the 42 changed lines, **21 contain `nsubs`** (the deleted sides) and 21 are their replacements. Nothing outside the stated radius was forced. Full `git status` below |
| 11 | **the floor holds** | ✅ **PASS.** `Summary [ 470.395s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — **0 FAIL, 0 TIMEOUT**, exit `0`, no `ARM.txt`, `grep -cE 'FAIL\|TRY\|TIMEOUT\|ABORT\|SIGSEGV' clean.log` → **0**. Ran **once**. Nothing re-run |

### ★★ Row 12 — the row whose failure would be the orchestrator's

| # | what | result |
|---|---|---|
| 12 | ★★ **the DESIGN's form analysis was complete** | ✅ **PASS on the forms. The table is right and there is no fourth form.** ⚠ **The trap-door list under it is short by nine sites** — see below |

**No fourth form of the field exists.** The finder, run over all 1809 `.wat` files with a token
census beside each form rule, reports **exactly** the three shapes the DESIGN names and nothing else:
20 kwarg pairs, 1 binder triple, 1 accessor — and the accessor is precisely the value of the
`:220` kwarg pair, so deleting the pair took it and **the separate accessor rule found nothing left
to do**, exactly as the EXPECTATIONS predicted. No quasiquote, no macro expansion, no `nsubs` in a
type position, no accessor outside a kwarg value. The `token-nsubs-keyword` ≡ `record-kwarg-nsubs`
identity is the proof for form 1, and it is stronger than any file list could be.

⚠ **What the DESIGN did get wrong, and it is worth naming because it is the same class of error the
DESIGN itself warned about.** The trap section lists the must-survive `nsubs` symbols as *":425 the
worker's `let` binding"* plus *":506 :527 :528 :533 :534"* — **6 sites**. The finder says **15**. The
nine the DESIGN missed are all in `scratch-pad/` probes, and three of them are *bindings*, not
references:

- `probe-a-batch-declares-how-many.wat:36` — `let [nsubs 2]`
- `probe-a-message-is-fanned-once.wat:67` — `let [nsubs 4]`
- `probe-the-topic-publishes-a-batch.wat:35` — `let [nsubs 2]`
- `probe-the-server-manages-its-own-capacity.wat:149` — a **function parameter**,
  `:cap::topic-at [nsubs <- :wat::core::i64  cap <- …  n <- …]`

A symbol-name rule would have deleted these too, and one of them is a *parameter*, so deleting it
would have silently changed a function's arity while its two call sites kept passing three
arguments. The parentage rule refused all four, and the row-12 spirit is satisfied — but the DESIGN's
"23 occurrences, most of them must survive" was itself produced by grepping one file, and it
undercounted the survivors by nine. **The census, not the list, is what protected them.**

---

## STOPS — none fired, and here is the evidence for each

- **STOP-1** (*the finder cannot separate the topic's `:durable` binder from the worker's `let`
  binding at `:425` by parentage*) — **did not fire.** Separation is clean and the fact-shape is
  recorded in the codemod: the symbol's parent must be kind `"vector"`, that vector's parent must be
  a list whose index-0 is `:wat::service::defservice` and index-1 is `:demo::topic`. The worker's
  vector's parent is `:wat::core::let`, so it never enters the join. Measured: the binder rule claims
  **1 of 16** `nsubs` symbols, and `:425` is not it. No symbol-name fallback and no line number
  appears anywhere in the codemod.
- **STOP-2** (*`--grep` reports a `:nsubs` site the DESIGN's exemplars did not anticipate*) — **did
  not fire, and I checked far beyond the exemplars.** The census covered **all 1809 `.wat` files**,
  not the seven. It found `:nsubs` in exactly **7 files** — the same seven the DESIGN's exemplars
  and `git grep` implied, `sns-fanout.wat` + `circuit.wat` + five `scratch-pad` probes. Six of the
  seven were named in the DESIGN's own inheritance note; `probe-refused-retry-self-consumes.wat` was
  not named in the DESIGN's prose but is inside the "six scratch-pad probes" the previous SCORE
  counted. **Nothing outside `wat-scripts/`.** Every path the finder named went on stdin.
- **STOP-3** (*deleting the field needs a new generic in `wat/fix.wat`*) — **did not fire. No change
  to `wat/`.** The existing primitives composed exactly as the DESIGN argued: `node-start-offset` /
  `node-end-offset` (`fix.wat:1002`/`:1006`) locate the endpoints, `fix-text-span-text` (`:231`)
  supplies the verbatim old-text — and its own header names this the **right** door for *"deleting a
  whole matched region"*, which is what a three-token binder and a two-node kwarg pair each are —
  and `fix-text-apply` (`:367`) splices. ⚠ **I did NOT use `fix-text-deletion-edit` (`:243`), and
  that is the one place the BRIEF's primitive list did not fit:** it derives its old-text length from
  `ast-name`, so it reaches a *leaf token only*, and the `:220` kwarg's value is a **list**
  (`(:demo::topic::Record/nsubs d)`). `strip-arrow-ascription`'s composition works because `->` and
  its type are both leaves. The span door was already there for exactly this case, so **no generic
  is missing** — but the BRIEF's *"a binder triple is that move with one more token"* is only true
  for the binder half; the kwarg half needed the span door, not the token door. **No builder's
  ruling is owed.**
- **STOP-4** (*any red in the corpus gate or the floor*) — **did not fire.** One floor run, exit 0,
  no `ARM.txt`, zero `FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines. **Nothing was re-run at any
  point in this stone.** The only non-zero exit anywhere was the finder's `137`, described above; it
  was a tool-cost finding on my own draft rule, before any corpus file was written, and it is
  reported rather than smoothed over.

---

## THE EDIT SITES — the real count, per file

**21 sites** — 20 kwarg pairs + 1 binder triple; the accessor is the 21st match but not a 22nd edit,
because it lies inside the `:220` pair. Every one was located by the codemod; **zero hand-edits, zero
`sed`, zero `python` on `.wat`.**

| file | kwarg pairs | binder | edits | changed lines (+/−) |
|---|---|---|---|---|
| `wat-scripts/topic/sns-fanout.wat` | 11 | 1 | **12** | 11 / 13 |
| `wat-scripts/fanout/circuit.wat` | 2 | — | **2** | 2 / 2 |
| `wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat` | 2 | — | **2** | 2 / 2 |
| `wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat` | 2 | — | **2** | 2 / 2 |
| `wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat` | 1 | — | **1** | 1 / 1 |
| `wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat` | 1 | — | **1** | 1 / 1 |
| `wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat` | 1 | — | **1** | 1 / 1 |
| **total** | **20** | **1** | **21** | **20 / 22** |

`sns-fanout.wat` loses two net lines because two of its twelve sites are whole-line deletions: the
binder's second line at `:72`, and the `:220` accessor pair which occupied a line of its own.

⚠ **The BRIEF's warning was right and my own arithmetic would have been wrong too.** The DESIGN's
discarded single-line grep said 19 constructions; the finder says **20** kwarg sites. The one it
missed is the multi-line construction at `:217-222` — the same site, and the same miss, the BRIEF
predicted.

The two structural results, verbatim from `git diff`:

```diff
 (:wat::service::defservice :demo::topic
   :satisfies :demo::Topic
-  :durable   [nsubs <- :wat::core::i64
-              inbox-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])
+  :durable   [inbox-addr <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])
               inbox-lost <- :wat::core::i64
```

```diff
   (:demo::topic::Record
-    :nsubs (:demo::topic::Record/nsubs d)
     :inbox-addr (:demo::topic::Record/inbox-addr d)
```

Both deletions carry their own separating whitespace: **no double space** where a single-line kwarg
was removed, **no blank indented line** where the multi-line pair was. That was a deliberate choice
of edit ranges — `[end(previous sibling) .. end(value)]` for a kwarg, `[start(symbol) ..
start(next binder)]` for the binder — rather than `fix-text-deletion-edit`'s token-exact deletion,
which `strip-arrow-ascription` leaves double-spaced.

---

## ⚠ WHAT I DID NOT TOUCH, AND WHY — the residue this stone leaves live

The DESIGN scoped comment prose out ("a separate manual pass, because the tool walks forms"). Three
things in that residue are **not** comments and I want them named rather than left for a grep to
rediscover:

1. **Two `let` bindings are now orphaned.** `probe-a-batch-declares-how-many.wat:36` (`let [nsubs 2]`)
   and `probe-a-message-is-fanned-once.wat:67` (`let [nsubs 4]`) each had **exactly one** reference —
   the kwarg this stone deleted. They now bind a value nothing reads. They **type-check** (the
   `_keep`/`_keep2`/`_keep3` idiom in `probe-the-server-manages-its-own-capacity.wat:161-163` is
   standing proof that an unread `let` binding is legal wat), so the corpus gate cannot see them.
2. **One function parameter is now inert.** `:cap::topic-at`
   (`probe-the-server-manages-its-own-capacity.wat:148-149`) still takes `nsubs <- :wat::core::i64` and
   its body no longer uses it; both call sites still pass `4` and `7`.
3. **Three format labels now describe a field that does not exist** —
   `probe-the-server-manages-its-own-capacity.wat:172` (`nsubs4-room6`, `nsubs7-pub10`),
   `probe-a-message-is-fanned-once.wat:120` (`nsubs7-pub10`), and
   `probe-the-topic-publishes-a-batch.wat:54,56` (`nsubs={ns}` — the only surviving *live read* of a
   probe-local `nsubs`, printed into a label).

I left all three alone deliberately: deleting a parameter changes a function's arity and its two call
sites, which is a different change from the one this stone was drawn for, and the BRIEF's radius
authorises forced changes, not adjacent ones. **But they are exactly the "dead state that reads as
configuration" class the DESIGN's own Why section indicts**, only now one scope down, in the probes
rather than on the record. They are on the record here as inherited work, not as done work.

Comment prose still mentioning `nsubs`, untouched as scoped: `sns-fanout.wat:22 :97 :99 :121 :1061`
(`:1061` — *"cap 2, nsubs=1: third is Accepted 0"* — describes a gate that no longer sets `nsubs`),
`probe-a-batch-declares-how-many.wat:6`, `probe-a-message-is-fanned-once.wat:7 :10 :14`,
`probe-the-server-manages-its-own-capacity.wat:7`, `probe-the-topic-publishes-a-batch.wat:4`.

---

## ⚠ ONE PROCESS DELTA I AM NAMING RATHER THAN HIDING

About a minute into the 470-second floor run I made a **comment-only** edit to the codemod's header
(adding the note that its line numbers are pre-migration, and that the finder is one join wider than
the applier). The corpus gate reads `wat-scripts/fixes/*.wat` from disk during that window, so it may
have parsed either version. Both are byte-identical outside `;;` comment text, so the parse and
type-check outcome cannot differ — and I verified the **final** file loads and type-checks on the
current runtime afterwards by running `--grep` with it (exit 0). Stating it because "it can't matter"
is a claim, and the log is the record.

---

## BLAST RADIUS

```
$ git status --porcelain
 M scripts/floor.sh                              ← ⚠ NOT MINE (see below)
 M wat-scripts/fanout/circuit.wat
 M wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat
 M wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat
 M wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat
 M wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat
 M wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat
 M wat-scripts/topic/sns-fanout.wat
?? docs/excursus/2026/08/001-sns-sqs/the-topic-forgets-its-subscriber-count/SCORE.md
?? scripts/capped.sh
?? wat-scripts/fixes/topic-record-drop-nsubs.wat

$ git diff --numstat
2	2	wat-scripts/fanout/circuit.wat
1	1	wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat
2	2	wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat
2	2	wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat
1	1	wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat
1	1	wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat
11	13	wat-scripts/topic/sns-fanout.wat
```

⚠ **Two of those entries are NOT mine, and both arrived while my floor was running.** `git status
--porcelain` was **empty** at HEAD `f9d1684ea` when I started.

- **`?? scripts/capped.sh`** — new at **02:12 local**. The orchestrator's response to the memory
  exhaustion my census caused (see the OOM section above).
- **`M scripts/floor.sh`** — modified at **02:13 local**, to route the floor through
  `scripts/capped.sh`, with a header note dated 2026-09-09 naming the same RAM/swap event.

**I did not create, edit, or run either.** Both are left exactly as found. ⚠ **Consequence for
row 11: my floor ran the HEAD version of `scripts/floor.sh`, UNCAPPED** — it started at
`2026-09-09T09-00-34Z` (02:00 local), thirteen minutes before that edit, so the `Summary` line quoted
at the top of this SCORE came out of `nice -n 19 cargo nextest run --release` with no slice wrapper.
It completed, exit 0, Summary line present. Flagged so the row-10 reading of `git status` is not
mistaken for files I added, and so nobody credits my green to a script it did not run under.

**No `wat/`. No `src/`. No `tests/*.rs`. No `mem.wat`. No store. Nothing under `scripts/` is mine.**
Nothing was forced outside the
radius — `grep -rn 'demo::topic::Record\|nsubs' --include='*.rs' src/ tests/ benches/` returns
**nothing**, so no Rust gate encoded the field. The nine `sns-fanout.wat` gates that
`tests/services/probe_async_publish.rs` drives all construct the record through the migrated kwargs
and all pass in the floor. **Left uncommitted** — the grading and the commit are the orchestrator's.
Of what I added, this SCORE is the only file outside `wat-scripts/`.

---

## WHAT THE NEXT STONE INHERITS

0. ⛔ **A finder can take the box down.** Mine did: RAM and swap both exhausted, sshd unreachable
   for ~30 minutes. `scripts/capped.sh` (untracked, not mine, written during this stone) exists
   because of it. **Run a whole-corpus `--grep` under that wrapper**, and never judge one by a piped
   exit code — my `time (...)` subshell reported `0` for a SIGKILL.
1. ⚠ **The rete finder has a cost cliff nothing in `wat-scripts/fixes/` documents.** Four `Node`
   patterns joined on one shared `?parent` is O(children⁴) and gets the process **SIGKILLed (137)**
   on `wat-scripts/fanout/circuit.wat`. ⚠ Measured on the five-pattern draft, not the shipped rule:
   1809 paths in one invocation produced **zero output at exit 0** (the wrapping `time (...)`
   subshell swallowed the real status — a fallback that collapsed a kill into a green), and a
   7-path invocation was killed at `137` after partial output. Chunking at **100 paths** worked.
   I did **not** re-test 1809-in-one-shot with the shipped four-pattern rule, so the cliff's exact
   position is unmeasured; what is measured is that it exists. Every recorded fix's usage header
   says *"list EVERY path"* and none of them says *"but not all at once."*
2. ★ **Four probe-local `nsubs` bindings are now dead or label-only** (two orphaned `let`s, one inert
   function parameter, three lying format labels — enumerated above). Deleting them is a
   ~4-site change, and it is the same fossil class one scope down.
3. **The topic now knows nothing about subscribers, and the divergence is unrepresentable.**
   `:demo::topic::Record` is `[inbox-addr inbox-lost inbox-closed inbox-timedout]`. The only `nsubs`
   in the service is the worker's `(:wat::core::count subs)` at `:423`. A publish-side decision can
   no longer reach for a stale count, because there is no field to reach for.
4. ⚠ Untouched, as declared: the residual drain slope, `scan-index`'s +45 % level shift, the
   counter-carrier tax, `:cap 64`, `:max-entries [msgs 10]`.

---

## THE BOX

`ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep` was captured before the apply,
before `run.wat`, before the circuit run and before the floor — **every time exactly two lines, both
idle MCP servers**:

```
/home/john/.cargo/bin/wat --mcp
/home/john/.cargo/bin/wat --mcp
```

`cut -d' ' -f1 /proc/loadavg` immediately before the circuit run: **0.57**. The circuit ran alone;
the floor ran alone, after it, once. `wat-scripts/` is read from disk rather than frozen into the
binary, so the migration needed no rebuild and the floor measured the same
`target/release/wat` binary the circuit did.

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

**Floor, read from `.floor/2026-09-09T09-00-34Z/clean.log` and not from the report:**
`Summary [ 470.395s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — **0** lines matching
`FAIL|TRY|TIMEOUT|ABORT|SIGSEGV`, no `ARM.txt`.

| row | my own check | result |
|---|---|---|
| 1 | `grep -n 'nsubs (:wat::core::count subs)'` | `423:` — the worker's binding **survives** ✅ |
| 3 | `grep -c 'nsubs <- '` | `0` ✅ |
| 4 | `:nsubs` outside `wat-scripts/fixes/` | **0**; 11 inside the codemod (its own literals + comments) ✅ |
| 7 | `capped.sh … run.wat` | `"3 3"` ✅ |
| 8 | `capped.sh … circuit.wat 2000 4 3 8192 true 1000` | `distinct=8000 dup=0`, inbox `accepted=2000 refused=190` ✅ |
| 11 | the log above | ✅ |

`refused=190` sits inside the prior stone's band (186–191), so the deletion moved nothing.

## ⭑⭑ The finding that outranks the stone

**The census OOM-killed the machine and took `sshd` down for ~30 minutes.** An O(children⁴) rete
self-join over 1809 paths exhausted RAM **and all 25 G of swap**. That is the most valuable thing
this strike produced, and it did not come from a row.

Three things make it worth more than the deletion:

- ★ **It is a defect class in the *finder*, not in this stone.** A five-`Node` self-join is available
  to every future codemod, and nothing in `wat/fix.wat`'s framework or in any recorded fix warns
  about it. The executor's fix — drop to the exact `alarm-after-to-delay` four-pattern shape and move
  the `:durable` adjacency test into the **applier**, making the applier stricter than the finder —
  is a *shape* other codemods can copy, not a patch.
- ★★ **The pipe trap hid it.** The executor's `time (...)` wrapper reported **exit 0** for that
  SIGKILL; only the direct run showed `137`. That is the third time in two sessions this exact
  substitution has misreported a result, and it is the same one `floor.sh`'s header was written
  about. It is now recorded in `scripts/capped.sh` and `scripts/floor.sh` with the worked numbers.
- **The box now bounds it** (`90c5e4c3b`): a shared 18 G/22 G/2 G slice, sized from the user slice's
  own `memory.peak`/`memory.swap.peak` rather than from a guess.

## ⛔ Row 12 passed on the forms and my trap-door list was still short by nine

The DESIGN's three-form table was **right** — the executor proved it by showing
`token-nsubs-keyword ≡ record-kwarg-nsubs` in both directions, and no fourth form exists.

But the **trap-doors** under it named 6 `nsubs` symbols that must survive. There are **15**. Four are
*bindings* I never named: `let [nsubs N]` in three probes, and a **function parameter** in
`probe-the-server-manages-its-own-capacity.wat:148`. A name-only rule would have silently changed a
function's arity.

★ Same defect as this stone's other one: **I hand-listed sites again.** I caught myself doing it for
the *count* and wrote "wat --grep is the census" into the BRIEF — then did the identical thing one
section lower for the *survivors*. Naming the property covered the census and not the trap-doors.

## One BRIEF assumption was wrong, and the executor read past it correctly

I cited `fix-text-deletion-edit` (`wat/fix.wat:243`) as the deletion primitive. It **cannot** serve:
it sizes old-text from `ast-name`, so it reaches **leaves only**, and the `:220` kwarg's value is a
**list**. The right door — `fix-text-span-text`, whose own header says so — was already there, so
**STOP-3 correctly did not fire** and no `wat/` change was needed. My conclusion held; my cited
mechanism did not. *Cite an exemplar you have read* applies to primitives too, not just to shapes.

## Residue, named and deliberately left

Two orphaned `let [nsubs N]` bindings, one inert function parameter with two live call sites, and
three format labels naming a field that no longer exists. All type-check, so the corpus gate cannot
see them. Deleting a parameter changes arity **and** its call sites — a different change from the one
drawn, and leaving it named rather than silently widening the stone is correct.

**Grade:** `1 ✅ · 2 ✅ · 3 ✅ · 4 ✅ · 5 ✅ · 6 ✅ · 7 ✅ · 8 ✅ · 9 ✅ · 10 ✅ · 11 ✅ · 12 ✅ on the
forms, ⛔ my trap-door list short by nine.` STOPs 1–4 all held without firing.
