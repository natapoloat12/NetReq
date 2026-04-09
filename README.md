# Anydesk Access Portal

Self-service portal for temporary Anydesk access by appending IP addresses to existing firewall policies.

## Project Structure

```text
AnydeskAccess/
├── backend/
│   ├── src/
│   │   ├── auth/          # LDAP & JWT logic
│   │   ├── mailer/        # SMTP notifications
│   │   ├── firewall/      # FortiGate & Palo Alto (Pluggable)
│   │   ├── handlers/      # API endpoints
│   │   ├── models/        # Data structures
│   │   ├── middleware/    # Auth middleware
│   │   └── main.rs        # Entry point
│   └── Cargo.toml
├── frontend/              # Simple HTML/JS UI
└── .env                   # Configuration
```

## Features

- **LDAP Integration**: Authenticate with existing Windows credentials.
- **Firewall Integration**:
  - **FortiGate**: Automatically creates address objects and appends them to a pre-defined policy.
  - **Palo Alto**: Pluggable module with placeholder for future integration.
- **Notifications**: Sends HTML email notifications to the user, admins, and CC recipients.
- **Modern UI**: Clean, responsive frontend built with Tailwind CSS.

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js (optional, for serving frontend during development) or any web server.

### Configuration

1. Copy `.env` to the root of the project (if not already present).
2. Configure the following variables:
   - `PORT`: Port to run the backend (Default: `5051`).
   - `LDAP_URL`: Your LDAP server URL.
   - `SMTP_*`: Your SMTP server settings.
   - `FORTIGATE_*`: FortiGate API details and the **Policy ID** to append IPs to.
   - `FIREWALL_TYPE`: Set to `fortigate` or `paloalto`.

### Running the Backend

```bash
cd backend
cargo run
```

### Running the Frontend

The backend does not serve static files by default (unless configured). You can use Nginx or any simple static server to serve the `frontend/` directory.

To test locally, you might need a proxy or enable CORS (already enabled in `main.rs` for development).

## Workflow Logic

1. User logs in via LDAP.
2. User submits their IP address and optional CC emails.
3. Backend checks if the IP already has an address object on the firewall.
4. If not, it creates a new address object (`ADDR_x_x_x_x`).
5. Backend appends the address object to the configured firewall policy ID.
6. A success response is returned, and an email notification is sent.
