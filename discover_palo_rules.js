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

async function listPaloAltoRules() {
    const HOST = process.env.PALOALTO_HOST || '10.90.10.2';
    const USER = process.env.PALOALTO_USER || 'admin';
    const PASS = process.env.PALOALTO_PASSWORD || 'admin';
    const RULE_NAME = process.env.PALOALTO_RULE_NAME || '126';
    const VSYS = process.env.PALOALTO_VSYS || 'vsys1';

    console.log(`--- Palo Alto API Rule Discovery ---`);
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

    try {
        console.log("[1/2] Getting API Key...");
        const keygenUrl = `https://${HOST}/api/?type=keygen&user=${encodeURIComponent(USER)}&password=${encodeURIComponent(PASS)}`;
        const keygenRes = await fetch(keygenUrl);
        const keygenText = await keygenRes.text();
        const apiKey = keygenText.match(/<key>(.*?)<\/key>/)[1];

        console.log("[2/2] Fetching ALL security rules to find the correct name...");
        const listUrl = `https://${HOST}/restapi/v10.2/Policies/SecurityRules?location=vsys&vsys=${encodeURIComponent(VSYS)}&output-format=json`;
        
        const listRes = await fetch(listUrl, {
            headers: { 'X-PAN-KEY': apiKey }
        });
        const listData = await listRes.json();

        if (!listRes.ok) {
            throw new Error(`List failed (${listRes.status}): ${JSON.stringify(listData)}`);
        }

        const rules = listData.result.entry || [];
        console.log(`\nFound ${rules.length} rules in ${VSYS}:`);
        rules.forEach(r => {
            console.log(` - ${r['@name']}`);
        });

        const exists = rules.find(r => r['@name'] === RULE_NAME);
        if (exists) {
            console.log(`\nExact match for '${RULE_NAME}' FOUND.`);
        } else {
            console.log(`\nExact match for '${RULE_NAME}' NOT FOUND.`);
            console.log("Please verify the rule name in Palo Alto Web UI.");
        }

    } catch (err) {
        console.error("\nFAILED:", err.message);
    }
}

listPaloAltoRules();
