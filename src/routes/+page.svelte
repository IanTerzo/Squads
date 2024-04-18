<script>
import { onMount } from 'svelte';
import { reloadTeams, teams,} from './app.js';

onMount(reloadTeams);

</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />

	<link rel="preconnect" href="https://fonts.googleapis.com">

</svelte:head>

<section >
<div class="infotext"> Teams ^ </div>
	<div class="selgroup" id="teams">

	{#each Object.entries($teams) as [id, team]}
		<a class="linkpage" href='team/{id}'><img src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg"><span>{team.name}</span></a>
 	{/each}
	<div>

</section>

<section>
<div class="infotext"> Activity </div>
<div class="activity-box">
	<div class="post-sender-info">
	<img class="pfp-img" width="32px" height="32x" src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg">
	<span>John Toe</span>
	</div>

	<h3>Titel</h3>
	<span> Lorem ipsus dolor sit amet pasa que nada </span>
	<img class="activity-box-img" width="350px" height="220px" src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg">

	<span class="show-replies"> Show replies </span>
	<div class="replies">

	<div class="messages">
	<div class="post-sender-info">
	<img class="pfp-img" width="32px" height="32x" src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg">
	<span>John Toe</span>

	</div>
	<span>hello world! </span>
	</div>


	</div>
	</div>
</section>



