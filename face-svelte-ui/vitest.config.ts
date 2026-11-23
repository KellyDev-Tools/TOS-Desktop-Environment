import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

// Component/unit tier (standards §2.1): adjacent `.spec.ts`
// files under src/, run headlessly in jsdom. The Playwright
// suites under tests/ are the E2E tier and are not picked up
// here.
const here = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
    plugins: [svelte(), svelteTesting()],
    resolve: {
        alias: {
            $lib: path.resolve(here, 'src/lib')
        }
    },
    test: {
        environment: 'jsdom',
        include: ['src/**/*.spec.ts']
    }
});
