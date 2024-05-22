<script>
import { onMount } from 'svelte';
import { goto } from '$app/navigation';
import { page } from '$app/stores';

let id;
$: id = $page.params.id;

onMount(() => {
    goto(`/team/${id}/${id}`);
});
</script>

<svelte:head>
    <title>Redirecting...</title>
    <meta name="description" content="Redirecting to another page" />
</svelte:head>