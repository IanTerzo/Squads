<script>
import { onMount } from 'svelte';
import { page } from '$app/stores';
import { reloadTeams, teams, getConversation } from '../../../app.js';

let id;
id = $page.params.id;
let topic;
topic = $page.params.topic;
var conversation = {"replyChains": {}}
onMount(async () => {
    conversation = await getConversation(id,topic)
    console.log(conversation)
  });


</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	{#each Object.entries($teams) as [teamId, team]}
        {#if teamId === id}

            <div id="teamInfo">
            <img width="50px" height="50px" src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg">
            <span id="teamTitle"> {team.name} </span>
            </div>
            <div class="selgroup">
            {#each team.topics as topic}
                <a class="linkpage" href='../../team/{id}/{topic.id}'><span># {topic.name}</span></a>
            {/each}
            </div>
        {/if}
 	{/each}

</section>
<section>
{#each Object.entries(conversation['replyChains']) as [number, replyChain]}
            <div class="activity-box">
                {#each replyChain.messages as message}

                    {@html message.content}
                {/each}
            </div>
            {/each}


</section>
<style>
#teamInfo{
display:flex;
align-items: center;
gap:10px;
margin-bottom:20px;
}

#teamTitle{
    color:white;
	position:relative;
	font-size: 16px;

}
</style>

