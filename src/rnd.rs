fn gen_tokens(refresh_token: AccessToken) -> Result<AccessToken, ApiError> {
    // Generate new refresh token if needed
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("origin"),
        HeaderValue::from_static("https://teams.microsoft.com"),
    );

    let body = format!(
        "client_id=5e3ce6c0-2b1f-4285-8d4b-75ee78787346&\
        scope=openid profile offline_access&\
        grant_type=refresh_token&\
        client_info=1&\
        x-client-SKU=msal.js.browser&\
        x-client-VER=3.7.1&\
        refresh_token={}&\
        claims={{\"access_token\":{{\"xms_cc\":{{\"values\":[\"CP1\"]}}}}}}",
        refresh_token.value
    );
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client
        .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .headers(headers)
        .body(body)
        .send()
        .unwrap();

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().unwrap();
        return Err(ApiError::RequestFailed(status, body));
    }

    let token_data: HashMap<String, Value> = res.json().unwrap();
    if let (Some(value), Some(expires_in)) = (
        token_data.get("access_token").and_then(|v| v.as_str()),
        token_data.get("expires_in").and_then(|v| v.as_u64()),
    ) {
        let access_token = AccessToken {
            value: value.to_string(),
            expires: get_epoch_s() + expires_in,
        };
        Ok(access_token)
    } else {
        Err(ApiError::MissingTokenOrExpiry)
    }
}

fn renew() {
    let expired = "1.AXQAtTAKZi6OaUe560ryi_0SvcDmPF4fK4VCjUt17nh4c0biAPJ0AA.AgABAwEAAABVrSpeuWamRam2jAF1XRQEAwDs_wUA9P8pDisipQVEPTjB_K1nA2-knrDNSVbyIY7bNiskdfCbLuthbTgsHYIWIR823N8oU_s_xpDFvaTnQmSjV6n7tDLEn3HCVZkrADDv16y5XEFAfGhhYGmLKsJwFOI6CpW5KuiOJK_us_JYUm810HkvB7TXN8-FFcFrHgtamQ0bC2UEIg-k4nDQV6C8NBGfxMe_vfvhQ5zltI7Z2gx9T5grOeMlhUSv_N96u9oIIe_9k-uZaBM1LKK1UnefRJMUPsrGP81aJL1RExV9T5-gyKkJDxz8SUhXmpwvvc2GjQqydBOeVsrxw_N1pcgAyeqqWUsW1HO5iIWBqs9na5dd2v3Y7pTdP36zOPWBJg5kZ-cArU31c9b8J9cuuvn83i4Qq41vwKe_7Br29rOpKDHs-Qdt7C4T7JFLOGtmbFanmxAbE5Onyq8yT49l__j9-74dP3qU15ZukpQDIvOBMxUdOOaRCLUprWrC2RzwBIGgCS4HBB_LQAwMf-KSqZHOTyS8aOjeHscX1SR3EAZ3oTTm_FQQiJIN7EY4Ku2dorWu7g3IJPrjSjxiBrKp_UsnYXfD7Z69O9y87f4bD2cHhVbiKmmq9CflTKr64XDFmRXJTUNeVzbIuK8Tvu3SfmcMM884iFRY2rowfqOd1y6da9bTUnWDPjroI9LyITtxG8lVnnxqMkPGI_L2V-7qfIi7Hs8OfBK_gk3jD_D3ddDdjyglSro1s3BeUEvYFaT-qsRHcwKk8dGiAWOqIffhIzddbGVxQ-0ZTTlAgVWpWw".to_string();
    gen_tokens()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token() {
        renew();
    }
}
