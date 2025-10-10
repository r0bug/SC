<script lang="ts">
	import { auth } from '$lib/stores/auth';

	let name = '';
	let email = '';
	let password = '';
	let confirmPassword = '';
	let error = '';
	let loading = false;

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		if (password !== confirmPassword) {
			error = 'Passwords do not match';
			return;
		}

		if (password.length < 8) {
			error = 'Password must be at least 8 characters';
			return;
		}

		loading = true;

		try {
			await auth.signup(email, password, name);
		} catch (err: any) {
			error = err.message || 'Signup failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="auth-container">
	<form class="auth-form card" on:submit={handleSubmit}>
		<h1>Sign Up</h1>
		<p class="subtitle">Create your SagensContact account</p>

		{#if error}
			<div class="alert alert-error">
				{error}
			</div>
		{/if}

		<div class="form-group">
			<label for="name">Full Name</label>
			<input
				id="name"
				type="text"
				bind:value={name}
				required
				placeholder="John Doe"
				disabled={loading}
			/>
		</div>

		<div class="form-group">
			<label for="email">Email</label>
			<input
				id="email"
				type="email"
				bind:value={email}
				required
				placeholder="you@example.com"
				disabled={loading}
			/>
		</div>

		<div class="form-group">
			<label for="password">Password</label>
			<input
				id="password"
				type="password"
				bind:value={password}
				required
				placeholder="••••••••"
				disabled={loading}
				minlength="8"
			/>
		</div>

		<div class="form-group">
			<label for="confirmPassword">Confirm Password</label>
			<input
				id="confirmPassword"
				type="password"
				bind:value={confirmPassword}
				required
				placeholder="••••••••"
				disabled={loading}
				minlength="8"
			/>
		</div>

		<button type="submit" class="btn btn-primary btn-block" disabled={loading}>
			{loading ? 'Creating account...' : 'Sign Up'}
		</button>

		<div class="divider">or</div>

		<p class="text-center">
			Already have an account?
			<a href="/auth/login">Sign In</a>
		</p>

		<div class="mock-notice">
			<strong>🔒 Alpha Notice:</strong> Authentication is functional. Your account will be created in the database.
		</div>
	</form>
</div>

<style>
	.auth-container {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background: linear-gradient(135deg, var(--primary-light), var(--primary));
		padding: 2rem;
	}

	.auth-form {
		width: 100%;
		max-width: 400px;
		padding: 2.5rem;
	}

	h1 {
		font-size: 2rem;
		margin-bottom: 0.5rem;
		text-align: center;
	}

	.subtitle {
		color: var(--gray-600);
		text-align: center;
		margin-bottom: 2rem;
	}

	.form-group {
		margin-bottom: 1.5rem;
	}

	label {
		display: block;
		margin-bottom: 0.5rem;
		font-weight: 500;
	}

	input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 4px;
		font-size: 1rem;
	}

	input:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
	}

	input:disabled {
		background: var(--gray-100);
	}

	.btn-block {
		width: 100%;
		padding: 0.875rem;
		font-size: 1rem;
	}

	.divider {
		text-align: center;
		margin: 1.5rem 0;
		color: var(--gray-500);
		position: relative;
	}

	.divider::before,
	.divider::after {
		content: '';
		position: absolute;
		top: 50%;
		width: calc(50% - 2rem);
		height: 1px;
		background: var(--gray-300);
	}

	.divider::before {
		left: 0;
	}

	.divider::after {
		right: 0;
	}

	.text-center {
		text-align: center;
	}

	.alert {
		padding: 0.75rem 1rem;
		border-radius: 4px;
		margin-bottom: 1rem;
	}

	.alert-error {
		background: #fef2f2;
		border: 1px solid #fecaca;
		color: #dc2626;
	}

	.mock-notice {
		margin-top: 2rem;
		padding: 1rem;
		background: #fef3c7;
		border: 1px solid #fde68a;
		border-radius: 4px;
		font-size: 0.875rem;
		color: #92400e;
	}
</style>
