from selenium import webdriver
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.common.by import By
import json

challenge = "lXHr5Zb7Mro-sKjZXn5xYpYhMX3ik5MsA9APHPlDtpQ"

auth_url = rf"https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id=5e3ce6c0-2b1f-4285-8d4b-75ee78787346&scope=openId%20profile%20openid%20offline_access&redirect_uri=https%3A%2F%2Fteams.microsoft.com%2Fv2&response_mode=fragment&response_type=code&x-client-SKU=msal.js.browser&x-client-VER=3.18.0&client_info=1&code_challenge={challenge}&code_challenge_method=plain"

chrome_options = Options()
chrome_options.add_argument(f"--app={auth_url}")

driver = webdriver.Chrome(options=chrome_options)

WebDriverWait(driver, 3600).until(lambda driver: "https://teams.microsoft.com/v2/" in driver.current_url)

code = driver.current_url.replace("https://teams.microsoft.com/v2/#code=", "").split("&")[0]
data = {"code": code, "code_verifier": challenge}

print(json.dumps(data))
