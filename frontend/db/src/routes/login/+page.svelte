<script lang="ts">
	import { login, auth_info_store } from '$lib/auth';
	import { get } from 'svelte/store';
    import { page } from '$app/stores';

	let username = '';
	let password = '';

	function handleSubmit() {
		// Handle login here
		let login_promise = login(username, password);

		login_promise.then(() => {
			console.log('Checking auth status');
			if (get(auth_info_store) !== null) {
				// Redirect to app which will check for auth
				// TODO: Redirect to the page the user was trying to access
                let url_params = new URLSearchParams(window.location.search);
                let redirect = url_params.get('redirect');
                // TODO: Just redirect to /app cause redirecting to orders/edit for example breaks loader stuff for some reason. Easier just to
                // redirect to /app and let the app handle the redirect
                console.log('Redirecting to', redirect);
                
                window.location.href = redirect ? redirect : '/app';
			} else {
				// TODO: Show error message
				alert('Login failed');
			}
		});
	}
</script>

<section class="hero is-primary is-fullheight">
	<div class="hero-body">
		<div class="container">
			<div class="columns is-centered">
                <div class="column is-5-tablet is-4-desktop is-3-widescreen">
                    <form action="" class="box" on:submit|preventDefault={handleSubmit}>
                        <figure class="image is-128x128 ml-auto mr-auto mb-4">
                            <!-- <img src={logo} alt="Logo" /> -->
                        </figure>
						<div class="field">
							<label for="" class="label">Username</label>
							<div class="control has-icons-left">
								<input
									type="username"
									placeholder=". . . . ."
									class="input"
									bind:value={username}
									required
								/>
								<span class="icon is-small is-left">
									<i class="fa fa-user"></i>
								</span>
							</div>
						</div>
						<div class="field">
							<label for="" class="label">Password</label>
							<div class="control has-icons-left">
								<input
									type="password"
									placeholder="*******"
									class="input"
									bind:value={password}
									required
								/>
								<span class="icon is-small is-left">
									<i class="fa fa-lock"></i>
								</span>
							</div>
						</div>
						<div class="field">
							<div class="columns is-centered">
								<div class="column is-narrow">
									<button class="button brand-color" type="submit"> Login </button>
								</div>
							</div>
						</div>
					</form>
				</div>
			</div>
		</div>
	</div>
</section>

<style>
    @keyframes gradient {
        0% {background-position: 0% 50%;}
        50% {background-position: 100% 50%;}
        100% {background-position: 0% 50%;}
    }

    .brand-color {
        background: #ffae23;
        color: #fff;
    }

    .brand-color:hover {
        background: #ffa200;
    }

    .hero {
        background: linear-gradient(0deg, #ff963fcc, #fdc36bcc, #ff9147cc);
        background-size: 200% 200%;
        animation: gradient 5s ease infinite;
    }
</style>