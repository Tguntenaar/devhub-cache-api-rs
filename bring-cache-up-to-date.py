import time
import requests

local = False 
reset_from_zero = True # False to continue from where it left off  
fly_app_name = "devhub-cache-api-rs"
max_calls = 120 # This is for devhub to catch up to the latest block

base_url = f"http://localhost:8080/" if local else f"https://{fly_app_name}.fly.dev/"

def call_api(count):
    url = f"{base_url}proposal/256/snapshots"  # Replace with your API URL
    try:
        response = requests.get(url)
        if response.status_code == 200:
            print(f"{count} API call successful: - response length {len(response.json())}")
        else:
            print("API call failed with status code:", response.status_code)
    except requests.exceptions.RequestException as e:
        print("An error occurred:", e)
    except Exception as e:
        print("An error2 occurred:", e)
        print(response.json())

def reset_cache():
    url = f"{base_url}proposals/info/0"  # Replace with your API URL
    try:
        response = requests.get(url)
        if response.status_code == 200:
            print("Cache reset successful")
        else:
            print("Cache reset failed with status code:", response.status_code)
    except requests.exceptions.RequestException as e:
        print("An error occurred:", e)

def main():
    if reset_from_zero:
        reset_cache()
    count = 0
    while count < max_calls: 
        call_api(count)
        count += 1
        # 6 calls/minute limit of near blocks https://nearblocks.io/apis#
        time.sleep(0.5)  # Wait for 11 seconds before the next call

if __name__ == "__main__":
    main()