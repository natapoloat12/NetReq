const BASE_URL = 'http://localhost:5052/api';

async function runTest() {
    console.log("--- Service Test (Internet for Server) ---");
    
    // 1. Login
    const loginRes = await fetch(`${BASE_URL}/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: "testuser", password: "testpassword" })
    });
    const cookieHeader = loginRes.headers.get('set-cookie');

    // 2. Request Access for Multiple IPs
    const randomIP1 = `10.155.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`;
    const randomIP2 = `10.155.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`;
    console.log(`\nRequesting Internet access for IPs ${randomIP1}, ${randomIP2}...`);
    
    const accessRes = await fetch(`${BASE_URL}/access`, {
        method: 'POST',
        headers: { 
            'Content-Type': 'application/json',
            'Cookie': cookieHeader
        },
        body: JSON.stringify({
            ips: [randomIP1, randomIP2],
            service: "internet",
            cc_emails: ["sysmon@kce.co.th"]
        })
    });

    const accessData = await accessRes.json();
    console.log("Access Status:", accessRes.status);
    console.log("Access Response:", accessData);

    if (accessRes.ok) {
        console.log("\nSUCCESS: Internet for Server rule update triggered!");
        console.log("Check logs to confirm FortiGate was skipped.");
    } else {
        console.error("\nFAILED: Firewall update failed.");
    }
}

runTest().catch(console.error);
