import requests

def main():
    url = "http://localhost:3000/hello"
    print('will request', url)
    response = requests.get(url)
    print(response.text)

if __name__ == "__main__":
    main()
