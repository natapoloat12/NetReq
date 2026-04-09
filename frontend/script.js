document.addEventListener('DOMContentLoaded', async () => {
    const loadingScreen = document.getElementById('loadingScreen');
    const mainContainer = document.getElementById('mainContainer');
    const accessForm = document.getElementById('accessForm');
    const submitBtn = document.getElementById('submitBtn');
    const statusMessage = document.getElementById('statusMessage');
    const statusIcon = document.getElementById('statusIcon');
    const statusText = document.getElementById('statusText');
    const logoutBtn = document.getElementById('logoutBtn');

    // 1. Session Verification
    try {
        const verifyRes = await fetch('/api/verify');
        if (!verifyRes.ok) {
            window.location.href = 'login.html';
            return;
        }
        loadingScreen.classList.add('opacity-0');
        setTimeout(() => {
            loadingScreen.style.display = 'none';
            mainContainer.style.display = 'block';
        }, 300);
    } catch (err) {
        window.location.href = 'login.html';
        return;
    }

    // 2. Form Submission & Multiple IPs
    const ipContainer = document.getElementById('ipContainer');
    const addIpBtn = document.getElementById('addIpBtn');

    addIpBtn.addEventListener('click', () => {
        const row = document.createElement('div');
        row.className = 'ip-row relative group mt-3';
        row.innerHTML = `
            <span class="absolute inset-y-0 left-0 pl-3.5 flex items-center text-slate-400">
                <i class="fas fa-network-wired text-sm"></i>
            </span>
            <input type="text" name="ip[]" required 
                placeholder="e.g. 10.10.x.x"
                class="block w-full pl-10 pr-10 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm transition-all focus:ring-4 focus:ring-blue-500/10 focus:border-blue-500 focus:outline-none">
            <button type="button" class="remove-ip absolute inset-y-0 right-0 pr-3.5 flex items-center text-slate-300 hover:text-red-500 transition-colors">
                <i class="fas fa-times-circle"></i>
            </button>
        `;
        ipContainer.appendChild(row);

        row.querySelector('.remove-ip').addEventListener('click', () => {
            row.remove();
        });
    });

    accessForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        
        const service = document.getElementById('service').value;
        const ipInputs = document.querySelectorAll('input[name="ip[]"]');
        const ips = Array.from(ipInputs).map(input => input.value.trim()).filter(ip => ip !== "");
        
        const cc_emails_str = document.getElementById('cc_emails').value.trim();
        const cc_emails = cc_emails_str ? cc_emails_str.split(',').map(email => email.trim()) : [];

        // Validation
        const ipRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
        
        if (ips.length === 0) {
            showStatus('At least one IP address is required', 'error');
            return;
        }

        for (const ip of ips) {
            if (!ipRegex.test(ip)) {
                showStatus(`Invalid IP address format: ${ip}`, 'error');
                return;
            }
        }

        submitBtn.disabled = true;
        submitBtn.innerHTML = `<span>Processing...</span><i class="fas fa-spinner animate-spin"></i>`;
        hideStatus();

        try {
            const response = await fetch('/api/access', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ips, service, cc_emails })
            });

            const result = await response.json();

            if (response.ok) {
                showStatus(result.message || 'Access request received and is being processed in the background.', 'success');
                // Reset to single IP row
                const rows = document.querySelectorAll('.ip-row');
                rows.forEach((row, index) => {
                    if (index > 0) row.remove();
                    else row.querySelector('input').value = '';
                });
                accessForm.reset();
            } else {
                showStatus(result.message || 'Failed to grant access', 'error');
            }
        } catch (err) {
            showStatus('Network error. Please try again.', 'error');
        } finally {
            submitBtn.disabled = false;
            submitBtn.innerHTML = `<span>Request Access</span><i class="fas fa-chevron-right text-xs"></i>`;
        }
    });

    // 3. Logout
    logoutBtn.addEventListener('click', async () => {
        try {
            await fetch('/api/logout', { method: 'POST' });
            window.location.href = 'login.html';
        } catch (err) {
            window.location.href = 'login.html';
        }
    });

    // Helpers
    function showStatus(message, type) {
        statusText.textContent = message;
        statusMessage.classList.remove('hidden', 'bg-red-50', 'border-red-100', 'text-red-700', 'bg-green-50', 'border-green-100', 'text-green-700');
        
        if (type === 'success') {
            statusMessage.classList.add('bg-green-50', 'border-green-100', 'text-green-700');
            statusIcon.innerHTML = '<i class="fas fa-check-circle text-lg"></i>';
        } else {
            statusMessage.classList.add('bg-red-50', 'border-red-100', 'text-red-700');
            statusIcon.innerHTML = '<i class="fas fa-exclamation-circle text-lg"></i>';
        }
        statusMessage.classList.remove('hidden');
    }

    function hideStatus() {
        statusMessage.classList.add('hidden');
    }
});
