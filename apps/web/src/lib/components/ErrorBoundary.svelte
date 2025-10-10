<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	export let fallback: 'page' | 'component' = 'component';

	let hasError = false;
	let error: Error | null = null;
	let errorInfo: string = '';

	onMount(() => {
		const handleError = (event: ErrorEvent) => {
			hasError = true;
			error = event.error;
			errorInfo = event.message;
			console.error('ErrorBoundary caught:', event.error);
			event.preventDefault();
		};

		const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
			hasError = true;
			error = event.reason instanceof Error ? event.reason : new Error(String(event.reason));
			errorInfo = 'Unhandled promise rejection';
			console.error('ErrorBoundary caught promise rejection:', event.reason);
			event.preventDefault();
		};

		window.addEventListener('error', handleError);
		window.addEventListener('unhandledrejection', handleUnhandledRejection);

		return () => {
			window.removeEventListener('error', handleError);
			window.removeEventListener('unhandledrejection', handleUnhandledRejection);
		};
	});

	function handleReload() {
		hasError = false;
		error = null;
		errorInfo = '';
		window.location.reload();
	}

	function handleGoHome() {
		hasError = false;
		error = null;
		errorInfo = '';
		goto('/');
	}

	function handleDismiss() {
		hasError = false;
		error = null;
		errorInfo = '';
	}
</script>

{#if hasError}
	{#if fallback === 'page'}
		<div class="error-page">
			<div class="error-container">
				<div class="error-icon">⚠️</div>
				<h1>Something went wrong</h1>
				<p class="error-message">
					{error?.message || 'An unexpected error occurred'}
				</p>
				{#if errorInfo}
					<p class="error-info">{errorInfo}</p>
				{/if}
				<div class="error-actions">
					<button class="btn btn-primary" on:click={handleReload}>
						Reload Page
					</button>
					<button class="btn btn-secondary" on:click={handleGoHome}>
						Go to Home
					</button>
				</div>
				<details class="error-details">
					<summary>Technical Details</summary>
					<pre>{error?.stack || 'No stack trace available'}</pre>
				</details>
			</div>
		</div>
	{:else}
		<div class="error-component">
			<div class="error-banner">
				<span class="error-icon">⚠️</span>
				<div class="error-content">
					<strong>Error:</strong> {error?.message || 'Something went wrong'}
				</div>
				<button class="dismiss-btn" on:click={handleDismiss} aria-label="Dismiss error">
					×
				</button>
			</div>
		</div>
	{/if}
{:else}
	<slot />
{/if}

<style>
	.error-page {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 2rem;
		background: var(--gray-50, #f9fafb);
	}

	.error-container {
		max-width: 600px;
		width: 100%;
		background: white;
		border-radius: 12px;
		padding: 3rem;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
		text-align: center;
	}

	.error-icon {
		font-size: 4rem;
		margin-bottom: 1rem;
	}

	.error-container h1 {
		font-size: 2rem;
		margin-bottom: 1rem;
		color: var(--error, #dc2626);
	}

	.error-message {
		font-size: 1.125rem;
		color: var(--gray-700, #374151);
		margin-bottom: 0.5rem;
	}

	.error-info {
		font-size: 0.875rem;
		color: var(--gray-600, #4b5563);
		margin-bottom: 2rem;
	}

	.error-actions {
		display: flex;
		gap: 1rem;
		justify-content: center;
		margin-bottom: 2rem;
	}

	.error-details {
		text-align: left;
		margin-top: 2rem;
		padding: 1rem;
		background: var(--gray-100, #f3f4f6);
		border-radius: 8px;
	}

	.error-details summary {
		cursor: pointer;
		font-weight: 500;
		color: var(--gray-700, #374151);
		user-select: none;
	}

	.error-details summary:hover {
		color: var(--primary, #3b82f6);
	}

	.error-details pre {
		margin-top: 1rem;
		padding: 1rem;
		background: white;
		border: 1px solid var(--gray-300, #d1d5db);
		border-radius: 4px;
		overflow-x: auto;
		font-size: 0.75rem;
		line-height: 1.5;
		color: var(--gray-800, #1f2937);
	}

	.error-component {
		margin-bottom: 1rem;
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
		background: #fee2e2;
		border: 1px solid #fecaca;
		border-radius: 8px;
		color: #991b1b;
	}

	.error-banner .error-icon {
		font-size: 1.5rem;
	}

	.error-content {
		flex: 1;
	}

	.dismiss-btn {
		background: none;
		border: none;
		font-size: 1.5rem;
		line-height: 1;
		cursor: pointer;
		color: #991b1b;
		padding: 0;
		width: 2rem;
		height: 2rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		transition: background 0.2s;
	}

	.dismiss-btn:hover {
		background: rgba(0, 0, 0, 0.1);
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border-radius: 8px;
		border: none;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--primary, #3b82f6);
		color: white;
	}

	.btn-primary:hover {
		background: var(--primary-dark, #2563eb);
	}

	.btn-secondary {
		background: var(--gray-200, #e5e7eb);
		color: var(--gray-800, #1f2937);
	}

	.btn-secondary:hover {
		background: var(--gray-300, #d1d5db);
	}
</style>
