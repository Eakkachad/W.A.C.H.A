# COMPETITION_DAY.md — ingesting the organizer's RID dataset

**Purpose.** On event day the organizer (ORST) may hand out the real พจนานุกรม
ฉบับราชบัณฑิตยสถาน ๒๕๕๔ dataset. This is the exact, timed runbook to ingest it —
no code archaeology required. Target: **under 10 minutes** end-to-end. (Dry-run
against the test fixtures: measured **~3 minutes** including the trie rebuild —
see PROGRESS.md.)

The ingestion point already exists: `RidImporter` (`wacha/src/import/rid.rs`) is
wired into `Engine::load_from_dir`, which auto-loads `data/rid/` if it exists,
at the **highest** merge priority (`Rid > HumanSeed > CoinedWord > Kaikki >
Lexitron`). So in the common case, ingestion is a *file drop + rebuild*, not a
code change.

---

## Step 1 — Put the file where the engine looks (30 s)

```bash
cd dict-hackathon
mkdir -p data/rid
cp /path/to/organizer_rid_dump.txt data/rid/
```

`data/rid/` may hold one or many `.txt` files; all are loaded and merged. (It is
gitignored like the other bulk data.)

## Step 2 — Check the format matches the fixtures (2 min — the make-or-break step)

Open the organizer's file and compare **one entry** against the fixture grammar
in `wacha/tests/fixtures/rid/sample_entries.txt` (header comment documents every
field). The importer expects this line grammar per blank-line-separated block:

```
HEADWORD: <headword> [<homograph Thai numeral>]
READING:  [<คำอ่าน>]                       (optional)
ETYM:     (ป. …; ส. …)                      (optional)
SUBENTRIES: w1, w2, w3                      (optional, ลูกคำ)
SENSE: [POS] (สาขาวิชา) {register} (๑) <definition> [ดู <xref>]
```

- **If it already looks like this** → skip to Step 3.
- **If it's a different serialization** (JSON, CSV, the raw
  `func_lookup.php` HTML, the `lookupWord_conditional.php` JS-array) → you have
  two clean options:
  1. **Reshape their file to the grammar above** with a throwaway script (fast
     if their format is regular), then Step 3 unchanged. *Preferred.*
  2. **Edit the one adapter function** `RidImporter::parse_entry` in
     `wacha/src/import/rid.rs`. It is the *only* function that touches the input
     serialization — everything downstream (`Entry`/`Sense`/graph) is
     format-independent. The controlled vocabularies it maps onto are already
     complete: `Pos::from_marker` (8), `Subject::from_marker` (32),
     `Register::from_marker` (5). Re-run `cargo test import::rid` — the 7
     fixture tests are your regression net; make them pass with any adapter
     edit before rebuilding.

**Check first if unsure:** run `head -40 data/rid/<file>` and look for the
headword/POS/sense-number markers. The Thai-numeral homograph (`๑ ๒`), the
`[น.]/[ก.]` POS brackets, and the `(ป. …)` etymology are the highest-signal
tells.

## Step 3 — Rebuild the trie cache (auto-invalidates) (~1–2 min)

Adding RID headwords changes the segmenter vocabulary, so the trie cache
**auto-invalidates** (it is keyed on a hash of the full vocab — Task 5). Just run
any command; the first run rebuilds (~60 s cold), subsequent runs are ~1.5 s:

```bash
cd wacha
cargo build --release
./target/release/wacha --data ../data stats     # first run rebuilds the trie
```

If you ever suspect a stale cache, delete it explicitly:
`rm ../data/words_th.datrie.cache` (it will be rebuilt on the next run).

## Step 4 — Spot-check 5 words end-to-end (2 min)

Pick 5 headwords you saw in the organizer's file and confirm each returns the RID
definition (source tag `RID ๒๕๕๔ (ORST)`) plus related words:

```bash
for w in <word1> <word2> <word3> <word4> <word5>; do
  ./target/release/wacha --data ../data lookup "$w"
done
```

Confirm, per word: the definition text is the RID one; the source line reads
`ที่มานิยาม: RID ๒๕๕๔ (ORST)`; multi-sense words show every sense; homographs
(`แมว ๑`/`แมว ๒`) are distinct.

## Step 5 — Serve it (30 s)

```bash
./target/release/wacha-web --data ../data --host 127.0.0.1 --port 8095
# then: curl 'http://127.0.0.1:8095/api/lookup?q=<word>'
```

---

## If something breaks

- **`RID parse error in block: …`** → the adapter hit a line it couldn't parse.
  The error prints the offending block. Fix `parse_entry` (Step 2, option 2) or
  the reshape script (option 1). Unknown line labels are ignored by design, so
  this only fires on a malformed `SENSE`/`ETYM`.
- **A word segments but has no definition** → it reached the segmenter vocab but
  not the dictionary. Confirm the entry has at least one `SENSE:` line (a
  headword-only block loads as a segmentable word with no sense — valid, but no
  definition).
- **Cache seems stale** → `rm data/words_th.datrie.cache` and re-run.
- **Merge didn't prefer RID** → RID is highest priority by construction; if a
  Kaikki definition still shows first, the RID entry's headword/homograph key
  didn't match (e.g. homograph number mismatch). Check `HEADWORD:` parsing.

## What NOT to do

- Do **not** bulk-scrape `dictionary.orst.go.th` — use only the file the
  organizer provides. (The fixtures are hand-copied fair-use test data.)
- Do **not** present RID content as open data — it is ORST educational /
  non-commercial. The combined dataset is CC BY-SA because of the Kaikki layer.
