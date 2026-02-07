#!/usr/bin/env node

/**
 * Test script to verify billing API fix
 * This simulates the frontend calls to test the plan fetching and QR generation
 */

const http = require('http');

// Helper function to make HTTP requests
function makeRequest(options, data = null) {
    return new Promise((resolve, reject) => {
        const req = http.request(options, (res) => {
            let body = '';
            res.on('data', (chunk) => {
                body += chunk;
            });
            res.on('end', () => {
                resolve({
                    statusCode: res.statusCode,
                    headers: res.headers,
                    body: body
                });
            });
        });

        req.on('error', (err) => {
            reject(err);
        });

        if (data) {
            req.write(JSON.stringify(data));
        }
        req.end();
    });
}

async function testBillingAPI() {
    console.log('🧪 Testing Billing API Fix...\n');

    try {
        // Test 1: Get plans from backend
        console.log('1. Testing GET /api/billing/plans');
        try {
            const plansResponse = await makeRequest({
                hostname: 'localhost',
                port: 8080,
                path: '/api/billing/plans',
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            console.log(`   Status: ${plansResponse.statusCode}`);
            if (plansResponse.statusCode === 200) {
                const plans = JSON.parse(plansResponse.body);
                console.log('   ✅ Plans fetched successfully:');
                console.log('   Plans data:', JSON.stringify(plans, null, 2));
                
                // Test 2: Generate QR with UUID (if we have plans)
                if (plans.plans && plans.plans.length > 0) {
                    const firstPlan = plans.plans[0];
                    console.log(`\n2. Testing POST /api/billing/generate-qr with UUID: ${firstPlan.id}`);
                    
                    try {
                        const qrResponse = await makeRequest({
                            hostname: 'localhost',
                            port: 8080,
                            path: '/api/billing/generate-qr',
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                                'Authorization': 'Bearer fake-token-for-test'
                            }
                        }, {
                            plan_id: firstPlan.id,
                            amount: firstPlan.price
                        });

                        console.log(`   Status: ${qrResponse.statusCode}`);
                        console.log('   Response:', qrResponse.body);
                        
                        if (qrResponse.statusCode === 200) {
                            console.log('   ✅ QR generation successful with UUID!');
                        } else if (qrResponse.statusCode === 400 && qrResponse.body.includes('INVALID_PLAN_ID')) {
                            console.log('   ❌ Still getting INVALID_PLAN_ID error');
                        } else {
                            console.log('   ⚠️  Unexpected response');
                        }
                    } catch (error) {
                        console.log('   ❌ QR generation failed:', error.message);
                    }
                }
            } else {
                console.log('   ❌ Failed to fetch plans');
                console.log('   Response:', plansResponse.body);
            }
        } catch (error) {
            console.log('   ❌ Request failed:', error.message);
            console.log('   💡 Note: API server might not be running on localhost:8080');
        }

    } catch (error) {
        console.error('Test failed:', error);
    }
}

// Run the test
testBillingAPI();
