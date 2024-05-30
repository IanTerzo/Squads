/** @type {import('@sveltejs/kit').Handle} */
import got from 'got';
import puppeteer from 'puppeteer';
import fs from 'fs';

let tokens = {}

async function authorize(email, password) {
    console.log("Authorizing...")

    const browser = await puppeteer.launch({
        headless: false
    });
    const page = await browser.newPage();

    // You will get redirected to the login-in page
    await page.goto('https://teams.microsoft.com/');

    // Enter email and continue

    await page.waitForSelector('input[name="loginfmt"]', {
        timeout: 5_000
    });

    await page.click('#i0116');
    await page.type('#i0116', email, {
        delay: 0
    });

    var next = await page.$('#idSIButton9[value="Next"]')
    await next.click()

    // Check if the email is wrong

    try {
        await page.waitForSelector('#usernameError', {
            timeout: 5_000
        });
        await browser.close();
        return "Wrong email / username"
    } catch {

    }

    // Enter password and continue
    
    await page.waitForSelector('#i0118', {
        timeout: 5_000
    });

    await page.click('#i0118');
    await page.type('#i0118', password, {
        delay: 0
    });

    await page.waitForSelector('#idSIButton9[value="Sign in"]', {
        timeout: 5_000
    });

    var next = await page.$('#idSIButton9[value="Sign in"]')
    await next.click()

    // Check if the password is wrong

    try {
        await page.waitForSelector('#passwordError', {
            timeout: 5_000
        });
        await browser.close();
        return "Wrong password"
    } catch {

    }

    // Say yes to stay signed in (doesn't really matter)

    try {
        await page.waitForSelector('#idSIButton9[value="Yes"]', {
            timeout: 5_000
        });

        var next = await page.$('#idSIButton9[value="Yes"]')
        await next.click()
    } catch {

    }

    // Wait for Teams to load

    await page.waitForFunction(() =>
        document.querySelectorAll('.app-bar-text, .fui-Button__icon').length
    );

    // Get the refresh token

    const localStorageData = await page.evaluate(() => Object.assign({}, window.localStorage));

    const refreshToken = Object.values(localStorageData).find(item => {
        try {
            const parsedItem = JSON.parse(item);
            return parsedItem.credentialType === 'RefreshToken';
        } catch {
            return false;
        }
    });

    await browser.close();

    tokens['refreshToken'] = {
        "secret": JSON.parse(refreshToken).secret,
        "expires": Math.floor(Date.now() / 1000) + 86400
    }

    tokens["credentials"].email = email
    tokens["credentials"].password = password

    const data = JSON.stringify(tokens, null, 2)
    fs.writeFileSync('tokens/tokens.json', data);

    console.log("Finished authorizing")
    return "Success";
}

async function userProperties() {
    if (tokens["skypetoken"].expires < Math.floor(Date.now() / 1000)) {
        await genSkypetoken()
    }

    const headers = {
        'Authentication': 'skypetoken=' + tokens['skypetoken'].secret,
    };

    const response = await got(`https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/properties`, {
        headers: headers
    });

    if (response.statusCode == 200) {
        return JSON.parse(response['body'])
    } else {
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }
}


async function userTeams() {
    if (tokens["https://chatsvcagg.teams.microsoft.com/.default"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens("https://chatsvcagg.teams.microsoft.com/.default")
    }

    const headers = {
        'Authentication': 'skypetoken=' + tokens['skypetoken'].secret,
    };


    const response = await got('https://teams.microsoft.com/api/csa/emea/api/v2/teams/users/me', {
        searchParams: {
            'isPrefetch': 'false',
            'enableMembershipSummary': 'true',
            'enableRC2Fetch': 'false'
        },
        headers: {
            'authorization': 'Bearer ' + tokens['https://chatsvcagg.teams.microsoft.com/.default'].secret
        }
    });


    if (response.statusCode == 200) {
        return JSON.parse(response['body'])
    } else {
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }
}


async function teamConversation(teamId, topicId) {

    if (tokens["https://chatsvcagg.teams.microsoft.com/.default"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens("https://chatsvcagg.teams.microsoft.com/.default")
    }

    const headers = {
        'authorization': 'Bearer ' + tokens['https://chatsvcagg.teams.microsoft.com/.default'].secret,
    };

    const response = await got(`https://teams.microsoft.com/api/csa/emea/api/v2/teams/${teamId}/channels/${topicId}?filterSystemMessage=true&pageSize=20`, {
        headers: headers
    });

    if (response.statusCode == 200) {
        return JSON.parse(response['body'])
    } else {
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }

}

async function teamDetails(TeamID) {

    if (tokens["https://ic3.teams.office.com/Teams.AccessAsUser.All"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens("https://ic3.teams.office.com/Teams.AccessAsUser.All")
    }

    const headers = {
        'authorization': 'Bearer ' + tokens["https://ic3.teams.office.com/Teams.AccessAsUser.All"].secret,
    };

    const response = await got(`https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/conversations/${TeamID}?view=msnp24Equivalent`, {
        headers: headers
    });

    if (response.statusCode == 200) {
        return JSON.parse(response['body'])
    } else {
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }
}

async function authorizeImage(imageId) {
    if (tokens["skypetoken"].expires < Math.floor(Date.now() / 1000)) {
        await genSkypetoken()
    }

    const headers = {
        'authorization': 'skype_token ' + tokens['skypetoken'].secret,
    };

    const response = await got(`https://eu-prod.asyncgw.teams.microsoft.com/v1/objects/${imageId}/views/imgo?v=1`, {
        headers: headers
    });

    return response['rawBody']
}

async function authorizeProfilePicture(userId, displayName) {
    if (tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens("https://api.spaces.skype.com/Authorization.ReadWrite")
    }

    const headers = {
        'Referer': 'https://teams.microsoft.com/_',
        'Cookie': `authtoken=Bearer=${tokens['https://api.spaces.skype.com/Authorization.ReadWrite'].secret}&Origin=https://teams.microsoft.com;`,

    }

    const params = {
        'displayname': displayName,
        'size': 'HR64x64'
    }

    const response = await got(`https://teams.microsoft.com/api/mt/part/emea-02/beta/users/${userId}/profilepicturev2`, {
        headers: headers,
        searchParams: params
    });

    return response['rawBody']
}


async function authorizeTeamPicture(groupId, ETag, displayName) {
    if (tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens("https://api.spaces.skype.com/Authorization.ReadWrite")
    }


    const response = await got('https://teams.microsoft.com/api/mt/part/emea-02/beta/users/15de4241-e9be-4910-a60f-3f37dd8652b8/profilepicturev2/teams/' + groupId, {
        searchParams: {
            'etag': ETag,
            'displayName': displayName,
        },
        headers: {
            'Referer': 'https://teams.microsoft.com/v2/',
            'Cookie': `authtoken=Bearer=${tokens['https://api.spaces.skype.com/Authorization.ReadWrite'].secret}&Origin=https://teams.microsoft.com;`,

        }


    });


    return response['rawBody']
}
async function userAggregateSettings(json) {
    if (tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens("https://api.spaces.skype.com/Authorization.ReadWrite")
    }

    const response = await got.post('https://teams.microsoft.com/api/mt/part/emea-02/beta/users/useraggregatesettings', {
        headers: {

            'Authorization': 'Bearer ' + tokens['https://api.spaces.skype.com/Authorization.ReadWrite'].secret,

        },
        json: json
    });

    return JSON.parse(response.body)
}



async function renderListDataAsStream(section, filesRelativePath) {
    if (tokens["SPOIDCRL"].expires < Math.floor(Date.now() / 1000)) {
        await genSPOIDCRL(section) // Uknown section, domain / org section?
    }

    const response = await got.post(`${tokens['webUrl']}/sites/${section}/_api/web/GetListUsingPath(DecodedUrl=@a1)/RenderListDataAsStream?@a1='${filesRelativePath}'&RootFolder=${filesRelativePath}&TryNewExperienceSingle=TRUE`, {
        headers: {
            'Cookie': 'SPOIDCRL=' + tokens["SPOIDCRL"].secret,
        },
        json: {
            'parameters': {
                'RenderOptions': 5723911,
                'AllowMultipleValueFilterForTaxonomyFields': true,
                'AddRequiredFields': true,
                'ModernListBoot': true,
                'RequireFolderColoringFields': true
            }
        }
    });

    return JSON.parse(response.body)
}
async function genWebUrl() {
    let userAggregateSetting = await userAggregateSettings({
        'tenantSiteUrl': true,
    })

    tokens["webUrl"] = userAggregateSetting.tenantSiteUrl.value.webUrl.replace("/_layouts/15/sharepoint.aspx", "")

    tokens[tokens["webUrl"] + "/.default"] = {
        secret: "",
        expires: 0
    }

    const data = JSON.stringify(tokens, null, 2)
    fs.writeFileSync('tokens/tokens.json', data);
}

async function genSPOIDCRL(section) {
    if (tokens.webUrl == "") {
        await genWebUrl()
    }

    if (tokens[tokens["webUrl"] + "/.default"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens(tokens["webUrl"] + "/.default")
    }

    const response = await got.post(`${tokens['webUrl']}/sites/${section}/_api/SP.OAuth.NativeClient/Authenticate`, {
        headers: {
            'Authorization': 'Bearer ' + tokens[tokens["webUrl"] + "/.default"].secret,
        }

    });


    const match = response.headers['set-cookie'][0].match(/SPOIDCRL=(.+?);/);
    const SPOIDCRL = match[0].replace("SPOIDCRL=", "").replace(";", "")

    tokens["SPOIDCRL"] = {
        "secret": SPOIDCRL,
        "expires": Math.floor(Date.now() / 1000) + 2628288
    } // One month

    const data = JSON.stringify(tokens, null, 2)
    fs.writeFileSync('tokens/tokens.json', data);


}

async function genSkypetoken() {
    if (tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].expires < Math.floor(Date.now() / 1000)) {
        await GenTokens("https://api.spaces.skype.com/Authorization.ReadWrite")
    }

    const headers = {
        'authorization': 'Bearer ' + tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].secret
    }

    const response = await got.post('https://teams.microsoft.com/api/authsvc/v1.0/authz', {
        headers: headers
    });

    const skypetoken = JSON.parse(response['body'])['tokens']
    tokens["skypetoken"] = {
        "secret": skypetoken['skypeToken'],
        "expires": Math.floor(Date.now() / 1000) + skypetoken["expiresIn"]
    }

    const data = JSON.stringify(tokens, null, 2)
    fs.writeFileSync('tokens/tokens.json', data);
}

async function GenTokens(scope) {
    if (tokens["refreshToken"].expires < Math.floor(Date.now() / 1000)) {
        await authorize(tokens["credentials"].email, tokens["credentials"].password)
    }

    const headers = {

        'Origin': 'https://teams.microsoft.com',

    };

    var dataString = `client_id=5e3ce6c0-2b1f-4285-8d4b-75ee78787346&scope=${scope} openid profile offline_access&grant_type=refresh_token&client_info=1&x-client-SKU=msal.js.browser&x-client-VER=3.7.1&refresh_token=${tokens['refreshToken'].secret}&claims={"access_token":{"xms_cc":{"values":["CP1"]}}}`;

    const response = await got.post(`https://login.microsoftonline.com/660a30b5-8e2e-4769-b9eb-4af28bfd12bd/oauth2/v2.0/token`, {
        body: dataString,
        headers: headers
    });

    if (response.statusCode == 200) {
        var responseBody = JSON.parse(response['body'])
        tokens[scope] = {
            "secret": responseBody['access_token'],
            "expires": Math.floor(Date.now() / 1000) + responseBody["expires_in"]
        }

        const data = JSON.stringify(tokens, null, 2)
        fs.writeFileSync('tokens/tokens.json', data);
    } else {
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }

}

async function Setup() {
    if (Object.values(tokens).length === 0) {
        const data = await fs.promises.readFile('tokens/tokens.json', 'utf8');
        tokens = JSON.parse(data);
    }

    //let teams = await userTeams()

    //let section = teams['teams'][4].smtpAddress.split('@')[0]
    //console.log(section)

    //let filesRelativePath = teams['teams'][4].channels[0].defaultFileSettings.filesRelativePath
    //console.log(filesRelativePath)

    // let datastream = await renderListDataAsStream(section, filesRelativePath)

    //console.log(datastream)


}


function isAuthorized() {
    if (tokens.credentials.email == "" || tokens.credentials.password == "") {
        if (tokens["refreshToken"].expires < Math.floor(Date.now() / 1000)) {
            return false

        }
    }
    return true

}



export async function handle({
    event,
    resolve
}) {
    const url = event.url.pathname.split("/")

    await Setup()

    if (url[1] != "login" && url[2] != "authorize" && !isAuthorized()) {

        return new Response(null, {
            status: 302,
            headers: {
                'Location': '/login'
            }
        })
    }

    if (url[1] === 'api') {
        try {
            if (url[2] === 'authorize') {
                const {
                    email,
                    password
                } = await event.request.json();
                const authorization = await authorize(email, password);

                if (authorization === "Success") {
                    return new Response(null, {
                        status: 200
                    });
                } else {
                    return new Response(JSON.stringify({
                        message: authorization
                    }), {
                        status: 401,
                        headers: {
                            'Content-Type': 'application/json'
                        }
                    });
                }
            }

            else if (url[2] === 'userProperties') {
                const properties = await userProperties();
                return new Response(JSON.stringify(properties), {
                    status: 200,
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });
            } else if (url[2] === 'userTeams') {
                const teams = await userTeams();
                return new Response(JSON.stringify(teams), {
                    status: 200,
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });
            }

            else if (url[2] === 'image' && url[3]) {
                const binaryData = await authorizeImage(url[3]);

                // This is needed
                const uint8Array = new Uint8Array(binaryData);
                return new Response(uint8Array.buffer, {
                    status: 200,
                    headers: {
                        'Content-Type': 'image/jpeg'
                    }
                });
            } else if (url[2] === 'profilePicture' && url[3] && url[4]) {
                const binaryData = await authorizeProfilePicture(url[3], url[4]);

                const uint8Array = new Uint8Array(binaryData);
                return new Response(uint8Array.buffer, {
                    status: 200,
                    headers: {
                        'Content-Type': 'image/jpeg'
                    }
                });
            }
            else if (url[2] === 'teamPicture' && url[3] && url[4] && url[5]) {
                const binaryData = await authorizeTeamPicture(url[3], url[4], url[4]);

                const uint8Array = new Uint8Array(binaryData);
                return new Response(uint8Array.buffer, {
                    status: 200,
                    headers: {
                        'Content-Type': 'image/jpeg'
                    }
                });
            }


            else if (url[2] === 'teamConversation' && url[3] && url[4]) {
                const conversation = await teamConversation(url[3], url[4]);
                return new Response(JSON.stringify(conversation), {
                    status: 200,
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });
            } else if (url[2] === 'teamDetails' && url[3]) {
                const details = await teamDetails(url[3]);
                return new Response(JSON.stringify(details), {
                    status: 200,
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });
            } else if (url[2] === 'renderListDataAsStream' && url[3]) {

                const section = url[3];

                const parametersArray = event.request.url.split('?')[1].split('&');
                const parameters = {};

                parametersArray.forEach(parameter => {
                    const [key, value] = parameter.split('=');
                    parameters[key] = value;
                });

                const conversation = await renderListDataAsStream(section, parameters.filesRelativePath);
                return new Response(JSON.stringify(conversation), {
                    status: 200,
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });
            }

        } catch (error) {
            console.log(error)
            return new Response(JSON.stringify({
                error: error.message
            }), {
                status: 500,
                headers: {
                    'Content-Type': 'application/json'
                }
            });
        }
    }


    const response = await resolve(event);
    return response;
}
