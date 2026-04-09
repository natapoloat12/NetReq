const fs = require('fs');

// Manual .env parsing
if (fs.existsSync('.env')) {
    const content = fs.readFileSync('.env', 'utf8');
    content.split('\n').forEach(line => {
        const [key, ...valueParts] = line.split('=');
        if (key && valueParts.length > 0) {
            process.env[key.trim()] = valueParts.join('=').trim();
        }
    });
}

async function testPaloAltoFullCycle() {
    const HOST = process.env.PALOALTO_HOST || '10.90.10.2';
    const USER = process.env.PALOALTO_USER || 'admin';
    const PASS = process.env.PALOALTO_PASSWORD || 'admin';
    const RULE_NAME = 'T2U-Allow-anydesk'; // Verified correct name
    const VSYS = process.env.PALOALTO_VSYS || 'vsys1';
    const TEST_IP = '10.99.99.99';

    console.log(`--- Palo Alto API Full Cycle Test ---`);
    console.log(`Target Rule: ${RULE_NAME}`);
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

    try {
        console.log("\n[1/4] Getting API Key...");
        const keygenUrl = `https://${HOST}/api/?type=keygen&user=${encodeURIComponent(USER)}&password=${encodeURIComponent(PASS)}`;
        const keygenRes = await fetch(keygenUrl);
        const keygenText = await keygenRes.text();
        const apiKey = keygenText.match(/<key>(.*?)<\/key>/)[1];
        console.log("SUCCESS");

        console.log("\n[2/4] Fetching current rule state...");
        const getUrl = `https://${HOST}/restapi/v10.2/Policies/SecurityRules?name=${encodeURIComponent(RULE_NAME)}&location=vsys&vsys=${encodeURIComponent(VSYS)}&output-format=json`;
        const getRes = await fetch(getUrl, { headers: { 'X-PAN-KEY': apiKey } });
        const getData = await getRes.json();
        const entry = getData.result ? getData.result.entry[0] : getData.entry;
        const originalMembers = [...entry.source.member];
        console.log(`Current members: ${JSON.stringify(originalMembers)}`);

        console.log("\n[3/4] Attempting to append test IP...");
        if (originalMembers.includes(TEST_IP)) {
            console.log("Test IP already exists, skipping append test.");
        } else {
            const newMembers = [...originalMembers, TEST_IP];
            entry.source.member = newMembers;
            
            const putUrl = `https://${HOST}/restapi/v10.2/Policies/SecurityRules?name=${encodeURIComponent(RULE_NAME)}&location=vsys&vsys=${encodeURIComponent(VSYS)}&input-format=json&output-format=json`;
            const putRes = await fetch(putUrl, {
                method: 'PUT',
                headers: { 'X-PAN-KEY': apiKey, 'Content-Type': 'application/json' },
                body: JSON.stringify({ entry })
            });

            if (putRes.ok) {
                console.log(`SUCCESS: Appended ${TEST_IP}`);
            } else {
                const errText = await putRes.text();
                throw new Error(`Append failed (${putRes.status}): ${errText}`);
            }
        }

        console.log("\n[4/4] Rolling back changes (restoring original members)...");
        entry.source.member = originalMembers;
        const rollbackRes = await fetch(`https://${HOST}/restapi/v10.2/Policies/SecurityRules?name=${encodeURIComponent(RULE_NAME)}&location=vsys&vsys=${encodeURIComponent(VSYS)}&input-format=json&output-format=json`, {
            method: 'PUT',
            headers: { 'X-PAN-KEY': apiKey, 'Content-Type': 'application/json' },
            body: JSON.stringify({ entry })
        });

        if (rollbackRes.ok) {
            console.log("SUCCESS: Original rule state restored.");
        } else {
            console.error("CRITICAL: Rollback failed. Please check rule manually.");
        }

    } catch (err) {
        console.error("\nFAILED:", err.message);
    }
}

testPaloAltoFullCycle();
