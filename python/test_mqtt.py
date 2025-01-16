import json
import base64
import requests

CONFIG = json.load(open('.mqtt.config.json'))
BASE_URL = CONFIG['mqtt_url']
API_KEY = CONFIG['api_key']
SECRET = CONFIG['secret']

def main():
    url = f'{BASE_URL}/api/v5/publish'

    event = {
        "event": "online"
    }

    params = {
        'topic': 'app/11222/status',
        'payload': json.dumps(event),
    }

    headers = {
        'Content-Type': 'application/json'
    }

    plain = f'{API_KEY}:{SECRET}'
    encoded = base64.b64encode(plain.encode()).decode()

    headers['Authorization'] = f'Basic {encoded}'

    response = requests.post(url, headers=headers, json=params)
    print(response.json())


if __name__ == '__main__':
    main()