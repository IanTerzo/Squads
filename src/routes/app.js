import { persisted } from 'svelte-persisted-store'

export const teams = persisted('persistent', {})

export async function reloadTeams() {
    try {
      const response = await fetch('http://localhost:5102/user-properties');
      if (!response.ok) {
        throw new Error('Network response was not ok');
      }
      const data = await response.json();
      const teamsOrder = JSON.parse(data['teamsOrder']);

      const tempTeams = {};
      for (const team of teamsOrder) {
        const teamResponse = await fetch(`http://localhost:5102/team-details/${team.teamId}`);
        if (!teamResponse.ok) {
          throw new Error('Network response was not ok');
        }
        const teamData = await teamResponse.json();

        var topics = [{'id':teamData['id'], 'name':'General'}]

        if (teamData['threadProperties']['topics']){
          topics = JSON.parse(teamData['threadProperties']['topics'])
        }

        tempTeams[teamData["id"]] = {"id":teamData["id"], "name":teamData['threadProperties']['spaceThreadTopic'], "topics": topics};
      }

      teams.set(tempTeams);
      
    } catch (error) {
      console.error('There was a problem with the fetch operation:', error);
    }
}

export async function getConversation(teamId, topicId){
  try {
    console.log(teamId, topicId);
      const response = await fetch(`http://localhost:5102/team-conversation/${teamId}/${topicId}`);
      if (!response.ok) {
        throw new Error('Network response was not ok');
      }
      const data = await response.json();
      return data;

    } catch (error) {
      console.error('There was a problem with the fetch operation:', error);
    }
}
