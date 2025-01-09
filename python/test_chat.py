import requests

def main():
    headers = {
        "x-oz-device-id": "1",
        "x-oz-dev-id": "1",
        "x-oz-user-id": "1",
        "Content-Type": "application/json",
        "Authorization": "Bearer 1234567890",
    }
    ret = requests.post("http://localhost:3000/api/chat", json={
        "message": "1",
        "session_id": "",
        "user_id": "2",
        "role_id": "1"
    }, headers=headers)
    print(ret.text)

if __name__ == "__main__":
    main()