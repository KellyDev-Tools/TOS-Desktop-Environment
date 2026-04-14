
const WebSocket = require('ws');

const url = 'ws://127.0.0.1:7001';
console.log(`Connecting to ${url}...`);

const ws = new WebSocket(url);

ws.on('open', () => {
    console.log('✅ Connected!');
    ws.send('get_state:');
});

ws.on('message', (data) => {
    console.log('Received:', data.toString());
    process.exit(0);
});

ws.on('error', (err) => {
    console.error('❌ Error:', err.message);
    process.exit(1);
});

setTimeout(() => {
    console.error('⌛ Timeout');
    process.exit(1);
}, 5000);
