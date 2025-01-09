import requests

def main():
    headers = { 
        "x-oz-device-id": "1",
        "x-oz-dev-id": "1",
        "x-oz-user-id": "2",
        "Content-Type": "application/json",
        "Authorization": "Bearer 1234567890",
    }

    ret = requests.get("http://localhost:3000/api/chat/history?offset=0&limit=10", headers=headers)
    print(ret.text)

def test_session_history():
    headers = { 
        "x-oz-device-id": "1",
        "x-oz-dev-id": "1",
        "x-oz-user-id": "2",
        "Content-Type": "application/json",
        "Authorization": "Bearer 1234567890",
    }

    data = {
        "chat_id": "45757394-2edc-49a8-9e47-f6881a5d41fe",
        "offset": 0,
        "limit": 10,
    }

    ret = requests.post("http://localhost:3000/api/chat/session_history", headers=headers, json=data)
    print(ret.text)

if __name__ == "__main__":
    test_session_history()


