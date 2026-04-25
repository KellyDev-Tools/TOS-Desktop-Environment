import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import path from 'path';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default async function globalSetup() {
    console.log('\n[E2E Setup] Starting TOS E2E Environment...');

    const rootDir = path.resolve(__dirname, '../../..');

    console.log('[E2E Setup] Initializing Auxiliary Daemons via Makefile...');
    await new Promise<void>((resolve, reject) => {
        const services = spawn('make', ['run-services'], { cwd: rootDir, stdio: 'inherit' });
        services.on('close', (code) => {
            if (code === 0) resolve();
            else reject(new Error(`make run-services failed with code ${code}`));
        });
    });

    console.log('[E2E Setup] Compiling Brain daemon...');
    await new Promise<void>((resolve, reject) => {
        const build = spawn('cargo', ['build', '--bin', 'tos-brain'], { cwd: rootDir, stdio: 'inherit' });
        build.on('close', (code) => {
            if (code === 0) resolve();
            else reject(new Error(`Cargo build failed with code ${code}`));
        });
    });

    console.log('[E2E Setup] Spawning Brain headless daemon directly...');

    const pidFile = path.join(rootDir, '.e2e-brain.pid');
    const binPath = path.join(rootDir, 'target/debug/tos-brain');

    return new Promise<void>((resolve, reject) => {
        const brainProcess = spawn(binPath, ['--headless'], {
            cwd: rootDir,
            stdio: ['ignore', 'pipe', 'pipe'],
            env: { ...process.env, TOS_ANCHOR_PORT: '7001' } // Force fixed port for tests
        });

        if (brainProcess.pid) {
            fs.writeFileSync(pidFile, brainProcess.pid.toString());
        }

        let resolved = false;

        brainProcess.stdout?.on('data', (data) => {
            const str = data.toString();
            // Wait for Brain's startup log (case-insensitive check)
            if (str.toLowerCase().includes('[brain]') || str.toLowerCase().includes('listening')) {
                if (!resolved) {
                    resolved = true;
                    console.log('[E2E Setup] Brain is responding.');
                    resolve();
                }
            }
        });

        brainProcess.stderr?.on('data', (data) => {
            // Echo stderr for debugging
            console.error(`[Brain] ${data.toString().trim()}`);
            if (data.toString().includes('address in use')) {
                console.error('[E2E Setup] ERROR: Port already bound. Teardown may have failed previously.');
            }
        });

        brainProcess.on('error', (err) => {
            if (!resolved) {
                resolved = true;
                reject(err);
            }
        });

        setTimeout(() => {
            if (!resolved) {
                console.log('[E2E Setup] Timeout waiting for Brain. Proceeding anyway...');
                resolved = true;
                resolve();
            }
        }, 8000);
    });
}
