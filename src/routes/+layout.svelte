<script>
	import './styles.css';
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

<style>
:global(body){
	font-family: "Montserrat", sans-serif;
	font-optical-sizing: auto;
	font-style: normal;
	background-color: #3c3c3c;
	overflow-y: hidden;
	
}

:global(main) {
    padding: 75px;
    padding-top:80px;
    margin-top:25px;
    display:flex;
    gap:30px;

}
:global(.linkpage) {
    margin-bottom: 8.5px;
    font-size: 15px;
    background-color: #333;
    width: 312px;
	min-height: 37px;
    padding: 7px;
    border-radius:10px;
    display:block;
    cursor: pointer;
    text-decoration:none;
	display: flex;
	align-items: center;
}

:global(.linkpage span) {

	color: white;
	vertical-align: super;

}

:global(.infotext){
	color:white;
	position:relative;
}

:global(.activity-box){
display: flex;
flex-direction: column;
background-color: #333;
width:690px;
height:fit-content;
margin-top:15px;
border-radius:10px;
padding:14px;
color: white;
}

:global(.post-sender-info){
display:flex;
align-items: center;
gap:10px;
margin-bottom:10px;
}

:global(.activity-box-img){
margin-top:15px;
border-radius: 5px;
}

:global(.pfp-img){
	width:32px;
	height:32px;
	border-radius: 3px;
}

:global(.selgroup){
	color:white;
	display: flex;
	flex-direction:column;
	overflow-y: scroll;
	margin-top:15px;
	height: 85vh;
	scrollbar-width: none;
}

:global(.linkpage img){
	width:32px;
	height:32px;
	margin-right: 9px;
	border-radius: 3px;
	margin-left: 3px;
}
</style>
