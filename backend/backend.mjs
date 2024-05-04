import express from 'express';
import bodyParser from 'body-parser';
import cors from 'cors';
import got from 'got';
import puppeteer from 'puppeteer';
import fs, {cpSync, rename} from 'fs';

const app = express();
const port = 5102;

app.use(bodyParser.json());
app.use(cors());

app.get('/user-properties', async (req, res) => {
    if (!tokens['credentials'].email == ""){
        try {
            const properties = await UserProperties();
            res.json(properties);
        } catch (error) {
            res.status(500).json({
                error: error.message
            });
        }     
    }
    else
    {
        res.sendStatus(401)
    }
});

app.get('/image/:imageId', async (req, res) => {
    if (!tokens['credentials'].email == ""){
        const {
            imageId
        } = req.params;
        try {
            const binaryData = await authorizeImage(imageId);
            res.setHeader('Content-Type', 'image/jpeg');
    
            res.send(binaryData);
        } catch (error) {
            res.status(500).json({
                error: error.message
            });
        }
    }
    else
    {
        res.sendStatus(401)
    }
});

app.post('/authorize/', async (req, res) => {
    try {
        const authorization = await authorize(req.body.email, req.body.password);
        if(authorization == "Success"){
            res.sendStatus(200)
        }
        else
        {
            res.status(401).send(authorization)
        }
        res.send()
    } catch (error) {
        res.status(500).json({
            error: error.message
        });
    }
});

app.get('/profilePicture/:userId/:displayName', async (req, res) => {
    if (!tokens['credentials'].email == ""){
        const userId = req.params.userId;
        const displayName = req.params.displayName;
        try {
            const binaryData = await authorizeProfilePicture(userId, displayName);
            res.setHeader('Content-Type', 'image/jpeg');

            res.send(binaryData);
        } catch (error) {
            res.status(500).json({
            error: error.message
        });
        }
    }
    else
    {
        res.sendStatus(401)
    }
});

app.get('/team-conversation/:teamId/:topicId', async (req, res) => {
    if (!tokens['credentials'].email == ""){
        const teamId = req.params.teamId;
        const topicId = req.params.topicId;
        try {
            const conversation = await TeamConversation(teamId, topicId);
            res.json(conversation);
        } catch (error) {
            res.status(500).json({
                error: error.message
            });
        }
    }
    else
    {
        res.sendStatus(401)
    }
});

app.get('/team-details/:teamId', async (req, res) => {
    if (!tokens['credentials'].email == ""){
        const {
            teamId
        } = req.params;
        try {
            const details = await TeamDetails(teamId);
            res.json(details);
        } catch (error) {
            res.status(500).json({
                error: error.message
            });
        }
    }
    else
    {
        res.sendStatus(401)
    }

});

var tokens = {}

async function authorize(email, password){
    console.log("Authorizing...")

    const browser = await puppeteer.launch({headless:true});
    const page = await browser.newPage();
   
    // You will get redirected to the login-in page
    await page.goto('https://teams.microsoft.com/'); 
    
    // Enter email and continue

    await page.waitForSelector('input[name="loginfmt"]', { timeout: 5_000 });

    await page.click('#i0116');
    await page.type('#i0116', email, {delay: 0});
   
    var next = await page.$('#idSIButton9[value="Next"]')
    await next.click()
    
    // Check if the email is wrong
    
    try {
        await page.waitForSelector('#usernameError', { timeout: 5_000 });
        await browser.close();
        return "Wrong email / username"
    }
    catch {

    }
   
    // Enter password and continue

    await page.waitForSelector('#i0118', { timeout: 5_000 });
    
    await page.click('#i0118');
    await page.type('#i0118', password, {delay: 0});

    await page.waitForSelector('#idSIButton9[value="Sign in"]', { timeout: 5_000 });
    
    var next = await page.$('#idSIButton9[value="Sign in"]')
    await next.click()

    // Check if the password is wrong
    
    try {
        await page.waitForSelector('#passwordError', { timeout: 5_000 });
        await browser.close();
        return "Wrong password"
    }
    catch {

    }

    // Say yes to stay signed in (doesn't really matter)

    await page.waitForSelector('#idSIButton9[value="Yes"]', { timeout: 30_000 });
     
    var next = await page.$('#idSIButton9[value="Yes"]')
    await next.click()
   
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
        }
        catch {
            return false;
        }
    });
   
    await browser.close();

    tokens['refresh_token'] = {"secret": JSON.parse(refreshToken).secret, "expires":  Math.floor(Date.now() / 1000) + 86400}
    
    tokens["credentials"].email = email
    tokens["credentials"].password = password

    const data = JSON.stringify(tokens, null, 2)
    fs.writeFileSync('tokens.json', data);
    
    console.log("Finished authorizing")
    return "Success";
}

async function UserProperties() {
    if (tokens["skypetoken"].expires <  Math.floor(Date.now() / 1000)){
        await genSkypetoken()
    }
    
    const headers = { 
        'Authentication': 'skypetoken=' + tokens['skypetoken'].secret,
    };
      
    const response = await got(`https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/properties`, {
        headers: headers
    });

    if (response.statusCode == 200)
    {
        return JSON.parse(response['body'])
    }
    else{
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }
}

async function TeamConversation(teamId, topicId) {

    if (tokens["https://chatsvcagg.teams.microsoft.com/.default"].expires <  Math.floor(Date.now() / 1000)){
        await GenTokens("https://chatsvcagg.teams.microsoft.com/.default")
    }
    
    const headers = {
        'authorization': 'Bearer ' + tokens['https://chatsvcagg.teams.microsoft.com/.default'].secret,
    };
      
    const response = await got(`https://teams.microsoft.com/api/csa/emea/api/v2/teams/${teamId}/channels/${topicId}?filterSystemMessage=true&pageSize=20`, {
        headers: headers
    });

    if (response.statusCode == 200)
    {
        return JSON.parse(response['body'])
    }
    else{
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }

}

async function TeamDetails(TeamID) {
    
    if (tokens["https://ic3.teams.office.com/Teams.AccessAsUser.All"].expires <  Math.floor(Date.now() / 1000)){
        await GenTokens("https://ic3.teams.office.com/Teams.AccessAsUser.All")
    }  

    const headers = {
        'authorization': 'Bearer ' + tokens["https://ic3.teams.office.com/Teams.AccessAsUser.All"].secret,
    };

    const response = await got(`https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/conversations/${TeamID}?view=msnp24Equivalent`, {
        headers: headers
    });

    if (response.statusCode == 200)
    {
        return JSON.parse(response['body'])
    }
    else{
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }
}

async function authorizeImage(imageId) {
    if (tokens["skypetoken"].expires <  Math.floor(Date.now() / 1000)){
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
    if (tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].expires <  Math.floor(Date.now() / 1000)){
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

async function genSkypetoken(){
    if (tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].expires <  Math.floor(Date.now() / 1000)){
        await GenTokens("https://api.spaces.skype.com/Authorization.ReadWrite")
    } 
    
    const headers = {
        'authorization': 'Bearer ' + tokens["https://api.spaces.skype.com/Authorization.ReadWrite"].secret
    }
    
    const response = await got.post('https://teams.microsoft.com/api/authsvc/v1.0/authz', {
        headers: headers
    });

    const skypetoken = JSON.parse(response['body'])['tokens']
    tokens["skypetoken"] =  {"secret": skypetoken['skypeToken'], "expires":  Math.floor(Date.now() / 1000) + skypetoken["expiresIn"]}
    
    const data = JSON.stringify(tokens, null, 2)
    fs.writeFileSync('tokens.json', data);
}

async function GenTokens(scope) {
    if (tokens["refresh_token"].expires <  Math.floor(Date.now() / 1000)){
        await authorize(tokens["credentials"].email, tokens["credentials"].password)
    }
    
    const headers = {

        'Origin': 'https://teams.microsoft.com',

    };

    var dataString = `client_id=5e3ce6c0-2b1f-4285-8d4b-75ee78787346&scope=${scope} openid profile offline_access&grant_type=refresh_token&client_info=1&x-client-SKU=msal.js.browser&x-client-VER=3.7.1&refresh_token=${tokens['refresh_token'].secret}&claims={"access_token":{"xms_cc":{"values":["CP1"]}}}`;
    
    const response = await got.post(`https://login.microsoftonline.com/660a30b5-8e2e-4769-b9eb-4af28bfd12bd/oauth2/v2.0/token`, {
        body: dataString,
        headers: headers
    });

    if (response.statusCode == 200)
    {
        var responseBody = JSON.parse(response['body'])
        tokens[scope] = {"secret": responseBody['access_token'], "expires":  Math.floor(Date.now() / 1000) + responseBody["expires_in"]}
        
        const data = JSON.stringify(tokens, null, 2)
        fs.writeFileSync('tokens.json', data);  
    }
    else{
        reject(new Error(`Request failed with status code ${response.statusCode}`));
        return;
    }

}

async function Setup() {

    const data = await fs.promises.readFile('tokens.json', 'utf8');
    tokens = JSON.parse(data);
}

Setup()

app.listen(port, () => {
    console.log(`Backend is running on port: ${port}`);
});
