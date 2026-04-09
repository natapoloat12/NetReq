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

async function findRuleByIndex() {
    const HOST = process.env.PALOALTO_HOST || '10.90.10.2';
    const USER = process.env.PALOALTO_USER || 'admin';
    const PASS = process.env.PALOALTO_PASSWORD || 'admin';
    const VSYS = process.env.PALOALTO_VSYS || 'vsys1';

    console.log(`--- Palo Alto Rule Index Lookup ---`);
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

    try {
        console.log("Getting API Key...");
        const keygenUrl = `https://${HOST}/api/?type=keygen&user=${encodeURIComponent(USER)}&password=${encodeURIComponent(PASS)}`;
        const keygenRes = await fetch(keygenUrl);
        const keygenText = await keygenRes.text();
        const apiKey = keygenText.match(/<key>(.*?)<\/key>/)[1];

        console.log("Fetching all rules...");
        const listUrl = `https://${HOST}/restapi/v10.2/Policies/SecurityRules?location=vsys&vsys=${encodeURIComponent(VSYS)}&output-format=json`;
        
        const listRes = await fetch(listUrl, {
            headers: { 'X-PAN-KEY': apiKey }
        });
        const listData = await listRes.json();
        const rules = listData.result.entry || [];

        console.log(`\nRule at Index 125 (0-indexed, pos 126):`);
        const rule126 = rules[125];
        if (rule126) {
            console.log(JSON.stringify(rule126, null, 2));
        } else {
            console.log("Rule not found at index 125.");
        }

        console.log(`\nRule at Index 126 (0-indexed, pos 127):`);
        const rule127 = rules[126];
        if (rule127) {
            console.log(JSON.stringify(rule127, null, 2));
        } else {
            console.log("Rule not found at index 126.");
        }

    } catch (err) {
        console.error("\nFAILED:", err.message);
    }
}

findRuleByIndex();
