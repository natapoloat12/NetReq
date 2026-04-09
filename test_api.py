import requests
import json
import os
from dotenv import load_dotenv

load_dotenv()

PORT = os.getenv("PORT", "5051")
BASE_URL = f"http://localhost:{PORT}/api"

def test_api():
    print(f"--- Testing Anydesk Access API on port {PORT} ---")
    
    # 1. Login Test (Using dummy credentials if not set)
    login_payload = {
        "username": "testuser",
        "password": "testpassword"
    }
    
    try:
        print(f"POST {BASE_URL}/login...")
        login_res = requests.post(f"{BASE_URL}/login", json=login_payload)
        print(f"Status: {login_res.status_code}")
        print(f"Response: {login_res.text}")
        
        if login_res.status_code == 200:
            cookies = login_res.cookies
            print("\n2. Access Request Test...")
            access_payload = {
                "ip": "10.90.10.55",
                "cc_emails": ["manager@kce.co.th"]
            }
            access_res = requests.post(f"{BASE_URL}/access", json=access_payload, cookies=cookies)
            print(f"Status: {access_res.status_code}")
            print(f"Response: {access_res.text}")
            
            print("\n3. Verify Session Test...")
            verify_res = requests.get(f"{BASE_URL}/verify", cookies=cookies)
            print(f"Status: {verify_res.status_code}")
            print(f"Response: {verify_res.text}")
        else:
            print("\nLogin failed (this is expected if LDAP is not reachable).")
            
    except requests.exceptions.ConnectionError:
        print("\nERROR: Could not connect to the server. Please start the backend with 'cargo run' first.")

if __name__ == "__main__":
    test_api()
