<script>
import {authorize} from '../app.js';

let loader;
let continueBtn; 

let email;
let password; 

let error;

async function getTokens(){
	error.style.display="none";

	if (email.value && password.value){
		continueBtn.style.display="none";
		loader.style.display="block";

		let body = await authorize(email.value, password.value)
		if (body == "OK"){
			console.log('Authorization successful');
            window.location.href ="../"
		}
		else {
			error.textContent = body;
			error.style.display="block";

			loader.style.display="none";
			continueBtn.style.display="block";

		}
	}
}

</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />

	<link rel="preconnect" href="https://fonts.googleapis.com">

</svelte:head>

<section id="login">
	<div id="title">Login with Teams </div>

	<div id="credentials" style="display: block;">
		<div class="group">
			<span class="over-inp">Email</span>
			<div><input class="field" bind:this={email}/></div>
		</div>

		<div class="group">
			<span class="over-inp">Password</span>
			<div><input class="field" type="password" bind:this={password}/></div>
		</div>
	</div>

	<div style="display: none;" id="refresh" class="group">
		<span class="over-inp">Refresh Token</span>
		<div><input class="field"/></div>
	</div>

	<span on:click={getTokens} id="login-button"><div bind:this={continueBtn} style="display: block;">Continue</div> <div bind:this={loader} style="display: none;" class="loader"></div> </span>

	<div bind:this={error} style="display: none;" id="error">  </div>
	
</section>

<style>

.loader {
  width: 42px;
  aspect-ratio: 4;
  --_g: no-repeat radial-gradient(circle closest-side,#ececec 90%,#0000);
  background: 
    var(--_g) 0%   50%,
    var(--_g) 50%  50%,
    var(--_g) 100% 50%;
  background-size: calc(100%/3) 100%;
  animation: l7 1s infinite linear;
}

@keyframes l7 {
    33%{background-size:calc(100%/3) 0%  ,calc(100%/3) 100%,calc(100%/3) 100%}
    50%{background-size:calc(100%/3) 100%,calc(100%/3) 0%  ,calc(100%/3) 100%}
    66%{background-size:calc(100%/3) 100%,calc(100%/3) 100%,calc(100%/3) 0%  }
}

#login{
	
	position: absolute;
	right:50%;
	transform: translateX(50%);
	width: 29vw;
	min-width: 300px;
	background-color: #333;
	padding: 14px;
	border-radius: 8px;
}

#title{
     font-size:22px;
	 text-align: center;
	 width: 100%;
     color:white;
}

.over-inp{
	color: white;
}

.field{
	width: calc(100% - 14px);
	background-color: #444;
	border: none;
	color: white;
	border-radius: 3px;
	padding: 8px;
	font-size: 16px;
	margin-top: 12px;
}

.divisor{
	width: 100%;
	text-align: center;
	color: white;
	cursor: pointer;
	font-size: 13.5px;


}

.field:focus{
    outline: none;
}

.group{
	margin-top: 14px;
}

#login-button{
	margin-top: 30px;
	margin-right: auto;
	margin-left: auto;

	text-align: center;
	display: block;

	background-color: #444;
	width: 120px;
	padding: 6px;
	color: white;
	border-radius: 5px;
	cursor: pointer;
	height: 30px;
	
	display: flex;
	align-items: center;
	justify-content: center;
	user-select: none; 
}

#error{
	margin-top: 14px;
	text-align: center;
	color: rgb(206, 60, 60);
}

</style>