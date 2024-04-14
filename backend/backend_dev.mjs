
import request from 'request';

// Passed in authz req
const authz_token = "Bearer eyJ0eXAiOiJKV1QiLCJub25jZSI6IlY2OXQ5NnhJNzhVYlpIRnRuV29pRXVULWJUSkg2UTBaVXczb2M4QTJaMm8iLCJhbGciOiJSUzI1NiIsIng1dCI6InEtMjNmYWxldlpoaEQzaG05Q1Fia1A1TVF5VSIsImtpZCI6InEtMjNmYWxldlpoaEQzaG05Q1Fia1A1TVF5VSJ9.eyJhdWQiOiJodHRwczovL2FwaS5zcGFjZXMuc2t5cGUuY29tIiwiaXNzIjoiaHR0cHM6Ly9zdHMud2luZG93cy5uZXQvNjYwYTMwYjUtOGUyZS00NzY5LWI5ZWItNGFmMjhiZmQxMmJkLyIsImlhdCI6MTcxMzA5NzYwMSwibmJmIjoxNzEzMDk3NjAxLCJleHAiOjE3MTMxNzE0ODgsImFjY3QiOjAsImFjciI6IjEiLCJhaW8iOiJBU1FBMi84V0FBQUFNZmVDemRHZ1NGRWxuK3prbllDN3c3ZmlNcXVPUEZaWnpzZ0JCaEJWMWMwPSIsImFtciI6WyJwd2QiXSwiYXBwaWQiOiI1ZTNjZTZjMC0yYjFmLTQyODUtOGQ0Yi03NWVlNzg3ODczNDYiLCJhcHBpZGFjciI6IjAiLCJhdXRoX3RpbWUiOjE3MTIxNTEwMTUsImZhbWlseV9uYW1lIjoiQmFsZGVsbGkiLCJnaXZlbl9uYW1lIjoiSWFuIiwiaXBhZGRyIjoiODEuMTYuMTYzLjE3NCIsIm5hbWUiOiJJYW4gQmFsZGVsbGkiLCJvaWQiOiIxNWRlNDI0MS1lOWJlLTQ5MTAtYTYwZi0zZjM3ZGQ4NjUyYjgiLCJvbnByZW1fc2lkIjoiUy0xLTUtMjEtMTQwOTA4MjIzMy00NDg1Mzk3MjMtNjgyMDAzMzMwLTE3MDM2IiwicHVpZCI6IjEwMDMyMDAyQ0RENEIxQjciLCJyaCI6IjAuQVhRQXRUQUtaaTZPYVVlNTYwcnlpXzBTdlZmOUZjeHNMQmRCcUl5RHNkVnJTNzdpQVBJLiIsInNjcCI6IkF1dGhvcml6YXRpb24uUmVhZFdyaXRlIHVzZXJfaW1wZXJzb25hdGlvbiIsInNpZCI6ImU1ZDQyZGZhLWFkMTQtNDYwNS05N2NjLTlkOTAyZDEwNjE4YSIsInNpZ25pbl9zdGF0ZSI6WyJrbXNpIl0sInN1YiI6IkxxZVlPeFB4MFl0SGNqUVJBSzVmakxNNFVlSVpkbVJGS05oRU1xd3lPZzQiLCJ0ZW5hbnRfY3RyeSI6IlNFIiwidGlkIjoiNjYwYTMwYjUtOGUyZS00NzY5LWI5ZWItNGFmMjhiZmQxMmJkIiwidW5pcXVlX25hbWUiOiJpYW4uYmFsZGVsbGlAaGl0YWNoaWd5bW5hc2lldC5zZSIsInVwbiI6Imlhbi5iYWxkZWxsaUBoaXRhY2hpZ3ltbmFzaWV0LnNlIiwidXRpIjoiaVA3WTQ1cDA4MDJkVmVpRU5hczNBQSIsInZlciI6IjEuMCIsIndpZHMiOlsiYjc5ZmJmNGQtM2VmOS00Njg5LTgxNDMtNzZiMTk0ZTg1NTA5Il0sInhtc19jYyI6WyJDUDEiXSwieG1zX3NzbSI6IjEifQ.fD5vzjwMxjkzkPB8Bx4LBg2CgJu4CcMmeQNYy-Lt_5PqZl9up2i8SgD32sMDJ9K9IRklfqfRg855O8rqAR3JzR3jVERmdcxmNGTl51PVngaepR0jLPzc4xBnVFQi0FRL4peW-VM9VNX0NcPS-tgrkndttgYJgz1x3K_g-6nxZWrF0fBJcb8FU_aZGpEOm7u92k_giUHr8aphhDf7susv1VZUx4_MDEAVOmWU9UgQ7I5YwLZ1rxhYeBKRIJWvoHMPudP-ACl8jBd2fN2MXPGggQzudqLDT7Nm7kqauCIfIEJuTwbgBpurpGCITW-OYAgGavhTa1XYnlcDd8zOsCG1LA"

// Deriver from authz req
const skypetoken = "eyJhbGciOiJSUzI1NiIsImtpZCI6IjYwNUVCMzFEMzBBMjBEQkRBNTMxODU2MkM4QTM2RDFCMzIyMkE2MTkiLCJ4NXQiOiJZRjZ6SFRDaURiMmxNWVZpeUtOdEd6SWlwaGsiLCJ0eXAiOiJKV1QifQ.eyJpYXQiOjE3MTMwOTc5MDEsImV4cCI6MTcxMzE3MTQ4OCwic2t5cGVpZCI6Im9yZ2lkOjE1ZGU0MjQxLWU5YmUtNDkxMC1hNjBmLTNmMzdkZDg2NTJiOCIsInNjcCI6NzgwLCJjc2kiOiIxNzEzMDk3NjAxIiwidGlkIjoiNjYwYTMwYjUtOGUyZS00NzY5LWI5ZWItNGFmMjhiZmQxMmJkIiwicmduIjoiZW1lYSIsImFhZF91dGkiOiJpUDdZNDVwMDgwMmRWZWlFTmFzM0FBIiwiYWFkX2lhdCI6MTcxMzA5NzYwMSwiYWFkX2FwcGlkIjoiNWUzY2U2YzAtMmIxZi00Mjg1LThkNGItNzVlZTc4Nzg3MzQ2In0.DzeCW3oksBtsa26D9yDqpPN7LS57TmmKGxbgjdtzmEpSWk4xF15YRg3iq7UbMTVPwAkJusPXO9EG8_Pe3sJnqdBi4faGpFkqwAo3yTBXLo0_kTtf5uDknESvaVAOHlq-me-cTovLMeoVwI58Ax9vSSeYtUFYUR139-lsMpbrJ2a-cxEDsNV0wM6eAVcoJaHu_WIU3RulqqBh3S95zggLluqbDTmb42-V9BsDmeUqA6fhJmAXYLlZS4iUeLUJmnwQ6KdL3Qgm0TtcRQtT-VJKVD_VQJSj7KBfXeMUcHRBptFQ6cZGklLWhMhe5H0ikTxTwczAyzq6F8f63aX8BViV-A"

// found in is?Prefetched
const volutile_maintoken = "Bearer eyJ0eXAiOiJKV1QiLCJub25jZSI6InpPOEY3SndUbTlsTXpqZm5PamxqMFQ4S0x1WGkwMTBYd3FQYXhQMFdGaDgiLCJhbGciOiJSUzI1NiIsIng1dCI6InEtMjNmYWxldlpoaEQzaG05Q1Fia1A1TVF5VSIsImtpZCI6InEtMjNmYWxldlpoaEQzaG05Q1Fia1A1TVF5VSJ9.eyJhdWQiOiJodHRwczovL2NoYXRzdmNhZ2cudGVhbXMubWljcm9zb2Z0LmNvbSIsImlzcyI6Imh0dHBzOi8vc3RzLndpbmRvd3MubmV0LzY2MGEzMGI1LThlMmUtNDc2OS1iOWViLTRhZjI4YmZkMTJiZC8iLCJpYXQiOjE3MTMwOTc2MDIsIm5iZiI6MTcxMzA5NzYwMiwiZXhwIjoxNzEzMTc3Nzg2LCJhY2N0IjowLCJhY3IiOiIxIiwiYWlvIjoiQVRRQXkvOFdBQUFBREM2N3hPSFFCckNhNHZ0UUw0aytmZHdiSFV1YlF3ZzVMbXhmdXZMeFgvWXYzd2ZDRzN3VUh1TUNNTndaUk5yMyIsImFtciI6WyJwd2QiXSwiYXBwaWQiOiI1ZTNjZTZjMC0yYjFmLTQyODUtOGQ0Yi03NWVlNzg3ODczNDYiLCJhcHBpZGFjciI6IjAiLCJmYW1pbHlfbmFtZSI6IkJhbGRlbGxpIiwiZ2l2ZW5fbmFtZSI6IklhbiIsImlwYWRkciI6IjgxLjE2LjE2My4xNzQiLCJuYW1lIjoiSWFuIEJhbGRlbGxpIiwib2lkIjoiMTVkZTQyNDEtZTliZS00OTEwLWE2MGYtM2YzN2RkODY1MmI4Iiwib25wcmVtX3NpZCI6IlMtMS01LTIxLTE0MDkwODIyMzMtNDQ4NTM5NzIzLTY4MjAwMzMzMC0xNzAzNiIsInB1aWQiOiIxMDAzMjAwMkNERDRCMUI3IiwicmgiOiIwLkFYUUF0VEFLWmk2T2FVZTU2MHJ5aV8wU3ZYV2FON0ZlenFOUGdNYUp1em1fWkd6aUFQSS4iLCJzY3AiOiJ1c2VyX2ltcGVyc29uYXRpb24iLCJzdWIiOiJjZ0xfbkF4X1hsR3YwSk5zQjlSUXk4ZzBDVGlteXV6eXhoREFaUU9ITmp3IiwidGVuYW50X3JlZ2lvbl9zY29wZSI6IkVVIiwidGlkIjoiNjYwYTMwYjUtOGUyZS00NzY5LWI5ZWItNGFmMjhiZmQxMmJkIiwidW5pcXVlX25hbWUiOiJpYW4uYmFsZGVsbGlAaGl0YWNoaWd5bW5hc2lldC5zZSIsInVwbiI6Imlhbi5iYWxkZWxsaUBoaXRhY2hpZ3ltbmFzaWV0LnNlIiwidXRpIjoiX3hHRFlabUhPRS1wNzRadWRUVktBQSIsInZlciI6IjEuMCIsIndpZHMiOlsiYjc5ZmJmNGQtM2VmOS00Njg5LTgxNDMtNzZiMTk0ZTg1NTA5Il0sInhtc19jYyI6WyJDUDEiXSwieG1zX3NzbSI6IjEifQ.BVYvcc5B7RxKKhXunY53xegMa3lVFzXB-secj9FB3_v_ZctqQdYb3ImW01R5fzP91yLnrxTR5uF96mF-V0jlpQk1dla0vuJKjD6Smdw3qu20SmOl7p1LeVTR8f2-DVtFK5MqW4pb8mF48meUtvSFcfmkUihkOtJ720v2M6gcwyyAekBDNLg5AfPP8OTLnHTTqCqEvhmnjNfWvDEYelSvZBXy3waddPy7UvToDfCbariUAuu4T2tLnMa-Uk0HEctIVijSuRFZl6NJ6NvscD8iz0dfWj2rSnuHCwzY28GfUmbzC1GAIxUDzf60BF5MDuurzVz09T3xAEHmsYELoqUMHg"

// Found in threads?view after creating a team
const details_token = "Bearer eyJ0eXAiOiJKV1QiLCJub25jZSI6IjFNZzlGZGtuMERyLXBCeEtHY29jdDJEUmhVajFxUHBPaUZJam9IRGVCMlEiLCJhbGciOiJSUzI1NiIsIng1dCI6InEtMjNmYWxldlpoaEQzaG05Q1Fia1A1TVF5VSIsImtpZCI6InEtMjNmYWxldlpoaEQzaG05Q1Fia1A1TVF5VSJ9.eyJhdWQiOiJodHRwczovL2ljMy50ZWFtcy5vZmZpY2UuY29tIiwiaXNzIjoiaHR0cHM6Ly9zdHMud2luZG93cy5uZXQvNjYwYTMwYjUtOGUyZS00NzY5LWI5ZWItNGFmMjhiZmQxMmJkLyIsImlhdCI6MTcxMzA5NzYwMywibmJmIjoxNzEzMDk3NjAzLCJleHAiOjE3MTMxODQzMDMsImFjciI6IjEiLCJhaW8iOiJBVFFBeS84V0FBQUFlYWVoN25MNzVkWUsxMGtROTQrK3ZPT09yUnVEZWdvcWUvc0o2eGc2WHorWWxLSmZEUVhTdkM5S0RrMDdvNXN6IiwiYW1yIjpbInB3ZCJdLCJhcHBpZCI6IjVlM2NlNmMwLTJiMWYtNDI4NS04ZDRiLTc1ZWU3ODc4NzM0NiIsImFwcGlkYWNyIjoiMCIsImZhbWlseV9uYW1lIjoiQmFsZGVsbGkiLCJnaXZlbl9uYW1lIjoiSWFuIiwiaXBhZGRyIjoiODEuMTYuMTYzLjE3NCIsIm5hbWUiOiJJYW4gQmFsZGVsbGkiLCJvaWQiOiIxNWRlNDI0MS1lOWJlLTQ5MTAtYTYwZi0zZjM3ZGQ4NjUyYjgiLCJvbnByZW1fc2lkIjoiUy0xLTUtMjEtMTQwOTA4MjIzMy00NDg1Mzk3MjMtNjgyMDAzMzMwLTE3MDM2IiwicHVpZCI6IjEwMDMyMDAyQ0RENEIxQjciLCJyaCI6IjAuQVhRQXRUQUtaaTZPYVVlNTYwcnlpXzBTdlZUd3FqbWxnY2RJcFBnQ2t3RWdsYm5pQVBJLiIsInNjcCI6IlRlYW1zLkFjY2Vzc0FzVXNlci5BbGwiLCJzdWIiOiJETVJpTjBOdUJYNkFWT255YzJrSVJ3VFR0cmw2LVNoTmZRZFNDblU0cF9ZIiwidGlkIjoiNjYwYTMwYjUtOGUyZS00NzY5LWI5ZWItNGFmMjhiZmQxMmJkIiwidW5pcXVlX25hbWUiOiJpYW4uYmFsZGVsbGlAaGl0YWNoaWd5bW5hc2lldC5zZSIsInVwbiI6Imlhbi5iYWxkZWxsaUBoaXRhY2hpZ3ltbmFzaWV0LnNlIiwidXRpIjoielNCeWZZc3BmMGl3OG9IRjhpWUVBQSIsInZlciI6IjEuMCIsInhtc19jYyI6WyJDUDEiXSwieG1zX3NzbSI6IjEifQ.56-56P9NNSOnfjLK1Rc-4rsgduA5LBPmSi8PPSpJHbuylqHK9WQ9QIkJd9pfAewXocX5OMZUcmKOYQSGWhPkYNq3wQzTra51C8uWjFXaKgPQNmCVN56lmCzHlUUQuifEg3QfgIuht5e_86zYz2P5b_cdDcMN-t9rdsrk-YMb_Wv7z7VVXXoOrH7z7-Gr_jh4YEknuiP9agH0sgi4qA9H8vFZmXXbkHpyrAhgXxpzVNM1J8Z0N6TF34nwXO0o5bs7t7H4y_w1pTIbB0eweX27g51W51WE_8YcryBL9-BjGDgfUtkJydpiptGv__dkAkWcCARwtdVApaW3GbGxqE9rvw"

async function UserProperties() {
    var headers = {
        'Authentication': 'skypetoken=' + skypetoken,
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

async function TeamConversation(TeamID) {


    var headers = {
        'authorization': volutile_maintoken,
    };
    
    var options = {
        url: `https://teams.microsoft.com/api/csa/emea/api/v2/teams/${TeamID}/channels/${TeamID}?filterSystemMessage=true&pageSize=20`,
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
        'authorization': details_token,
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


async function Main(){
    var properties = await UserProperties()
    var conversations = await TeamDetails(JSON.parse(properties['teamsOrder'])[1]['teamId'])
    console.log(conversations)
}

Main()