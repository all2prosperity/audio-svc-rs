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
        "message": "7",
        "session_id": "45757394-2edc-49a8-9e47-f6881a5d41fe",
        "user_id": "1",
        "role_id": "1"
    }, headers=headers)
    print(ret.text)

if __name__ == "__main__":
    main()