import { persisted } from 'svelte-persisted-store'

export const teams = persisted('persistent', {})

export async function authorize(email, password) {
    try {
        const response = await fetch(`http://localhost:5102/authorize`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ email, password }),
        });

        if (response.ok || response.status == 401)  {
           return response.text();
            
        }
        else {
            throw new Error('Network response was not ok');
        }

    } catch (error) {
        console.error('There was a problem with the fetch operation:', error);
    }
}

export async function reloadTeams() {
    try {
        const response = await fetch('http://localhost:5102/user-properties');
        if (!response.ok) {
            if (response.status == 401){
                window.location.href ="../../../../../../../login"  // temporary fix
            }
            throw new Error('Network response was not ok');
        }
        const data = await response.json();
        const teamsOrder = JSON.parse(data['teamsOrder']);

        const tempTeams = {};
        for (const team of teamsOrder) {
            const teamResponse = await fetch(`http://localhost:5102/team-details/${team.teamId}`);
            if (!teamResponse.ok) {
                if (teamResponse.status == 401){
                    window.location.href ="../../../../../../../login" 
                }
                throw new Error('Network response was not ok');
            }
            const teamData = await teamResponse.json();

            var topics = [{
                'id': teamData['id'],
                'name': 'General'
            }]

            if (teamData['threadProperties']['topics']) {
                const parsedTopics = JSON.parse(teamData['threadProperties']['topics'])
                topics.push(...parsedTopics);
            }

            tempTeams[teamData["id"]] = {
                "id": teamData["id"],
                "name": teamData['threadProperties']['spaceThreadTopic'],
                "topics": topics
            };
        }

        teams.set(tempTeams);

    } catch (error) {
        console.error('There was a problem with the fetch operation:', error);
    }
}

export async function getConversation(teamId, topicId) {
    try {
        var messages = []
        const response = await fetch(`http://localhost:5102/team-conversation/${teamId}/${topicId}`);
        if (!response.ok) {
            if (response.status == 401){
                window.location.href ="../../../../../../../login" 
            }
            throw new Error('Network response was not ok');
        }
        const data = await response.json();

        const reversedReplyChains = {};
        Object.keys(data.replyChains).reverse().forEach(key => {
            reversedReplyChains[key] = data.replyChains[key];
        });

        // Reverse the order of messages in each reply chain
        Object.keys(reversedReplyChains).forEach(key => {
            reversedReplyChains[key].messages.reverse();
        });

        console.log(reversedReplyChains)
        return reversedReplyChains;


    } catch (error) {
        console.error('There was a problem with the fetch operation:', error);
    }
}

export async function profilePicture(userId) {
    try {
        const response = await fetch(`http://localhost:5102/profilePicture/${userId}`);
        if (!response.ok) {           
            if (response.status == 401){
                window.location.href ="../../../../../../../login" 
            }
            throw new Error('Network response was not ok');
        }
        const blob = await response.blob();

        const imageUrl = URL.createObjectURL(blob);
        return imageUrl;

    } catch (error) {
        console.error('There was a problem with the fetch operation:', error);
    }
}



export async function authorizeImage(imageId) {
    try {
        const response = await fetch(`http://localhost:5102/image/${imageId}`);
        if (!response.ok) {
            if (response.status == 401){
                window.location.href ="../../../../../../../login" 
            }
            throw new Error('Network response was not ok');
        }
        const blob = await response.blob();

        const imageUrl = URL.createObjectURL(blob);
        return imageUrl;

    } catch (error) {
        console.error('There was a problem with the fetch operation:', error);
    }
}
