<script>
import { onMount } from 'svelte';
import { reloadTeams, teams} from './app.js';

onMount(reloadTeams);

</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
	<link rel="preconnect" href="https://fonts.googleapis.com">

</svelte:head>

<section >
<div class="infoText"> Teams </div>
	<div class="selGroup" id="teams">

	{#each Object.entries($teams) as [index, team]}
		<a class="linkPage" href='team/{team.id}/{team.id}'><img src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg"><span>{team.displayName}</span></a>
 	{/each}
	<div>

</section>
