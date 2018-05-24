/*
const CACHE = 'hwpb-precache';

self.addEventListener('activate', event => {
    return self.clients.claim();
});

self.addEventListener('fetch', event => {
    event.respondWith(async () => {
        let cache = await caches.open(CACHE);
        let cachedResponse = await cache.match(event.request);

        if(cachedResponse) {
            return cachedResponse;
        }

        let response = await fetch(event.request);
        cache.put(event.request, response.clone());
        return response;
    });
});
*/
