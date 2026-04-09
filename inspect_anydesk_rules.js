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

async function inspectAnydeskRules() {
    const HOST = process.env.PALOALTO_HOST || '10.90.10.2';
    const USER = process.env.PALOALTO_USER || 'admin';
    const PASS = process.env.PALOALTO_PASSWORD || 'admin';
    const VSYS = process.env.PALOALTO_VSYS || 'vsys1';

    console.log(`--- Palo Alto Anydesk Rules Inspection ---`);
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

    try {
        console.log("Getting API Key...");
        const keygenUrl = `https://${HOST}/api/?type=keygen&user=${encodeURIComponent(USER)}&password=${encodeURIComponent(PASS)}`;
        const keygenRes = await fetch(keygenUrl);
        const keygenText = await keygenRes.text();
        const apiKey = keygenText.match(/<key>(.*?)<\/key>/)[1];

        const targetRules = ['T2U-Allow-anydesk', 'D2U-Allow-anydesk', 'V2U-Allow-anydesk'];

        for (const ruleName of targetRules) {
            console.log(`\nInspecting Rule: ${ruleName}`);
            const getUrl = `https://${HOST}/restapi/v10.2/Policies/SecurityRules?name=${encodeURIComponent(ruleName)}&location=vsys&vsys=${encodeURIComponent(VSYS)}&output-format=json`;
            
            const res = await fetch(getUrl, {
                headers: { 'X-PAN-KEY': apiKey }
            });
            const data = await res.json();

            if (res.ok) {
                const entry = data.result ? data.result.entry[0] : data.entry;
                console.log(JSON.stringify(entry, null, 2));
            } else {
                console.log(`Failed to fetch ${ruleName}: ${res.status}`);
            }
        }

    } catch (err) {
        console.error("\nFAILED:", err.message);
    }
}

inspectAnydeskRules();
