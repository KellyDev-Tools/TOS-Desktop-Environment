import { defineConfig, devices } from '@playwright/test';
import { fileURLToPath } from 'url';
import path from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default defineConfig({
    globalSetup: path.resolve(__dirname, 'tests/e2e/globalSetup.ts'),
    globalTeardown: path.resolve(__dirname, 'tests/e2e/globalTeardown.ts'),

    testDir: './tests/e2e',
    testMatch: /(.+\.)?(test|spec)\.[jt]s/,

    // Run tests serially as they all share the same local Brain instance
    workers: 1,

    use: {
        baseURL: 'http://127.0.0.1:4173',
        trace: 'retain-on-failure',
        ignoreHTTPSErrors: true,
        video: 'on-first-retry',
        launchOptions: {
            args: ['--ignore-certificate-errors']
        }
    },

    webServer: {
        command: 'npm run build && npm run preview',
        port: 4173,
        reuseExistingServer: !process.env.CI,
    },
});
