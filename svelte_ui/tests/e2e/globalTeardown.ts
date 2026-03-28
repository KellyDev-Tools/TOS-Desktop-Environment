import { fileURLToPath } from 'url';
import path from 'path';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default async function globalTeardown() {
    console.log('\n[E2E Teardown] Terminating TOS E2E Environment...');
    const rootDir = path.resolve(__dirname, '../../..');
    const pidFile = path.join(rootDir, '.e2e-brain.pid');

    if (fs.existsSync(pidFile)) {
        const pidStr = fs.readFileSync(pidFile, 'utf-8');
        const pid = parseInt(pidStr, 10);

        if (!isNaN(pid)) {
            console.log(`[E2E Teardown] Killing Brain process (PID ${pid})...`);
            try {
                process.kill(pid, 'SIGTERM');

                // Give it a second to shutdown gracefully, then SIGKILL
                setTimeout(() => {
                    try {
                        process.kill(pid, 'SIGKILL');
                    } catch (e) {
                        // Already dead
                    }
                }, 1000);
            } catch (err) {
                console.log(`[E2E Teardown] Could not kill process ${pid}. It may have already exited.`);
            }
        }

        try {
            fs.unlinkSync(pidFile);
        } catch (e) { }
    } else {
        console.log('[E2E Teardown] No Brain PID file found. Skipping cleanup.');
    }
}
