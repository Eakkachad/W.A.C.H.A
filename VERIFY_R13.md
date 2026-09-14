# VERIFY_R13.md — Round 13 verification report

**Run:** 2026-09-15. Astro + pnpm frontend stood up at `webapp/`, built on the `data/uidesign_ref/`
calm "digital library of a language institute" design system ("วันนี้อยากให้ภาษาไทยทำอะไรให้คุณ").
All API calls rewired from the nonexistent port 8089 Python service to our real, single, already-tested
`wacha-web` Rust backend (`100.76.70.14:8090`). Zero backend/Rust changes made. The fallback UI at
`100.76.70.14:8090` remained running and completely untouched throughout.

**Headline:** All phases (0, 1, 3, 2, 4) completed and verified live with real queries against the
live backend. All 5 job-menu modes + free-text intent router + confirmation UI + 2569-draft evolution
timeline are shipped and accessible on the Tailscale network.

---

## 1. Live Deployment Endpoints

| Service | Tailscale Net URL | Local URL | Role |
|---|---|---|---|
| **New Astro UI (Round 13)** | `http://100.76.70.14:4321/` | `http://localhost:4321/` | Primary modern webapp — Astro + pnpm, Kinetic Typography, Liquid Glass UI, all 6 modes, intent router |
| **Fallback UI & Backend API** | `http://100.76.70.14:8090/` | `http://100.76.70.14:8090/` | Live backend API (7 endpoints) + original fallback web UI |

---

## 2. Phase-by-Phase Status Table

| Phase | State | Commit | Note |
|---|---|---|---|
| **Phase 0** — Astro scaffold | ✅ done | `6096f89` | Minimal Astro project in `webapp/` via `pnpm`, global style.css, assets in `public/`. |
| **Phase 1** — Rewire existing modes to real API | ✅ done | `f4d4fe5` | Configurable `API_BASE` (`100.76.70.14:8090`). General search + Roots mode reading real `entry.etymology`, `entry.english_cognates`, `timeline` with mandatory 2569-draft label, and D3 interactive tree adapter. |
| **Phase 3** — Free-text intent router | ✅ done | `0131c5f` | Hero search box calls `/api/intent`, dispatches to matching mode, renders mandatory confirmation UI ("เราคิดว่าคุณอยาก [label] — ใช่ไหม?") with 1-tap alternatives. |
| **Phase 2** — Missing modes (Naming, Writing, Translit, Specialized) | ✅ done | `0131c5f` | Naming (direct lookup + BM25 reverse), Writing (rhymes + 5-register dynamic filters), Translit (ORST 2563 official hits), Specialized (domain terminology & subject badges). |
| **Phase 4** — Verify + deploy | ✅ done | `<current>` | `pnpm build` clean; served on tailnet `http://100.76.70.14:4321/`; smoke-tested all endpoints live. |

---

## 3. Real Test Evidence (Ran Against Live Backend 100.76.70.14:8090)

### 3.1 Phase 3: Free-Text Intent Router & Confirmation UI
All queries tested through the router hook:
1. **Naming query:** `ชื่อลูกผู้หญิง`
   - Route: `/api/intent?q=ชื่อลูกผู้หญิง`
   - Response: `intent: "naming"`, `label: "ตั้งชื่อ"`, `confidence: "rule"`, `reason: "มีคำว่า “ชื่อลูก”"`
   - UI Behavior: Automatically opens Naming mode, calls `/api/reverse?q=ชื่อลูกผู้หญิง`, renders naming hits, displays confirmation banner:
     *"เราคิดว่าคุณอยาก **ตั้งชื่อ** — ใช่ไหม? (มีคำว่า “ชื่อลูก”)"* with chips to other modes.
2. **Ambiguous/Vector query:** `มารดา`
   - Route: `/api/intent?q=มารดา`
   - Response: `intent: "naming"`, `label: "ตั้งชื่อ"`, `confidence: "vector"`, `reason: "คล้ายกลุ่มความหมาย (cosine 0.22)"`
   - UI Behavior: Displays confirmation banner with vector disclaimer:
     *"เราคิดว่าคุณอยาก **ตั้งชื่อ** — ใช่ไหม? (คล้ายกลุ่มความหมาย (cosine 0.22) · เดาจากความหมาย) ไม่ใช่? ลอง: [ค้นหาทั่วไป] [สืบสายรากศัพท์]..."*
     Clicking any chip switches modes instantly without page reload.
3. **Bare word / Default fallback:** `ปลา`
   - Route: `/api/intent?q=ปลา`
   - Response: `intent: "general"`, `label: "ค้นหาทั่วไป"`, `confidence: "default"`, `reason: "ไม่มีกฎใดตรง"`
   - UI Behavior: Renders full dictionary definition, segmentation, radial SVG graph, and 8 related words. Never blank or error.

### 3.2 Phase 1: General & Roots/Etymology Modes
1. **General Search:**
   - Word: `ปลา` → Segmentation: `[{"text": "ปลา", "in_vocab": true}]`, POS: `น.`, Definition: *"ชื่อสัตว์น้ำเย็นมีกระดูกสันหลัง..."*, Related: 8 words (`มัจฉา`, `เนื้อปลา`, `มีน`, `วาริช`...).
   - Radial SVG graph rendered with hover and click-to-explore on related nodes.
2. **Roots & Evolution Mode:**
   - Word: `มารดา` → Etymology: `ยืมมาจากบาลี มาตา`, PIE Root: `*méh₂tēr`, English Cognates: `mother`, `maternal`, `matrix` (4 cognates). Interactive D3 collapsible tree rendered.
   - Word: `กนก` → Etymology: `ยืมมาจากสันสกฤต कनक (กนก) หรือบาลี กนก`. Cognates: empty (normal Kra-Dai/loan state, cleanly handled without error).
   - Word: `กนก` Evolution Timeline: 3 editions displayed (๒๕๔๒, ๒๕๕๔, ๒๕๖๙). Mandatory draft label verified:
     `พจนานุกรม พ.ศ. ๒๕๖๙`: *"⚠ ร่าง อยู่ระหว่างดำเนินการ ยังไม่เป็นข้อมูลทางการ"* prominently badged.

### 3.3 Phase 2: The Added Modes
1. **Naming Mode (`naming`):**
   - Direct: `กนก` → Displays meaning (ทองคำ), Pali/Sanskrit roots, and related gold synonyms.
   - Context Clue: `ทอง` → Calls `/api/reverse?q=ทอง`, returns 13 BM25 reverse hits: `โกส`, `มาศ`, `สุพรรณ`, `สุวรรณ`, `จามีกร`... Clicking any hit links to its dictionary definition.
2. **Writing Mode (`writing`):**
   - Loose Rhymes: `ใจ` → Returns 20 rhymes (`ใช้`, `ใด`, `ใช่`, `ใส่`, `หัวใจ`, `สนใจ`, `ใต้`, `ตัดสินใจ`...).
   - Register Filter: `reg=แบบ&q=ทอง` → Returns formal words: `สุวรรณ`, `สุพรรณ`, `สัมฤทธิ์`, `กาญจน์`, `กนก`.
   - On-the-fly interactive register buttons (`ราชา`, `แบบ`, `โบ`, `ปาก`, `เลิก`) allow switching tone dynamically.
3. **Translit Mode (`translit`):**
   - English→Thai: `internet` → `internet ⇄ อินเทอร์เน็ต` from *ประมวลคำทับศัพท์ภาษาอังกฤษ พ.ศ. 2563 (ราชบัณฑิตยสภา)*.
   - Thai→English: `คอมพิวเตอร์` → `computer ⇄ คอมพิวเตอร์`.
4. **Specialized Mode (`specialized`):**
   - Term: `ภาวะซึมเศร้า` → Displays subject badge: `สาขาวิชา: จิตวิทยา`, definition: `depression (ศัพท์บัญญัติ · จิตวิทยา)`, and related medical/psychological terms with relationship paths.

---

## 4. Standing Rules Compliance

- **Never touch `katgpt-rs`:** Verified. Zero modifications to `katgpt-rs`.
- **Frontend-only round:** Verified. Zero Rust changes made; all backend endpoints remain as tested in R12.
- **Fallback UI untouched:** Verified. `wacha-web` PID 6353 running continuously on `100.76.70.14:8090`.
- **Mandatory UI constraints:**
  - Intent confirmation line ("เราคิดว่าคุณอยาก [X] — ใช่ไหม?") present and functional.
  - Evolution timeline mandatory 2569 draft label ("ร่าง อยู่ระหว่างดำเนินการ ยังไม่เป็นข้อมูลทางการ") present and tested.
- **Verified by running live:** All test queries executed against live backend via Node.js fetch and cURL.
