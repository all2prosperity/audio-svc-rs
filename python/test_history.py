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

if __name__ == "__main__":
    main()

