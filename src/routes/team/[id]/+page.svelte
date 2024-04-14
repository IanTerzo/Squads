<script>
import { onMount } from 'svelte';
import { page } from '$app/stores';
import { reloadTeams, teams,} from '../../app.js';

onMount(reloadTeams);
let id;
$: id = $page.params.id;

</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	{#each Object.entries($teams) as [teamId, team]}
        {#if teamId === id}
            <p>{team.name}</p>
            {#each team.topics as topic}
                <a href="../team/{id}/{topic.id}">{topic.name}</a>
            {/each}
        {/if}
 	{/each}

</section>


