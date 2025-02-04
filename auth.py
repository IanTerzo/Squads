from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.support.ui import WebDriverWait
import json
import urllib.parse

base_url = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"

challenge = "lXHr5Zb7Mro-sKjZXn5xYpYhMX3ik5MsA9APHPlDtpQ"

params = {
    "client_id": "5e3ce6c0-2b1f-4285-8d4b-75ee78787346",
    "scope": "openId profile openid offline_access",
    "redirect_uri": "https://teams.microsoft.com/v2",
    "response_mode": "fragment",
    "response_type": "code",
    "x-client-SKU": "msal.js.browser",
    "x-client-VER": "3.18.0",
    "client_info": "1",
    "code_challenge": challenge,
    "code_challenge_method": "plain",
    "prompt": "none"
}

encoded_params = urllib.parse.urlencode(params)
auth_url = f"{base_url}?{encoded_params}"

chrome_options = Options()
chrome_options.add_argument(f"--app={auth_url}")
chrome_options.add_argument("--user-data-dir=./user-data-dir")
chrome_options.add_argument("--window-size=550,500")
chrome_options.add_argument("--disable-infobars")  # Disable the infobar
chrome_options.add_experimental_option("excludeSwitches", ["enable-automation"])  # Disable "Chr

driver = webdriver.Chrome(options=chrome_options)

WebDriverWait(driver, 3600).until(lambda driver: "https://teams.microsoft.com/v2/" in driver.current_url)

if  "https://teams.microsoft.com/v2/#error=interaction_required" in driver.current_url:
	params.pop("prompt")

	encoded_params = urllib.parse.urlencode(params)
	auth_url = f"{base_url}?{encoded_params}"

	driver.get(auth_url)

	WebDriverWait(driver, 3600).until(lambda driver: "https://teams.microsoft.com/v2/" in driver.current_url)


code = driver.current_url.replace("https://teams.microsoft.com/v2/#code=", "").split("&")[0]
data = {"code": code, "code_verifier": challenge}

print(json.dumps(data))

driver.quit()
