// วาจา (WACHA) offline PWA service worker.
// Cache-first for the app shell + wasm, so a second visit is fully offline.
const CACHE = 'wacha-v3';
const ASSETS = ['./', './index.html', './wacha_wasm.wasm', './manifest.webmanifest', './icon-192.png', './icon-512.png', './icon-maskable-512.png', './favicon.png'];

self.addEventListener('install', (e) => {
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(ASSETS)).then(() => self.skipWaiting()));
});

self.addEventListener('activate', (e) => {
  e.waitUntil(
    caches.keys().then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (e) => {
  e.respondWith(
    caches.match(e.request).then((hit) => hit || fetch(e.request).then((resp) => {
      // Runtime-cache same-origin GETs so the wasm is available offline next time.
      if (e.request.method === 'GET' && resp.ok) {
        const copy = resp.clone();
        caches.open(CACHE).then((c) => c.put(e.request, copy));
      }
      return resp;
    }).catch(() => caches.match('./index.html')))
  );
});
