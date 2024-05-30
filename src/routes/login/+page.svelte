<script>

	let loader;
	let continueBtn; 
	
	let email;
	let password; 
	
	let error;
	
	async function authorize(){
		error.style.display="none";
		
		if (email.value && password.value){
			continueBtn.style.display="none";
			loader.style.display="block";
	
			const response = await fetch(`/api/authorize`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({"email" : email.value, "password" : password.value}),
			});
	 
			if (response.ok)  {
	
				console.log('Authorization successful');
				window.location.href ="../"
	
			   return response.text();
				
			}
			else if (response.status == 401){
				error.textContent = "Unauthorized"
				error.style.display="block";
	
				loader.style.display="none";
				continueBtn.style.display="block";
	
			}
			else {
				error.textContent = "Network response was not ok";
				error.style.display="block";
				
				loader.style.display="none";
				continueBtn.style.display="block";
	
				throw new Error('Network response was not ok');
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
				<span class="overInp">Email</span>
				<div><input class="field" bind:this={email}/></div>
			</div>
	
			<div class="group">
				<span class="overInp">Password</span>
				<div><input class="field" type="password" bind:this={password}/></div>
			</div>
		</div>
	
		<div style="display: none;" id="refresh" class="group">
			<span class="overInp">Refresh Token</span>
			<div><input class="field"/></div>
		</div>
	
		<span on:click={authorize} id="loginButton"><div bind:this={continueBtn} style="display: block;">Continue</div> <div bind:this={loader} style="display: none;" class="loader"></div> </span>
	
		<div bind:this={error} style="display: none;" id="error">  </div>
		
	</section>
	