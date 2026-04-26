import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import path from 'path';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default async function globalSetup() {
    console.log('\n[E2E Setup] Starting TOS E2E Environment...');

    const rootDir = path.resolve(__dirname, '../../..');

    console.log('[E2E Setup] Cleaning up existing TOS processes...');
    await new Promise<void>((resolve) => {
        const cleanup = spawn('pkill', ['-9', 'tos-'], { stdio: 'inherit' });
        cleanup.on('close', () => resolve());
    });


    const sentinelFile = path.join(rootDir, '.e2e_ready');
    const logsDir = path.join(rootDir, 'logs');
    if (!fs.existsSync(logsDir)) fs.mkdirSync(logsDir);

    if (fs.existsSync(sentinelFile)) {
        console.log('[E2E Setup] Environment marked as READY via sentinel. Skipping heavy build phase.');
        console.log('[E2E Setup] Spawning Auxiliary Daemons (pre-compiled)...');
        
        const daemons = [
            'tos-settingsd',
            'tos-loggerd',
            'tos-marketplaced',
            'tos-priorityd',
            'tos-sessiond',
            'tos-heuristicd',
            'tos-searchd'
        ];

        for (const daemon of daemons) {
            const bin = path.join(rootDir, 'target/debug', daemon);
            if (fs.existsSync(bin)) {
                const logStream = fs.createWriteStream(path.join(logsDir, `${daemon}.log`));
                const proc = spawn(bin, [], { 
                    cwd: rootDir, 
                    stdio: ['ignore', logStream, logStream],
                    detached: true 
                });
                proc.unref();
            } else {
                console.warn(`[E2E Setup] Daemon binary not found: ${bin}`);
            }
        }
    } else {
        console.log('[E2E Setup] Initializing Auxiliary Daemons via Makefile...');
        // Run make run-services which builds and spawns all daemons.
        // We'll wait for it to finish its build phase.
        await new Promise<void>((resolve, reject) => {
            const services = spawn('make', ['run-services'], { cwd: rootDir, stdio: 'inherit' });
            services.on('close', (code) => {
                if (code === 0) resolve();
                else reject(new Error(`make run-services failed with code ${code}`));
            });
        });
    }

    // Removed redundant cargo build for tos-brain here as make run-services already handles it.
 
    // Cleanup existing session data for a truly fresh start
    const sessionDir = path.join(process.env.HOME || '/root', '.local/share/tos/sessions');
    if (fs.existsSync(sessionDir)) {
        fs.readdirSync(sessionDir).forEach(f => fs.unlinkSync(path.join(sessionDir, f)));
    }

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
