<script lang="ts">
	let result = '';
	let error = '';

	async function testBackend() {
		error = '';
		result = 'Testing...';

		try {
			// Test 1: Health check
			const health = await fetch('/api/health');
			result += '\n✓ API health: ' + health.status;

			// Test 2: Signup
			const signup = await fetch('/api/auth/signup', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					email: 'test@test.com',
					password: 'test123',
					name: 'Test User'
				})
			});
			const signupData = await signup.json();
			result += '\n✓ Signup: ' + JSON.stringify(signupData, null, 2);

			// Test 3: Login
			const login = await fetch('/api/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					email: 'test@test.com',
					password: 'test123'
				})
			});
			const loginData = await login.json();
			result += '\n✓ Login: ' + JSON.stringify(loginData, null, 2);

		} catch (e: any) {
			error = e.message;
			result += '\n✗ Error: ' + e.message;
		}
	}
</script>

<div style="padding: 2rem; max-width: 800px; margin: 0 auto;">
	<h1>API Connection Test</h1>

	<button on:click={testBackend} style="padding: 1rem 2rem; font-size: 1.2rem; cursor: pointer;">
		Test Backend Connection
	</button>

	{#if error}
		<div style="margin-top: 2rem; padding: 1rem; background: #fee; border: 1px solid #f00; color: #900;">
			<strong>Error:</strong> {error}
		</div>
	{/if}

	{#if result}
		<pre style="margin-top: 2rem; padding: 1rem; background: #f5f5f5; border: 1px solid #ddd; white-space: pre-wrap;">
{result}
		</pre>
	{/if}
</div>
