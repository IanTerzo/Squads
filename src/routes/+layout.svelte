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
	font-family: "Montserrat", sans-serif !important;
	font-optical-sizing: auto;
	font-style: normal;
	background-color: #3c3c3c;
	overflow-y: hidden;
	line-height: 1.52 !important;
	
}

:global(a){
	color: #6698d9;
	text-decoration: none;
}

:global(span){
	font-size: 16px;
	font-family: "Montserrat", sans-serif !important;
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

	text-overflow: ellipsis;
	white-space: nowrap; 
    overflow: hidden;

}

:global(.infotext){
	color:white;
	position:relative;
}

:global(.activity-box){
display: flex;
flex-direction: column;
background-color: #333;
width:35vw;
height:fit-content;
margin-top:15px;
border-radius:10px;
padding:14px;
color: white;
}


:global(.titlespan){
    color:white;
	position:relative;
	font-size: 16.5px;
    font-weight: 600;
	margin-bottom: 12px;
}

:global(.post-sender-info){
display:flex;
align-items: center;
gap:10px;
margin-bottom:18px;
}

:global(#content p){
margin: 0px;
}


:global(#content img){
margin-top:10px;
margin-bottom: 10px;
border-radius: 5px;
}

:global(#content){
display: inline-grid;
}

:global(.pfp-img){
	width:32px;
	height:32px;
	border-radius: 3px;
}

:global(.show-replies){
	margin-top: 16px;
	margin-bottom: -12px;
	font-size: 14px;
	color: #6698d9;
	cursor: pointer;
}

:global([itemtype="http://schema.skype.com/Emoji"]){
	margin-top: 0px !important;
	margin-bottom: -5px !important;
}

:global(.reply){
margin-top: 25px;
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
