<script lang="ts">
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let message: string = '';
	export let inline: boolean = false;

	const sizeMap = {
		sm: '1.5rem',
		md: '2.5rem',
		lg: '4rem'
	};
</script>

<div class="spinner-container" class:inline aria-live="polite" aria-busy="true">
	<div class="spinner" style="width: {sizeMap[size]}; height: {sizeMap[size]}" role="status">
		<span class="sr-only">Loading...</span>
	</div>
	{#if message}
		<p class="spinner-message">{message}</p>
	{/if}
</div>

<style>
	.spinner-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		padding: 2rem;
	}

	.spinner-container.inline {
		display: inline-flex;
		padding: 0;
	}

	.spinner {
		border: 3px solid var(--gray-200, #e5e7eb);
		border-top-color: var(--primary, #3b82f6);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.spinner-message {
		color: var(--gray-600, #4b5563);
		font-size: 0.875rem;
		margin: 0;
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}
</style>
