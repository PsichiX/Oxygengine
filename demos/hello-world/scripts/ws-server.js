#!/usr/bin/env node

var port = 8090,
  WebSocketServer = require('ws').Server,
  wss = new WebSocketServer({ port });

console.log('Listening on port: ' + port);

wss.on('connection', function(ws) {
  console.log('new client connected!');
  let buff = Buffer.alloc(32);
  buff.writeUInt32BE(10, 0);
  buff.writeUInt32BE(0, 4);
  buff.writeUInt32BE(42, 8);
  buff.writeUInt32BE(1, 12);
  buff.writeUInt32BE(2, 16);
  buff.writeUInt32BE(3, 24);
  ws.send(buff);
  console.log('sent message:', buff);
});
