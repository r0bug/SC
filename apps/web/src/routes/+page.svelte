<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isAuthenticated } from '$lib/stores/auth';

	onMount(() => {
		// Redirect based on authentication status
		if ($isAuthenticated) {
			goto('/dashboard');
		} else {
			goto('/auth/login');
		}
	});
</script>

<!-- Loading state while redirecting -->
<div class="loading-container">
	<div class="spinner"></div>
	<p>Loading...</p>
</div>

<style>
	.loading-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 100vh;
		gap: var(--space-4);
	}

	.spinner {
		width: 40px;
		height: 40px;
		border: 4px solid var(--color-neutral-200);
		border-top-color: var(--color-primary-600);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	p {
		color: var(--color-neutral-600);
		font-size: var(--text-sm);
	}
</style>
