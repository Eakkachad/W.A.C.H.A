#!/usr/bin/env python3
"""R10 Phase W — Word Evolution Timeline (ก only).

Builds one keyed comparison table across the 3 RID editions the organizer shipped,
restricted to อักษร ก (all DICT_2569 covers):
    2542  data/official/.../DICT_2542 (ก_ฮ).xlsx   cols: number,K_W,M_W,D_T  (filter ก)
    2554  data/rid/dict_2554.txt                    (already reshaped in Phase R)
    2569  data/official/.../DICT_2569 (ก_Incomplete).xlsx  cols: head_word,...,definition (DRAFT)

Output: data/evolution_ko.tsv   ->   headword \t edition \t definition
Editions labelled "๒๕๔๒" / "๒๕๕๔" / "๒๕๖๙". The 2569 rows are a DRAFT — the CLI/UI
carry the mandatory ORST integrity caveat verbatim; the reshaper just tags edition.
"""
import sys, glob, os, math, re, html as _html
import pandas as pd

OUT = "data/evolution_ko.tsv"
THAI_NUM = "๐๑๒๓๔๕๖๗๘๙"


def clean(v):
    if v is None:
        return ""
    if isinstance(v, float) and math.isnan(v):
        return ""
    s = str(v).strip()
    if s.lower() == "nan":
        return ""
    s = re.sub(r"</?[a-zA-Z][^>]*>", "", s)  # strip HTML italics
    s = _html.unescape(s)                    # decode &#160; &#x0e4d; &amp; etc.
    # drop private-use / control glyphs that some entities decode to
    s = "".join(c if (c.isprintable() and c != "\ufeff") else " " for c in s)
    return re.sub(r"\s+", " ", s.replace("\t", " ").replace("\n", " ")).strip()


def base_headword(hw):
    """Normalize a headword to its base form for cross-edition keying:
    drop a trailing homograph numeral and anything after a comma."""
    hw = hw.split(",")[0].strip()
    # drop trailing Thai/Arabic numeral markers like 'ก็ ๑'
    parts = hw.split()
    if len(parts) > 1 and all(c in THAI_NUM for c in parts[-1]):
        hw = " ".join(parts[:-1])
    return hw.strip()


def load_2542():
    f = glob.glob("data/official/พจนานุกรม*/DICT_2542*.xlsx")
    if not f:
        return {}
    df = pd.read_excel(f[0], sheet_name=0, dtype=str)
    out = {}
    for _, r in df.iterrows():
        hw = clean(r.get("K_W"))
        if not hw.startswith("ก"):
            continue
        d = clean(r.get("D_T"))
        if not d:
            continue
        out.setdefault(base_headword(hw), d)  # first sense/definition
    return out


def load_2554():
    """Parse the ก-slice from the Phase-R block file."""
    path = "data/rid/dict_2554.txt"
    if not os.path.exists(path):
        return {}
    out = {}
    cur = None
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        if line.startswith("HEADWORD:"):
            cur = base_headword(line[len("HEADWORD:"):].strip())
        elif line.startswith("SENSE:") and cur and cur.startswith("ก"):
            # strip the "[POS] (subj) {reg}" prefix to leave the definition text
            body = line[len("SENSE:"):].strip()
            body = re.sub(r"^\[[^\]]*\]\s*", "", body)
            body = re.sub(r"^\([^)]*\)\s*", "", body)
            body = re.sub(r"^\{[^}]*\}\s*", "", body)
            if body and cur not in out:
                out[cur] = body.strip()
    return out


def load_2569():
    f = glob.glob("data/official/พจนานุกรม*/DICT_2569*.xlsx")
    if not f:
        return {}
    df = pd.read_excel(f[0], sheet_name=0, dtype=str)
    out = {}
    for _, r in df.iterrows():
        hw = clean(r.get("head_word"))
        if not hw.startswith("ก"):
            continue
        d = clean(r.get("definition"))
        if not d:
            continue
        out.setdefault(base_headword(hw), d)
    return out


def main():
    ed = {"๒๕๔๒": load_2542(), "๒๕๕๔": load_2554(), "๒๕๖๙": load_2569()}
    for k, v in ed.items():
        print(f"  {k}: {len(v)} ก-headwords")
    # union of all headwords, emit one row per (headword, edition) that has a def
    all_hw = set()
    for v in ed.values():
        all_hw.update(v.keys())
    rows = []
    for hw in sorted(all_hw):
        for edition, table in ed.items():
            if hw in table:
                rows.append((hw, edition, table[hw]))
    with open(OUT, "w", encoding="utf-8") as fh:
        for hw, edition, d in rows:
            fh.write(f"{hw}\t{edition}\t{d}\n")
    # headwords present in ALL THREE editions (the strongest timeline demo)
    in_all3 = [hw for hw in all_hw if all(hw in ed[e] for e in ed)]
    print(f"wrote {OUT}: {len(rows)} rows, {len(all_hw)} distinct ก-headwords, "
          f"{len(in_all3)} present in all 3 editions")


if __name__ == "__main__":
    main()
