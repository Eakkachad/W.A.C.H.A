// WASM smoke test — catches the class of bug that shipped a broken flagship in R7.
//
// R7 verified the WASM build (W2), then S2 added `Instant::now()` to the engine
// build path, which panics on wasm32 ("time not implemented"). Nothing re-checked
// the WASM build, so R7 shipped with the offline demo trapping on init. This test
// exists so that any change to `wacha` which breaks the wasm32 target fails loudly
// in verify_r5.sh instead of silently.
//
// Node is enough to catch traps, panics and wrong output. It is NOT a substitute
// for opening the page in a real browser (service worker, offline toggle, phone
// rendering) — that check is on the pre-event human checklist.
//
// Usage: node smoke.mjs [path/to/wacha_wasm.wasm]

import { readFileSync } from 'node:fs';

const WASM = process.argv[2] ?? new URL('./web/wacha_wasm.wasm', import.meta.url).pathname;

let failures = 0;
const check = (name, ok, detail = '') => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`);
  if (!ok) failures++;
};

const bytes = readFileSync(WASM);
const t0 = performance.now();
const { instance } = await WebAssembly.instantiate(bytes, {});
const tInstantiate = performance.now() - t0;

const x = instance.exports;
const mem = () => new Uint8Array(x.memory.buffer);

// Call a (ptr,len)->ptr export with a UTF-8 string, decode the
// [u32 LE len][len bytes JSON] envelope, free both buffers.
function call(fn, query) {
  const q = new TextEncoder().encode(query);
  const qptr = x.wacha_alloc(q.length);
  mem().set(q, qptr);
  const rptr = fn(qptr, q.length);
  x.wacha_free(qptr, q.length);
  const len = new DataView(x.memory.buffer).getUint32(rptr, true);
  const json = new TextDecoder().decode(mem().subarray(rptr + 4, rptr + 4 + len));
  x.wacha_free(rptr, 4 + len);
  return JSON.parse(json);
}

console.log(`=== WACHA WASM smoke test ===`);
console.log(`artifact: ${WASM} (${(bytes.length / 1048576).toFixed(2)} MB raw)`);
check('instantiate without imports', true, `${tInstantiate.toFixed(1)} ms`);

// 1. init must not trap. This is the exact failure R7 shipped.
const t1 = performance.now();
const words = x.wacha_init();
const tInit = performance.now() - t1;
check('wacha_init does not trap', words > 0, `${words.toLocaleString()} words, ${tInit.toFixed(1)} ms`);

// 2. segmentation on a known sentence
const seg = call(x.wacha_segment, 'เด็กน้อยรักสุนัข');
const segText = seg.map((t) => t.text).join(' | ');
check('segment เด็กน้อยรักสุนัข', segText === 'เด็กน้อย | รัก | สุนัข', segText);

// 3. a NON-SEED word must return a real Kaikki definition. This is what proves
//    the definitions actually shipped (W2) rather than the reduced dataset.
for (const w of ['ปัญญาประดิษฐ์', 'ครอบครัว', 'รถยนต์']) {
  const r = call(x.wacha_lookup, w);
  const def = r.entry?.senses?.[0]?.definition ?? r.entry?.definition ?? null;
  check(`lookup ${w} returns a definition`, !!def && def.length > 0, def ? def.slice(0, 40) + '…' : 'NO DEFINITION');
}

// 4. related words still come back with provenance
const kru = call(x.wacha_lookup, 'ครู');
check('lookup ครู has related words', (kru.related?.length ?? 0) > 0, `${kru.related?.length ?? 0} related`);

// 5. reverse dictionary works offline in the artifact
const rev = call(x.wacha_reverse, 'ที่เก็บเงินของรัฐ');
const top = rev.hits?.[0]?.word ?? null;
check('reverse "ที่เก็บเงินของรัฐ" returns hits', (rev.hits?.length ?? 0) > 0, `top: ${top}`);

// 6. repeated calls must not corrupt memory or leak into a trap
for (let i = 0; i < 50; i++) call(x.wacha_lookup, 'บ้าน');
check('50 repeated lookups do not trap', true);

console.log('---');
console.log(failures === 0 ? 'WASM smoke: ALL PASS' : `WASM smoke: ${failures} FAILURE(S)`);
process.exit(failures === 0 ? 0 : 1);
