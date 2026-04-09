const fs = require('fs');
const path = require('path');

async function testApi() {
    console.log("--- Testing Anydesk Access API (Node.js) ---");
    const PORT = 5051;
    const BASE_URL = `http://localhost:${PORT}/api`;

    const loginPayload = {
        username: "testuser",
        password: "testpassword"
    };

    try {
        console.log(`POST ${BASE_URL}/login...`);
        const loginRes = await fetch(`${BASE_URL}/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(loginPayload)
        });

        console.log(`Status: ${loginRes.status}`);
        const loginData = await loginRes.json();
        console.log("Response:", loginData);

        if (loginRes.ok) {
            // Further tests would go here if login succeeded
        }
    } catch (err) {
        console.error("\nERROR: Could not connect to the server. Please start the backend with 'cargo run' first.");
        console.error("The script is ready, but the server is not running because 'cargo' is not installed in the environment.");
    }
}

testApi();
