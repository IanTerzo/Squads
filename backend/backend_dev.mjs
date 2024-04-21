import request from 'request';
import got from 'got';

// Search "RefreshToken" in localstorage
const client_id = ""

// Search "RefreshToken" in localstorage
const refresh_token = ""

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

async function Main() {
    await RenewTokens()
    const userProperties = await UserProperties()
    console.log(userProperties)
}

Main()
