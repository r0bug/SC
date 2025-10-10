<script lang="ts">
	export let progress: number = 0; // 0-100
	export let label: string = '';
	export let showPercentage: boolean = true;
	export let variant: 'default' | 'success' | 'warning' | 'error' = 'default';
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let estimatedTimeRemaining: string = '';

	$: clampedProgress = Math.min(Math.max(progress, 0), 100);

	const variantColors = {
		default: '#0066cc',
		success: '#10b981',
		warning: '#f59e0b',
		error: '#ef4444'
	};

	const heights = {
		sm: '6px',
		md: '10px',
		lg: '14px'
	};
</script>

<div class="progress-bar-container">
	{#if label || showPercentage}
		<div class="progress-header">
			{#if label}
				<span class="progress-label">{label}</span>
			{/if}
			<div class="progress-info">
				{#if showPercentage}
					<span class="progress-percentage">{Math.round(clampedProgress)}%</span>
				{/if}
				{#if estimatedTimeRemaining}
					<span class="time-remaining">{estimatedTimeRemaining} remaining</span>
				{/if}
			</div>
		</div>
	{/if}
	<div class="progress-bar-track" style="height: {heights[size]};">
		<div
			class="progress-bar-fill"
			style="width: {clampedProgress}%; background-color: {variantColors[variant]};"
		></div>
	</div>
</div>

<style>
	.progress-bar-container {
		width: 100%;
	}

	.progress-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
		gap: 1rem;
	}

	.progress-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--gray-700, #374151);
	}

	.progress-info {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	.progress-percentage {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--gray-900, #111827);
		min-width: 3ch;
		text-align: right;
	}

	.time-remaining {
		font-size: 0.75rem;
		color: var(--gray-600, #6b7280);
		white-space: nowrap;
	}

	.progress-bar-track {
		width: 100%;
		background-color: var(--gray-200, #e5e7eb);
		border-radius: 9999px;
		overflow: hidden;
	}

	.progress-bar-fill {
		height: 100%;
		transition: width 0.3s ease;
		border-radius: 9999px;
	}
</style>
