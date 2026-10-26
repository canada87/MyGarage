// Makes the app installable and lets the shell open offline. API data is always
// fetched from the network: deadlines must never come from a stale cache.
import { self } from '$app/service-worker';
import { assets, immutable } from '$app/manifest';
import { version } from '$app/env';

const CACHE = `mygarage-${version}`;
const abs = (p: string) => (p.startsWith('/') ? p : `/${p}`);
const ASSETS = [...immutable, ...assets].map((f) => abs(f.path));

self.addEventListener('install', (event) => {
	event.waitUntil(
		caches
			.open(CACHE)
			.then((cache) => cache.addAll([...ASSETS, '/']))
			.then(() => self.skipWaiting())
	);
});

self.addEventListener('activate', (event) => {
	event.waitUntil(
		caches
			.keys()
			.then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
			.then(() => self.clients.claim())
	);
});

self.addEventListener('fetch', (event) => {
	const req = event.request;
	if (req.method !== 'GET') return;
	const url = new URL(req.url);
	if (url.origin !== self.location.origin || url.pathname.startsWith('/api/')) return;

	event.respondWith(
		(async () => {
			const cache = await caches.open(CACHE);
			if (ASSETS.includes(url.pathname)) {
				const hit = await cache.match(url.pathname);
				if (hit) return hit;
			}
			try {
				return await fetch(req);
			} catch {
				// Offline: serve the app shell for page navigations.
				const shell = req.mode === 'navigate' ? await cache.match('/') : undefined;
				return shell ?? new Response('Offline', { status: 503 });
			}
		})()
	);
});
