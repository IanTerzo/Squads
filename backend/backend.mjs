import express from 'express';
import bodyParser from 'body-parser';
import request from 'request';
import cors from 'cors';
import got from 'got';
import fs, { cpSync } from 'fs';

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

app.get('/image/:imageId', async (req, res) => {
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
});


app.get('/profilePicture/:userId/:displayName', async (req, res) => {
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
});


app.get('/team-conversation/:teamId/:topicId', async (req, res) => {
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
        'authorization': 'Bearer ' + tokens['https://chatsvcagg.teams.microsoft.com/.default'],
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

        'Origin': 'https://teams.microsoft.com',

    };

    var dataString = `client_id=${client_id}&scope=${scope} openid profile offline_access&grant_type=refresh_token&client_info=1&x-client-SKU=msal.js.browser&x-client-VER=3.7.1&refresh_token=${refresh_token}&claims={"access_token":{"xms_cc":{"values":["CP1"]}}}`;

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
            if (response.statusCode !== 200) {
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
    var chatsvcagg_token = await GenTokens("https://chatsvcagg.teams.microsoft.com/.default");
    var skyptoken_asm = await GenTokens("https://api.spaces.skype.com/Authorization.ReadWrite")

    // To aquire a skypetoken we must first generate an skyptoken_asm token, and with it do the following authz request wich will give us a skypetoken. The skypetoken lasts a day, unlike all the other tokens that expire in around 4 hours. I am using got instead of request because i can't seem to get this working with request.

    const response = await got.post('https://teams.microsoft.com/api/authsvc/v1.0/authz', {
        headers: {
            'authorization': 'Bearer ' + skyptoken_asm['access_token']

        }
    });

    const skypetoken = JSON.parse(response['body'])['tokens']['skypeToken']

    tokens = {
        "skypetoken": skypetoken,
        "https://ic3.teams.office.com/Teams.AccessAsUser.Al": ic3_token['access_token'],
        "https://api.spaces.skype.com/Authorization.ReadWrite": skyptoken_asm['access_token'],
        "https://chatsvcagg.teams.microsoft.com/.default": chatsvcagg_token['access_token']
    }
}


async function authorizeImage(imageId) {
    var headers = {
        'authorization': 'skype_token ' + tokens['skypetoken'],
    };

    var options = {
        url: `https://eu-prod.asyncgw.teams.microsoft.com/v1/objects/${imageId}/views/imgo?v=1`,
        headers: headers,
        gzip: true,
        encoding: null
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
            resolve(body);
        });
    });
}

async function authorizeProfilePicture(userId, displayName) {

    const response = await got(`https://teams.microsoft.com/api/mt/part/emea-02/beta/users/${userId}/profilepicturev2`, {
        searchParams: {
            'displayname': displayName,
            'size': 'HR64x64'
        },
        headers: {
            'Referer': 'https://teams.microsoft.com/_',
            'Cookie': `authtoken=Bearer=${tokens['https://api.spaces.skype.com/Authorization.ReadWrite']}&Origin=https://teams.microsoft.com;`,

        }
    });

    return response['rawBody']
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