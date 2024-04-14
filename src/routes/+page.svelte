<script>
import { onMount } from 'svelte';
import { reloadTeams, teams,} from './app.js';

onMount(reloadTeams);

</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	{#each Object.entries($teams) as [id, team]}
		<div><a href='team/{id}'>{team.name}</a></div>
 	{/each}

</section>


