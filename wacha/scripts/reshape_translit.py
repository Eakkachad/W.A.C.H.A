#!/usr/bin/env python3
"""R10 Phase T — reshape the organizer's official transliteration list to a flat
TSV the translit module loads.

Source (git-ignored, ORST):
    data/official/คำทับศัพท์/คำทับศัพท์ที่ใช้บ่อย.xlsx  (sheet "จากเล่ม", 2,256 rows)
    cols: ลำดับที่ | ศัพท์ (English) | คำทับศัพท์ (Thai) | หมายเหตุ
Reference: ประมวลคำทับศัพท์ภาษาอังกฤษ พ.ศ. 2563 (ราชบัณฑิตยสภา).

Output:
    data/translit.tsv   —  english \t thai \t note   (one row per pair)
"""
import sys, glob, math
import pandas as pd

SRC = glob.glob("data/official/คำทับศัพท์/*.xlsx")
if not SRC:
    sys.exit("ERROR: คำทับศัพท์ xlsx not found under data/official/")
OUT = "data/translit.tsv"


def clean(v):
    if v is None:
        return ""
    if isinstance(v, float) and math.isnan(v):
        return ""
    s = str(v).strip()
    return "" if s.lower() == "nan" else s


def main():
    df = pd.read_excel(SRC[0], sheet_name="จากเล่ม", dtype=str)
    rows = []
    for _, r in df.iterrows():
        en = clean(r.get("ศัพท์ (English)"))
        th = clean(r.get("คำทับศัพท์ (Thai)"))
        note = clean(r.get("หมายเหตุ"))
        if note == "-":
            note = ""
        if en and th:
            # tabs/newlines can't appear in a TSV cell
            en = en.replace("\t", " ").replace("\n", " ")
            th = th.replace("\t", " ").replace("\n", " ")
            note = note.replace("\t", " ").replace("\n", " ")
            rows.append((en, th, note))

    with open(OUT, "w", encoding="utf-8") as fh:
        for en, th, note in rows:
            fh.write(f"{en}\t{th}\t{note}\n")
    print(f"wrote {OUT}: {len(rows)} transliteration pairs")


if __name__ == "__main__":
    main()
