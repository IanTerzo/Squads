<script>
	import './styles/default.css';
	import Navbar from "./navbar.svelte";
</script>

<div class="app">
	<header>
		<Navbar/>
	</header>
	<main>
		<slot />
	</main>
</div>

