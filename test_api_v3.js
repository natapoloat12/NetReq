const BASE_URL = 'http://localhost:5051/api';

async function runTest() {
    console.log("--- Multi-Service API Test (TeamViewer) ---");
    
    // 1. Login
    const loginRes = await fetch(`${BASE_URL}/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: "testuser", password: "testpassword" })
    });
    const cookieHeader = loginRes.headers.get('set-cookie');

    // 2. Request Access for TeamViewer
    const randomIP = `10.200.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`;
    console.log(`\nRequesting TeamViewer access for IP ${randomIP}...`);
    
    const accessRes = await fetch(`${BASE_URL}/access`, {
        method: 'POST',
        headers: { 
            'Content-Type': 'application/json',
            'Cookie': cookieHeader
        },
        body: JSON.stringify({
            ip: randomIP,
            service: "teamviewer",
            cc_emails: ["sysmon@kce.co.th"]
        })
    });

    const accessData = await accessRes.json();
    console.log("Access Status:", accessRes.status);
    console.log("Access Response:", accessData);

    if (accessRes.ok) {
        console.log("\nSUCCESS: TeamViewer rule update triggered!");
    } else {
        console.error("\nFAILED: Check logs for Palo Alto rule name mismatch.");
    }
}

runTest().catch(console.error);
