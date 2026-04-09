# AnydeskAccess Project Architecture

## 1. Overview
AnydeskAccess is a secure self-service portal designed to grant temporary firewall access for specific remote desktop services (AnyDesk, TeamViewer) and general Internet access. It integrates with enterprise infrastructure including LDAP for authentication and multiple firewall vendors (Palo Alto and FortiGate) for policy enforcement.

## 2. System Architecture

The project follows a standard three-tier architecture, containerized using Docker.

### 2.1. Infrastructure Layer
- **Docker Compose**: Orchestrates the entire stack.
- **Nginx**: Acts as a reverse proxy and static file server.
  - Serves the HTML/JS frontend.
  - Routes `/api/*` requests to the Rust backend.
  - Handles SSL termination (configurable).
- **Network**: A private bridge network (`anydesk_network`) facilitates communication between the frontend (Nginx) and backend.

### 2.2. Frontend Layer (Vanilla JS)
- **Technology**: HTML5, CSS3 (with TailwindCSS via CDN), and Vanilla JavaScript.
- **Features**:
  - Responsive login page.
  - Access request form with IP validation.
  - Real-time status feedback for API operations.
  - Session persistence via HTTP-only cookies.

### 2.3. Backend Layer (Rust)
Built with the **Axum** framework, focusing on safety, performance, and concurrency.

#### Key Components:
- **Auth Module**:
  - **LDAP**: Integrates with Active Directory/LDAP for user authentication.
  - **JWT**: Issues and validates JSON Web Tokens for session management.
- **Firewall Module**:
  - **FirewallProvider Trait**: An abstraction layer for firewall operations.
  - **PaloAltoClient**: Communicates with Palo Alto PAN-OS via REST and XML APIs. Supports policy updates and configuration commits.
  - **FortiGateClient**: Communicates with FortiOS via the REST API to manage address objects and policy members.
  - **MultiFirewallProvider**: A wrapper that allows simultaneous updates to multiple firewall systems.
- **Handlers**: Orchestrates the business logic (Authentication, Access Requests).
- **Middleware**: Custom JWT authentication middleware protecting sensitive routes.
- **Mailer**: SMTP integration via the `lettre` crate for sending access notifications.

## 3. Data Flow

### 3.1. Authentication Flow
1. User enters LDAP credentials in the frontend.
2. Frontend sends credentials to `/api/login`.
3. Backend validates credentials against the LDAP server.
4. Upon success, Backend generates a JWT containing user claims (username, email, etc.).
5. Backend sets an `HTTP-Only`, `Secure` cookie containing the JWT.

### 3.2. Access Request Flow
1. User submits an IP address and service type (e.g., AnyDesk).
2. Frontend sends a POST request to `/api/access` with the JWT cookie.
3. **Auth Middleware** validates the JWT and extracts user information.
4. **Firewall Module** determines which policy to update based on the requested service.
5. **Firewall Implementation** (Palo Alto/FortiGate) performs the update:
   - Palo Alto: Updates the security rule and triggers a `commit`.
   - FortiGate: Ensures an address object exists and adds it to the policy.
6. **Mailer Module** sends a confirmation email to the user and any CC'd addresses.
7. Backend returns a success/failure response to the frontend.

## 4. Security Considerations
- **Secure Sessions**: JWTs are stored in `HTTP-Only` cookies to prevent XSS-based token theft.
- **Input Validation**: Rigorous IP and email validation on both frontend and backend.
- **TLS/SSL**: Support for certificate validation for all outbound API calls (LDAP, Firewalls, SMTP).
- **Environment Isolation**: All sensitive configurations (API keys, passwords, hostnames) are managed via `.env` files and never hardcoded.
- **Audit Logging**: Structured logging (via `tracing`) records all access requests and system operations.

## 5. Technology Stack
- **Frontend**: HTML, Vanilla JS, TailwindCSS.
- **Backend**: Rust (Axum, Tokio, Serde, Reqwest).
- **Authentication**: LDAP3, Jsonwebtoken.
- **Email**: Lettre.
- **Reverse Proxy**: Nginx.
- **Deployment**: Docker, Docker Compose.
