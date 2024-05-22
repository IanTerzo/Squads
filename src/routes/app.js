import { persisted } from 'svelte-persisted-store'

export const teams = persisted('persistent', {}, {
    storage: 'local',
    syncTabs: true,
    onWriteError: (error) => {
      console.error("Write Error:", error);
    },
    onParseError: (raw, error) => {
      console.error("Parse Error:", raw, error);
    }
  }); 


export async function reloadTeams() {
    try {
        const response = await fetch('/api/userTeams');
        if (!response.ok) {
            throw new Error('Network response was not ok');
        }
        
        const data = await response.json();  

        teams.set(data.teams);

    } catch (error) {
        console.error('There was a problem with the fetch operation:', error);
    }
}