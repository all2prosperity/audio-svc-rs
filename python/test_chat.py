import requests

def main():
    ret = requests.post("http://localhost:3000/chat", json={
        "message": "你好",
        "session_id": "1",
        "user_id": "1",
        "role_id": "1"
    })
    print(ret.text)

if __name__ == "__main__":
    main()