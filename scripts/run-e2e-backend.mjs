import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';

const DB_FILE = 'data/kestrel-e2e.db';
const PORT = '8080';
const SEED_PASSWORD = 'e2e-password-123';

const env = {
    ...process.env,
    DATABASE_URL: `sqlite:./${DB_FILE}`,
    PORT,
    HOST: '0.0.0.0',
    KESTREL_SEED_PASSWORD: SEED_PASSWORD,
    JWT_SECRET: 'e2e-jwt-secret-12345678901234567890',
    VAULT_SECRET: 'e2e-vault-secret-12345678901234567890'
};

if (fs.existsSync(`backend/${DB_FILE}`)) {
    fs.rmSync(`backend/${DB_FILE}`);
}

console.log('Running database migrations...');
spawnSync('mise', ['run', 'db:migrate'], { env, stdio: 'inherit', cwd: 'backend' });

console.log('Seeding database...');
spawnSync('cargo', ['run', '--bin', 'seed'], { env: { ...env, SQLX_OFFLINE: 'true' }, stdio: 'inherit', cwd: 'backend' });

console.log('Starting backend server...');
const server = spawn('cargo', ['run', '--bin', 'kestrel-server'], { env, cwd: 'backend' });

let backendOutput = '';
server.stdout.on('data', (data) => {
    backendOutput += data.toString();
});
server.stderr.on('data', (data) => {
    backendOutput += data.toString();
});

const checkHealth = async () => {
    for (let i = 0; i < 30; i++) {
        try {
            const res = await fetch(`http://127.0.0.1:${PORT}/api/v1/health`);
            if (res.ok) {
                console.log('Backend is healthy.');
                return true;
            }
        } catch (e) {
            // connection refused, expected initially
        }
        await new Promise(r => setTimeout(r, 1000));
    }
    return false;
};

checkHealth().then((isHealthy) => {
    if (!isHealthy) {
        console.error('Backend failed to start in time. Logs:');
        console.error(backendOutput);
        server.kill();
        process.exit(1);
    }

    console.log('Running Playwright tests...');
    spawnSync('mise', ['run', 'test:e2e:install'], { env, stdio: 'inherit' });
    const pwResult = spawnSync('mise', ['run', 'test:e2e'], { env, stdio: 'inherit' });

    server.kill();

    if (pwResult.status !== 0) {
        console.error('Tests failed. Backend Logs:');
        console.error(backendOutput);
        process.exit(pwResult.status ?? 1);
    }
});
