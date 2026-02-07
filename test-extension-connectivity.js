#!/usr/bin/env node

// Test VSCode extension connectivity to production API
const https = require('https');

const options = {
  hostname: 'localhost',
  port: 443,
  path: '/api/health',
  method: 'GET',
  rejectUnauthorized: false // For testing with self-signed certs
};

const req = https.request(options, (res) => {
  console.log(`STATUS: ${res.statusCode}`);
  console.log(`HEADERS: ${JSON.stringify(res.headers)}`);
  
  res.on('data', (d) => {
    console.log(`BODY: ${d}`);
  });
});

req.on('error', (e) => {
  console.error(`PROBLEM: ${e.message}`);
});

req.end();
