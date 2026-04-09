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

    // 2. Form Submission
    accessForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        
        const service = document.getElementById('service').value;
        const ip = document.getElementById('ip').value.trim();
        const cc_emails_str = document.getElementById('cc_emails').value.trim();
        const cc_emails = cc_emails_str ? cc_emails_str.split(',').map(email => email.trim()) : [];

        // Simple validation
        const ipRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
        if (!ipRegex.test(ip)) {
            showStatus('Invalid IP address format', 'error');
            return;
        }

        submitBtn.disabled = true;
        submitBtn.innerHTML = `<span>Processing...</span><i class="fas fa-spinner animate-spin"></i>`;
        hideStatus();

        try {
            const response = await fetch('/api/access', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip, service, cc_emails })
            });

            const result = await response.json();

            if (response.ok) {
                showStatus(result.message || 'Access request received and is being processed in the background.', 'success');
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
