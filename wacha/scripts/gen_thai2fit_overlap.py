#!/usr/bin/env python3
"""R11 Phase VEC — generate the thai2fit_wv overlap vector blob (regenerable).

Downloads PyThaiNLP's pretrained `thai2fit_wv` word embeddings ONCE (MIT-licensed;
51,358 words × 300 dim, trained on Thai Wikipedia — we train nothing), intersects
its vocabulary with the wacha vocabulary (words_th.txt + RID headwords), and writes
ONLY the overlapping subset to a compact binary blob.

Output (gitignored, regenerable — never committed):
    data/thai2fit_overlap.vec.blob
    format: u32 count, u32 dim, then per word: u16 len + utf8 + dim*f32 (LE)

Needs: pip install gensim pythainlp   (one-time, offline; NOT a runtime dep of wacha)
Run from the repo root:  python3 wacha/scripts/gen_thai2fit_overlap.py
"""
import re, struct, os, sys

try:
    from pythainlp.word_vector import WordVector
except Exception as e:
    sys.exit(f"pythainlp not available ({e}); pip install gensim pythainlp")

model = WordVector(model_name="thai2fit_wv").get_model()
dim = model.vector_size

wacha = set()
for line in open("data/words_th.txt", encoding="utf-8"):
    w = line.strip()
    if w:
        wacha.add(w)
rid = "data/rid/dict_2554.txt"
if os.path.exists(rid):
    for line in open(rid, encoding="utf-8"):
        m = re.match(r"^HEADWORD: (.+?)(\s+[๑๒๓๔๕๖๗๘๙])?\s*$", line)
        if m:
            wacha.add(m.group(1).strip())

overlap = [w for w in model.index_to_key if w in wacha]
out = "data/thai2fit_overlap.vec.blob"
with open(out, "wb") as f:
    f.write(struct.pack("<II", len(overlap), dim))
    for w in overlap:
        b = w.encode("utf-8")
        f.write(struct.pack("<H", len(b)))
        f.write(b)
        f.write(model[w].astype("float32").tobytes())

pct = 100 * len(overlap) / len(wacha) if wacha else 0
print(f"wrote {out}: {len(overlap)} words x {dim} dim "
      f"({pct:.1f}% of wacha vocab, {100*len(overlap)/len(model.index_to_key):.1f}% of thai2fit), "
      f"{os.path.getsize(out)} bytes")
