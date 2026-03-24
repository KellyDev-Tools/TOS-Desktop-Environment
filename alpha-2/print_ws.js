const WebSocket = require('ws');
const ws = new WebSocket('ws://127.0.0.1:7001');
ws.on('open', () => {
    ws.send('sync');
});
ws.on('message', (data) => {
    console.log(JSON.parse(data));
    process.exit(0);
});
