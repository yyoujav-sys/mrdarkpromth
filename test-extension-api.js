#!/usr/bin/env node

// Extension API Contract Test
const https = require('https');

// Get auth token first
function getAuthToken() {
    return new Promise((resolve, reject) => {
        const data = JSON.stringify({
            email: 'sretest@bt-shop-dark.online',
            password: 'SecureTestPass123!'
        });

        const options = {
            hostname: 'bt-shop-dark.online',
            port: 443,
            path: '/api/auth/login',
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': data.length
            },
            rejectUnauthorized: false
        };

        const req = https.request(options, (res) => {
            let body = '';
            res.on('data', (chunk) => body += chunk);
            res.on('end', () => {
                try {
                    const response = JSON.parse(body);
                    resolve(response.token);
                } catch (e) {
                    reject(e);
                }
            });
        });

        req.on('error', reject);
        req.write(data);
        req.end();
    });
}

// Test extension API contract
async function testExtensionAPI() {
    try {
        console.log('🔍 Testing Extension API Contract...');
        
        // Get auth token
        const token = await getAuthToken();
        console.log('✅ Auth token obtained');
        
        // Test chat endpoint (extension's main API)
        const chatData = JSON.stringify({
            message: 'Test message from VSCode extension SRE audit',
            user_tier: 'free',
            model: 'gpt-3.5-turbo'
        });

        const options = {
            hostname: 'bt-shop-dark.online',
            port: 443,
            path: '/api/chat',
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`,
                'Content-Length': chatData.length,
                'User-Agent': 'VSCode-Extension/1.0.0'
            },
            rejectUnauthorized: false
        };

        const response = await new Promise((resolve, reject) => {
            const req = https.request(options, (res) => {
                let body = '';
                res.on('data', (chunk) => body += chunk);
                res.on('end', () => {
                    resolve({
                        statusCode: res.statusCode,
                        headers: res.headers,
                        body: body
                    });
                });
            });

            req.on('error', reject);
            req.write(chatData);
            req.end();
        });

        console.log(`📊 Response Status: ${response.statusCode}`);
        console.log(`📋 Response Headers:`, JSON.stringify(response.headers, null, 2));
        
        // Validate response format
        try {
            const responseData = JSON.parse(response.body);
            console.log('✅ Response is valid JSON');
            console.log('📝 Response Structure:', Object.keys(responseData));
            
            // Check for expected fields
            const expectedFields = ['response', 'conversation_id'];
            const hasExpectedFields = expectedFields.every(field => field in responseData);
            console.log(hasExpectedFields ? '✅ Expected fields present' : '❌ Missing expected fields');
            
            console.log('🎯 Extension API Contract Test:', hasExpectedFields ? 'PASS' : 'FAIL');
            
        } catch (e) {
            console.log('❌ Invalid JSON response:', response.body);
            console.log('🎯 Extension API Contract Test: FAIL');
        }
        
    } catch (error) {
        console.error('❌ Extension API Test Failed:', error.message);
        console.log('🎯 Extension API Contract Test: FAIL');
    }
}

testExtensionAPI();
