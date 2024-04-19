<script>
import { onMount } from 'svelte';
import { goto } from '$app/navigation';
import { page } from '$app/stores';

let id;
$: id = $page.params.id;

onMount(() => {
    // Redirect to the desired page
    goto(`/team/${id}/${id}`);
});
</script>

<svelte:head>
    <title>Redirecting...</title>
    <meta name="description" content="Redirecting to another page" />
</svelte:head>

<!-- You can leave the rest of the code as is, but it won't be executed since the page will redirect immediately -->
