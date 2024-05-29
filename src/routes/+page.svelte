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
		<a class="linkPage" href='team/{team.id}/{team.id}'><img src="/api/teamPicture/{team.teamSiteInformation.groupId}/{team.displayName}/{team.pictureETag}"><span>{team.displayName}</span></a>
 	{/each}
	<div>

</section>
