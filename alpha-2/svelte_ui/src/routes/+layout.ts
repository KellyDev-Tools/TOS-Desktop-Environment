// Disable SSR — this is a pure client-side SPA served into a webview or browser.
// No server-side rendering needed; the Brain provides all state via WebSocket IPC.
export const ssr = false;
export const prerender = true;
