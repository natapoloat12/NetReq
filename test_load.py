import requests
import threading
import time
import sys

BASE_URL = "http://localhost:5053"
LOGIN_PAYLOAD = {
    "username": "testuser",
    "password": "testpassword"
}

def run_test():
    session = requests.Session()
    
    # 1. Login
    try:
        res = session.post(f"{BASE_URL}/api/login", json=LOGIN_PAYLOAD)
        if res.status_code != 200:
            print(f"Login failed: {res.status_code}")
            return
    except Exception as e:
        print(f"Connection error: {e}")
        return

    # 2. Access Request
    access_payload = {
        "ip": "10.0.0.1",
        "service": "anydesk",
        "cc_emails": ["test@kce.co.th"]
    }
    
    start_time = time.time()
    try:
        res = session.post(f"{BASE_URL}/api/access", json=access_payload)
        duration = time.time() - start_time
        print(f"Access Request: Status {res.status_code}, Time: {duration:.2f}s")
    except Exception as e:
        print(f"Request error: {e}")

def main():
    threads = []
    num_requests = 10
    
    print(f"Starting load test with {num_requests} concurrent requests...")
    for i in range(num_requests):
        t = threading.Thread(target=run_test)
        threads.append(t)
        t.start()

    for t in threads:
        t.join()

if __name__ == "__main__":
    main()
