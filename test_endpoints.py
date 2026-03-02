#!/usr/bin/env python3
"""
Quick API endpoint tester for PixelWar
Tests auth, canvas, chat, voting, and group endpoints
"""
import requests
import json
import uuid
from typing import Optional

BASE_URL = "http://localhost:3000/api/v1"
TOKEN = None
USER_ID = None
ROUND_ID = str(uuid.uuid4())
PARCEL_ID = None
GROUP_ID = None

def print_section(title: str):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}")

def print_result(method: str, endpoint: str, status: int, success: bool, data: dict = None):
    symbol = "✓" if success else "✗"
    print(f"{symbol} {method} {endpoint} -> {status}")
    if data and success:
        print(f"  Response: {json.dumps(data, indent=2)[:200]}...")

def test_auth():
    """Test user registration and login"""
    global TOKEN, USER_ID

    print_section("TESTING AUTH ENDPOINTS")

    # Register
    email = f"user_{uuid.uuid4().hex[:8]}@test.com"
    username = f"user_{uuid.uuid4().hex[:8]}"
    password = "TestPassword123!"

    resp = requests.post(
        f"{BASE_URL}/auth/register",
        json={"username": username, "email": email, "password": password}
    )
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", "/auth/register", resp.status_code, success, data)

    if success:
        TOKEN = data.get("token")
        USER_ID = data.get("user_id")
        print(f"  Token: {TOKEN[:20]}...")
        print(f"  User ID: {USER_ID}")
        return True

    # Try login instead
    resp = requests.post(
        f"{BASE_URL}/auth/login",
        json={"email": email, "password": password}
    )
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", "/auth/login", resp.status_code, success, data)

    if success:
        TOKEN = data.get("token")
        USER_ID = data.get("user_id")

    return success

def get_headers():
    """Get headers with JWT token"""
    return {
        "Authorization": f"Bearer {TOKEN}",
        "Content-Type": "application/json"
    }

def test_canvas():
    """Test canvas (parcel) endpoints"""
    global PARCEL_ID

    print_section("TESTING CANVAS ENDPOINTS")

    if not TOKEN:
        print("⚠ Skipping - no auth token")
        return

    headers = get_headers()

    # Claim parcel
    parcel_data = {
        "round_id": ROUND_ID,
        "origin_x": 0,
        "origin_y": 0,
        "width": 100,
        "height": 100,
        "description": "Test parcel for pixel art"
    }
    resp = requests.post(f"{BASE_URL}/canvas/parcels", json=parcel_data, headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", "/canvas/parcels", resp.status_code, success, data)

    if success:
        PARCEL_ID = data.get("parcel_id")
        print(f"  Parcel ID: {PARCEL_ID}")

    if not PARCEL_ID:
        print("⚠ Skipping pixel update - no parcel")
        return

    # Update pixels
    pixel_data = {
        "parcel_id": PARCEL_ID,
        "pixels": [
            {"local_x": 0, "local_y": 0, "color": 0xFF0000FF},
            {"local_x": 1, "local_y": 1, "color": 0x00FF00FF},
            {"local_x": 2, "local_y": 2, "color": 0x0000FFFF},
        ]
    }
    resp = requests.post(f"{BASE_URL}/canvas/pixels", json=pixel_data, headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", "/canvas/pixels", resp.status_code, success, data)

    # Get snapshot
    resp = requests.get(f"{BASE_URL}/canvas/snapshot?x=0&y=0&width=10&height=10", headers=headers)
    print_result("GET", "/canvas/snapshot", resp.status_code, resp.status_code == 200)

def test_chat():
    """Test chat endpoints"""
    print_section("TESTING CHAT ENDPOINTS")

    if not TOKEN:
        print("⚠ Skipping - no auth token")
        return

    headers = get_headers()

    # Send global message
    msg_data = {
        "channel_type": "global",
        "content": "Hello pixel warriors! 🎨"
    }
    resp = requests.post(f"{BASE_URL}/chat/messages", json=msg_data, headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", "/chat/messages (global)", resp.status_code, success, data)

    # Get messages
    resp = requests.get(f"{BASE_URL}/chat/messages/global?limit=10", headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("GET", "/chat/messages/global", resp.status_code, success, data)

def test_voting():
    """Test voting endpoints"""
    print_section("TESTING VOTING ENDPOINTS")

    if not TOKEN or not PARCEL_ID:
        print("⚠ Skipping - need auth token and parcel")
        return

    headers = get_headers()

    # Create another user's parcel to vote for
    target_id = str(uuid.uuid4())

    # Cast vote
    vote_data = {
        "round_id": ROUND_ID,
        "target_id": target_id
    }
    resp = requests.post(f"{BASE_URL}/voting/vote", json=vote_data, headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", "/voting/vote", resp.status_code, success, data)

    # Get results
    resp = requests.get(f"{BASE_URL}/voting/results?round_id={ROUND_ID}", headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("GET", "/voting/results", resp.status_code, success, data)

def test_groups():
    """Test group endpoints"""
    global GROUP_ID

    print_section("TESTING GROUP ENDPOINTS")

    if not TOKEN:
        print("⚠ Skipping - no auth token")
        return

    headers = get_headers()

    # Create group
    group_data = {
        "name": f"Pixel Squad {uuid.uuid4().hex[:6]}",
        "round_id": ROUND_ID
    }
    resp = requests.post(f"{BASE_URL}/groups", json=group_data, headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", "/groups", resp.status_code, success, data)

    if success:
        GROUP_ID = data.get("group_id")
        print(f"  Group ID: {GROUP_ID}")

    if not GROUP_ID:
        return

    # Get group
    resp = requests.get(f"{BASE_URL}/groups/{GROUP_ID}", headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("GET", f"/groups/{GROUP_ID}", resp.status_code, success, data)

    # Invite someone (use a fake user ID for demo)
    invite_data = {
        "to_user_id": str(uuid.uuid4())
    }
    resp = requests.post(f"{BASE_URL}/groups/{GROUP_ID}/invite", json=invite_data, headers=headers)
    success = resp.status_code == 200
    data = resp.json() if success else None
    print_result("POST", f"/groups/{GROUP_ID}/invite", resp.status_code, success, data)

def main():
    print("\n" + "="*60)
    print("  PixelWar API Endpoint Test Suite")
    print("="*60)
    print(f"Base URL: {BASE_URL}")
    print(f"Round ID: {ROUND_ID}")

    try:
        test_auth()
        test_canvas()
        test_chat()
        test_voting()
        test_groups()

        print_section("TEST SUMMARY")
        print("✓ All endpoint tests completed!")
        print("\nNext steps:")
        print("1. Ensure all services are running (auth, canvas, chat, voting, group)")
        print("2. Verify database migrations have been applied")
        print("3. Check service logs for any errors")
        print("4. Modify GRPC endpoints in .env if services are on different hosts")

    except requests.exceptions.ConnectionError as e:
        print(f"\n✗ Connection Error: {e}")
        print("Make sure the API Gateway is running on http://localhost:3000")
    except Exception as e:
        print(f"\n✗ Error: {e}")

if __name__ == "__main__":
    main()
