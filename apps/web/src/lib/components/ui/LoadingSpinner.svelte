<script lang="ts">
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let message: string = '';
	export let inline: boolean = false;

	const sizes = {
		sm: '16px',
		md: '32px',
		lg: '48px'
	};
</script>

<div class="loading-spinner" class:inline>
	<div class="spinner" style="width: {sizes[size]}; height: {sizes[size]};">
		<svg viewBox="0 0 50 50" class="spinner-svg">
			<circle
				class="path"
				cx="25"
				cy="25"
				r="20"
				fill="none"
				stroke-width="4"
			/>
		</svg>
	</div>
	{#if message}
		<p class="message">{message}</p>
	{/if}
</div>

<style>
	.loading-spinner {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		padding: 2rem;
	}

	.loading-spinner.inline {
		flex-direction: row;
		padding: 0.5rem;
	}

	.spinner {
		animation: rotate 2s linear infinite;
	}

	.spinner-svg {
		width: 100%;
		height: 100%;
	}

	.path {
		stroke: var(--primary, #0066cc);
		stroke-linecap: round;
		animation: dash 1.5s ease-in-out infinite;
	}

	@keyframes rotate {
		100% {
			transform: rotate(360deg);
		}
	}

	@keyframes dash {
		0% {
			stroke-dasharray: 1, 150;
			stroke-dashoffset: 0;
		}
		50% {
			stroke-dasharray: 90, 150;
			stroke-dashoffset: -35;
		}
		100% {
			stroke-dasharray: 90, 150;
			stroke-dashoffset: -124;
		}
	}

	.message {
		margin: 0;
		color: var(--gray-700, #374151);
		font-size: 0.875rem;
		text-align: center;
	}
</style>
