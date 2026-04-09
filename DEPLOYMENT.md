# Deployment Guide: AnydeskAccess on Ubuntu Server

This guide provides step-by-step instructions to deploy the AnydeskAccess portal on a fresh Ubuntu 22.04 or 24.04 LTS server.

## 1. System Preparation

First, update your system packages:

```bash
sudo apt update && sudo apt upgrade -y
```

Install essential tools:

```bash
sudo apt install -y git curl ca-certificates gnupg lsb-release
```

## 2. Install Docker & Docker Compose

Follow the official Docker installation steps:

```bash
# Add Docker's official GPG key:
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

# Add the repository to Apt sources:
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

sudo apt update

# Install Docker packages:
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

Verify installation:
```bash
docker --version
docker compose version
```

## 3. Clone and Configure the Project

Clone the repository to your preferred directory (e.g., `/opt`):

```bash
sudo mkdir -p /opt/anydesk-access
sudo chown $USER:$USER /opt/anydesk-access
cd /opt/anydesk-access

git clone <your-repository-url> .
```

### Configure Environment Variables

Copy the template and edit it with your production values:

```bash
cp .env.example .env
nano .env
```

**Critical settings to update:**
- `JWT_SECRET`: Use a strong random string (`openssl rand -base64 32`).
- `LDAP_URL`: Your Active Directory / LDAP server address.
- `FIREWALL_TYPE`: Set to `paloalto`, `fortigate`, or `both`.
- `CORS_ORIGIN`: Set to your production domain (e.g., `https://access.yourdomain.com`).
- `SMTP_*`: Your mail server credentials for notifications.

## 4. SSL/TLS & Domain Configuration

You have two main options for setting up your domain and SSL:

### Option A: Nginx Proxy Manager (Recommended for GUI)
Nginx Proxy Manager (NPM) provides a web-based interface to manage reverse proxies and Let's Encrypt SSL certificates easily.

1.  **Create a dedicated directory for NPM:**
    ```bash
    mkdir -p ~/nginx-proxy-manager
    cd ~/nginx-proxy-manager
    ```

2.  **Create a `docker-compose.yml` for NPM:**
    ```yaml
    version: '3.8'
    services:
      app:
        image: 'jc21/nginx-proxy-manager:latest'
        restart: unless-stopped
        ports:
          - '80:80'
          - '81:81'
          - '443:443'
        volumes:
          - ./data:/data
          - ./letsencrypt:/etc/letsencrypt
    ```

3.  **Start NPM:**
    ```bash
    docker compose up -d
    ```

4.  **Access the Admin UI:**
    - Go to `http://<your-server-ip>:81`
    - Default credentials: `admin@example.com` / `changeme`
    - Immediately update your email and password.

5.  **Configure Proxy Host for AnydeskAccess:**
    - Click **Proxy Hosts** -> **Add Proxy Host**.
    - **Domain Names**: `access.yourdomain.com`
    - **Scheme**: `http`
    - **Forward Name/IP**: Use your server's local IP (e.g., `172.17.0.1` or the actual LAN IP).
    - **Forward Port**: `5053`
    - **Block Common Exploits**: Enable.
    - **SSL Tab**: Select **Request a new SSL Certificate**, enable **Force SSL** and **HTTP/2 Support**.

### Option B: Certbot (Manual CLI)
If you prefer a lightweight CLI-only approach:
```bash
sudo apt install -y certbot python3-certbot-nginx
```

### Obtain a Certificate
If you have a domain pointed to the server IP:
```bash
sudo certbot certonly --standalone -d access.yourdomain.com
```

### Update `docker-compose.yml` for Production
Update the `frontend` ports if you plan to use an external reverse proxy, or map 443 if doing it inside. 

**Recommended Path:** Run the Docker stack on internal ports and use an Ubuntu-level Nginx to handle SSL.

## 5. Deployment

Build and start the containers in detached mode:

```bash
docker compose build --pull
docker compose up -d
```

Check the status:
```bash
docker compose ps
```

## 6. Verification

1.  **Logs**: Monitor logs to ensure LDAP and Firewall connections are working.
    ```bash
    docker compose logs -f backend
    ```
2.  **Health**: Ensure both containers show as `healthy`.
3.  **Access**: Navigate to `http://<server-ip>:5053` (or your domain) and attempt a login.

## 7. Maintenance & Updates

### Updating to latest version
```bash
git pull
docker compose build backend
docker compose up -d
```

### Cleaning up old images
```bash
docker system prune -f
```

### Troubleshooting
- **Firewall Issues**: Ensure the Ubuntu host allows incoming traffic on the web port (default 5053 or 80/443).
  ```bash
  sudo ufw allow 5053/tcp
  ```
- **LDAP Connectivity**: Ensure the server can reach the LDAP port (usually 389 or 636) on your DC.
- **Palo Alto Commit**: Note that commits can take 15-30 seconds. They now run in the background, so check backend logs if rules don't appear immediately.

## 8. Security Hardening (Post-Deployment)

1.  **Restrict Port 5052**: The backend port `5052` is exposed for testing. In production, you should either remove it from `docker-compose.yml` or block it via UFW, allowing only Nginx to communicate with it.
2.  **Unprivileged User**: Ensure Docker is not running unnecessary services as root.
3.  **Logs Rotation**: Docker handles this by default, but monitor `/var/lib/docker/containers` size.
