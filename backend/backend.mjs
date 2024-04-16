import express from 'express';
import bodyParser from 'body-parser';
import request from 'request';
import cors from 'cors';
import got from 'got';
import fs from 'fs';

const app = express();
const port = 5102;

app.use(bodyParser.json());
app.use(cors());

app.get('/user-properties', async (req, res) => {
    try {
        const properties = await UserProperties();
        res.json(properties);
    } catch (error) {
        res.status(500).json({
            error: error.message
        });
    }
});

app.get('/team-conversation/:teamId/:topicId', async (req, res) => {
    const teamId = req.params.teamId;
    const topicId = req.params.topicId;
    try {
        const conversation = await TeamConversation(teamId, topicI);
        res.json(conversation);
    } catch (error) {
        res.status(500).json({
            error: error.message
        });
    }
});

app.get('/team-details/:teamId', async (req, res) => {
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
});


// Search "RefreshToken" in localstorage
var client_id = ""

// Search "RefreshToken" in localstorage
var refresh_token = ""

var tokens = {}

async function UserProperties() {
    var headers = {
        'Authentication': 'skypetoken=' + tokens['skypetoken'],
    };

    var options = {
        url: 'https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/properties',
        headers: headers,
        gzip: true
    };

    return new Promise((resolve, reject) => {
        request(options, (error, response, body) => {
            if (error) {
                reject(error);
                return;
            }
            if (response.statusCode !== 200) {
                reject(new Error(`Request failed with status code ${response.statusCode}`));
                return;
            }
            const responseBody = JSON.parse(body);
            resolve(responseBody);
        });
    });
}

async function TeamConversation(teamId, topicId) {


    var headers = {
        'authorization': volutile_maintoken,
    };

    var options = {
        url: `https://teams.microsoft.com/api/csa/emea/api/v2/teams/${teamId}/channels/${topicId}?filterSystemMessage=true&pageSize=20`,
        headers: headers,
        gzip: true
    };

    return new Promise((resolve, reject) => {
        request(options, (error, response, body) => {
            if (error) {
                reject(error);
                return;
            }
            if (response.statusCode !== 200) {
                reject(new Error(`Request failed with status code ${response.statusCode}`));
                return;
            }
            const responseBody = JSON.parse(body);
            resolve(responseBody);
        });
    });
}

async function TeamDetails(TeamID) {

    var headers = {
        'authorization': 'Bearer ' + tokens["https://ic3.teams.office.com/Teams.AccessAsUser.Al"],
    };

    var options = {
        url: `https://teams.microsoft.com/api/chatsvc/emea/v1/users/ME/conversations/${TeamID}?view=msnp24Equivalent`,
        headers: headers,
        gzip: true
    };

    return new Promise((resolve, reject) => {
        request(options, (error, response, body) => {
            if (error) {
                reject(error);
                return;
            }
            if (response.statusCode !== 200) {
                reject(new Error(`Request failed with status code ${response.statusCode}`));
                return;
            }
            const responseBody = JSON.parse(body);
            resolve(responseBody);
        });
    });
}

async function GenTokens(scope) {
    var headers = {
        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0',
        'Accept': '*/*',
        'Accept-Language': 'en-US,en;q=0.5',
        'Accept-Encoding': 'gzip, deflate, br',
        'Referer': 'https://teams.microsoft.com/',
        'content-type': 'application/x-www-form-urlencoded;charset=utf-8',
        'Origin': 'https://teams.microsoft.com',
        'Connection': 'keep-alive',
        'Sec-Fetch-Dest': 'empty',
        'Sec-Fetch-Mode': 'cors',
        'Sec-Fetch-Site': 'cross-site',
        'TE': 'trailers'
    };


    var dataString = `client_id=${client_id}&scope=${scope} openid profile offline_access&grant_type=refresh_token&client_info=1&x-client-SKU=msal.js.browser&x-client-VER=3.7.1&x-ms-lib-capability=retry-after,h429&x-client-current-telemetry=5|61,0,,,|,&x-client-last-telemetry=5|0|||0,0&refresh_token=${refresh_token}`;

    var options = {
        url: 'https://login.microsoftonline.com/660a30b5-8e2e-4769-b9eb-4af28bfd12bd/oauth2/v2.0/token',
        method: 'POST',
        headers: headers,
        gzip: true,
        body: dataString
    };

    return new Promise((resolve, reject) => {
        request(options, (error, response, body) => {
            if (error) {
                reject(error);
                return;
            }
            if (response.statusCode == 400) {
                reject(new Error(`Request failed with status code ${response.statusCode}. Is your refresh token or client id right?`));

            } else if (response.statusCode !== 200) {
                reject(new Error(`Request failed with status code ${response.statusCode}`));
                return;
            }
            const responseBody = JSON.parse(body);
            resolve(responseBody);
        });
    });
}


async function RenewTokens() {

    var ic3_token = await GenTokens("https://ic3.teams.office.com/Teams.AccessAsUser.All")
    var authz = await GenTokens("https://api.spaces.skype.com/Authorization.ReadWrite")


    // To aquire a skypetoken we must first generate an authz token, and with it do the following authz request wich will give us a skypetoken. The skypetoken lasts a day, unlike all the other tokens that expire in around 4 hours. I am using got instead of request because i can't seem to get this working with request.



    const response = await got.post('https://teams.microsoft.com/api/authsvc/v1.0/authz', {
        headers: {
            'authorization': 'Bearer ' + authz['access_token']

        }
    });

    const skypetoken = JSON.parse(response['body'])['tokens']['skypeToken']

    tokens = {
        "skypetoken": skypetoken,
        "https://ic3.teams.office.com/Teams.AccessAsUser.Al": ic3_token['access_token'],
        "https://api.spaces.skype.com/Authorization.ReadWrite": authz['access_token']
    }



}


async function Setup() {


    const data = await fs.promises.readFile('config.json', 'utf8');
    const config = JSON.parse(data);

    client_id = config.client_id;
    refresh_token = config.refresh_token;

    await RenewTokens();

}

Setup()

app.listen(port, () => {
    console.log(`Backend is running on port: ${port}`);
});
