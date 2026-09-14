#!/usr/bin/env python3
"""R10 Phase E — reshape the 3 ORST specialized-domain dictionaries to one TSV.

Sources (git-ignored, ORST): data/official/พจนานุกรม เฉพาะสาขาวิชา/
    ศัพท์จิตวิทยา.xlsx (1,461)  ศัพท์ปรัชญา.xlsx (1,600)  ศัพท์แพทย์.xlsx (4,180)
Shared columns: number | ศัพท์ตั้ง (English) | ศัพท์บัญญัติ (Thai coined) |
    สาขาวิชาของศัพท์ตั้ง | เดือนและปีที่จัดพิมพ์ | ศัพท์ตั้งอื่น | คำอ้างอิง |
    ศัพท์อื่นที่เป็นคำเดียวกันในต่างสาขา | คำอธิบาย (sparse — often empty).

HONEST framing (matches CoinedWordImporter's own doc): this is authoritative
English↔Thai *term equivalence* per discipline, NOT full encyclopedic definitions.

Output: data/specialized_terms.tsv
    english \t thai \t discipline \t cross_discipline
(one row per (english, thai-variant); the Thai cell may hold ONE coined term after
splitting the source's comma/numbered variants; cross_discipline = the raw
"ศัพท์อื่นที่เป็นคำเดียวกันในต่างสาขา" cell, kept as a display/relation hint.)
"""
import sys, glob, os, math, re
import pandas as pd

SRCS = sorted(glob.glob("data/official/พจนานุกรม เฉพาะสาขาวิชา/*.xlsx"))
if not SRCS:
    sys.exit("ERROR: specialized-domain xlsx not found under data/official/")
OUT = "data/specialized_terms.tsv"

# Normalize the discipline label to a short tag.
DISC_MAP = {
    "จิตวิทยา": "จิตวิทยา",
    "ปรัชญา": "ปรัชญา",
    "ศัพท์แพทยศาสตร์": "แพทยศาสตร์",
    "แพทยศาสตร์": "แพทยศาสตร์",
}

POS_PREFIX = re.compile(r"^(น\.|ก\.|ว\.|ส\.|สัน\.?|บ\.|อ\.|นิ\.)\s*")


def clean(v):
    if v is None:
        return ""
    if isinstance(v, float) and math.isnan(v):
        return ""
    s = str(v).strip()
    if s.lower() == "nan":
        return ""
    return re.sub(r"\s+", " ", s.replace("\t", " ").replace("\n", " ")).strip()


def split_thai_terms(cell):
    """The ศัพท์บัญญัติ cell may hold: 'น. ก, ข', '1. …, 2. …', 'ก; ข'.
    Return a list of individual coined Thai terms (POS marker + numbering removed)."""
    cell = POS_PREFIX.sub("", cell).strip()
    # Split off numbered senses like '1. term, 2. term'
    parts = re.split(r"\s*\d+\.\s*|[;,]", cell)
    out = []
    for p in parts:
        p = POS_PREFIX.sub("", p).strip()
        # drop parentheticals / trailing English glosses
        p = re.sub(r"\([^)]*\)", "", p).strip()
        if p and any('\u0E00' <= c <= '\u0E7F' for c in p):
            out.append(p)
    # dedupe, keep order
    seen = set(); res = []
    for p in out:
        if p not in seen:
            seen.add(p); res.append(p)
    return res


def main():
    rows = []
    for f in SRCS:
        df = pd.read_excel(f, sheet_name=0, dtype=str)
        for _, r in df.iterrows():
            en = clean(r.get("ศัพท์ตั้ง"))
            thai_cell = clean(r.get("ศัพท์บัญญัติ"))
            disc_raw = clean(r.get("สาขาวิชาของศัพท์ตั้ง"))
            disc = DISC_MAP.get(disc_raw, disc_raw or os.path.basename(f))
            cross = clean(r.get("ศัพท์อื่นที่เป็นคำเดียวกันในต่างสาขา"))
            if not en or not thai_cell:
                continue
            for thai in split_thai_terms(thai_cell):
                rows.append((en, thai, disc, cross))

    with open(OUT, "w", encoding="utf-8") as fh:
        for en, thai, disc, cross in rows:
            fh.write(f"{en}\t{thai}\t{disc}\t{cross}\n")
    n_cross = sum(1 for r in rows if r[3])
    print(f"wrote {OUT}: {len(rows)} (english,thai) rows ({n_cross} with a cross-discipline note) from {len(SRCS)} files")


if __name__ == "__main__":
    main()
