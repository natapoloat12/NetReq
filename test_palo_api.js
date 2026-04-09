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

async function testPaloAlto() {
    const HOST = process.env.PALOALTO_HOST || '10.90.10.2';
    const USER = (process.env.PALOALTO_USER || 'admin').trim();
    const PASS = (process.env.PALOALTO_PASSWORD || 'admin').trim();
    const RULE_NAME = (process.env.PALOALTO_RULE_NAME || '126').trim();
    const VSYS = (process.env.PALOALTO_VSYS || 'vsys1').trim();

    console.log(`--- Palo Alto API Connection Test ---`);
    console.log(`Target: https://${HOST}`);
    console.log(`User: [${USER}] (Length: ${USER.length})`);
    console.log(`Pass: (Length: ${PASS.length})`);
    console.log(`Rule: [${RULE_NAME}] (${VSYS})`);

    // We disable certificate validation for testing purposes if configured
    const fetchOptions = {
        // Note: Node.js fetch doesn't have an easy "danger_accept_invalid_certs" like reqwest
        // but we can use process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0'
    };
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

    try {
        let apiKey = process.env.PALOALTO_API_KEY;
        
        if (apiKey && apiKey.trim() !== '') {
            console.log("\n[1/3] Using existing API key from .env...");
            apiKey = apiKey.trim();
        } else {
            // 1. Keygen Test
            console.log("\n[1/3] Attempting Keygen...");
            const keygenUrl = `https://${HOST}/api/?type=keygen&user=${encodeURIComponent(USER)}&password=${encodeURIComponent(PASS)}`;
            const keygenRes = await fetch(keygenUrl);
            const keygenText = await keygenRes.text();

            if (!keygenRes.ok) {
                throw new Error(`Keygen failed (${keygenRes.status}): ${keygenText}`);
            }

            const keyMatch = keygenText.match(/<key>(.*?)<\/key>/);
            if (!keyMatch) {
                throw new Error(`Could not find <key> in response: ${keygenText}`);
            }
            apiKey = keyMatch[1];
            console.log("SUCCESS: API Key retrieved.");
        }

        // 2. GET Rule Test
        console.log("\n[2/3] Attempting to fetch security rule...");
        const getUrl = `https://${HOST}/restapi/v10.2/Policies/SecurityRules?name=${encodeURIComponent(RULE_NAME)}&location=vsys&vsys=${encodeURIComponent(VSYS)}&output-format=json`;
        
        const getRes = await fetch(getUrl, {
            headers: { 'X-PAN-KEY': apiKey }
        });
        const getData = await getRes.json();

        if (!getRes.ok) {
            throw new Error(`GET Rule failed (${getRes.status}): ${JSON.stringify(getData)}`);
        }

        console.log("SUCCESS: Rule fetched.");
        
        // Find entry
        let entry = null;
        if (getData.result && getData.result.entry && getData.result.entry[0]) {
            entry = getData.result.entry[0];
        } else if (getData.entry) {
            entry = getData.entry;
        }

        if (!entry) {
            throw new Error(`Rule '${RULE_NAME}' not found in the response.`);
        }

        console.log("Rule Details:");
        console.log(` - Name: ${entry['@name']}`);
        console.log(` - Current Source Members: ${JSON.stringify(entry.source.member)}`);

        // 3. Validation
        console.log("\n[3/3] Validating JSON structure for PUT update...");
        if (!entry.source || !Array.isArray(entry.source.member)) {
            throw new Error("Invalid structure: 'source.member' is missing or not an array.");
        }
        console.log("SUCCESS: JSON structure is compatible with our update logic.");

    } catch (err) {
        console.error("\nFAILED:", err.message);
        if (err.message.includes('fetch failed')) {
            console.error("Tip: Check if the Palo Alto management IP is reachable from this machine.");
        }
    }
}

testPaloAlto();
