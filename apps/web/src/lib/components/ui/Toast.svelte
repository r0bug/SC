<script lang="ts">
	import { toasts, type Toast } from '$lib/stores/toast';
	import { fly, fade } from 'svelte/transition';
	import { flip } from 'svelte/animate';

	$: toastList = $toasts;

	function getIcon(type: Toast['type']): string {
		switch (type) {
			case 'success':
				return '✓';
			case 'error':
				return '✕';
			case 'warning':
				return '⚠';
			case 'info':
				return 'ℹ';
		}
	}

	function getColor(type: Toast['type']): string {
		switch (type) {
			case 'success':
				return 'var(--success, #10b981)';
			case 'error':
				return 'var(--error, #ef4444)';
			case 'warning':
				return 'var(--warning, #f59e0b)';
			case 'info':
				return 'var(--info, #3b82f6)';
		}
	}
</script>

<div class="toast-container">
	{#each toastList as toast (toast.id)}
		<div
			class="toast toast-{toast.type}"
			style="border-left-color: {getColor(toast.type)};"
			in:fly={{ y: -20, duration: 300 }}
			out:fade={{ duration: 200 }}
			animate:flip={{ duration: 200 }}
		>
			<div class="toast-icon" style="color: {getColor(toast.type)};">
				{getIcon(toast.type)}
			</div>
			<div class="toast-content">
				{#if toast.title}
					<div class="toast-title">{toast.title}</div>
				{/if}
				<div class="toast-message">{toast.message}</div>
			</div>
			{#if toast.action}
				<button class="toast-action" on:click={toast.action.callback}>
					{toast.action.label}
				</button>
			{/if}
			<button class="toast-close" on:click={() => toasts.remove(toast.id)}>
				×
			</button>
		</div>
	{/each}
</div>

<style>
	.toast-container {
		position: fixed;
		top: 1rem;
		right: 1rem;
		z-index: 9999;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		max-width: 400px;
		pointer-events: none;
	}

	.toast {
		pointer-events: auto;
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 1rem;
		background: white;
		border-radius: 8px;
		border-left: 4px solid;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		min-width: 300px;
	}

	.toast-icon {
		font-size: 1.25rem;
		font-weight: bold;
		flex-shrink: 0;
		width: 1.5rem;
		height: 1.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.toast-content {
		flex: 1;
		min-width: 0;
	}

	.toast-title {
		font-weight: 600;
		font-size: 0.875rem;
		color: var(--gray-900, #111827);
		margin-bottom: 0.25rem;
	}

	.toast-message {
		font-size: 0.875rem;
		color: var(--gray-700, #374151);
		line-height: 1.4;
	}

	.toast-action {
		padding: 0.25rem 0.75rem;
		background: transparent;
		border: 1px solid var(--gray-300, #d1d5db);
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
		flex-shrink: 0;
	}

	.toast-action:hover {
		background: var(--gray-50, #f9fafb);
	}

	.toast-close {
		background: transparent;
		border: none;
		font-size: 1.5rem;
		line-height: 1;
		color: var(--gray-400, #9ca3af);
		cursor: pointer;
		padding: 0;
		width: 1.5rem;
		height: 1.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		transition: color 0.2s;
	}

	.toast-close:hover {
		color: var(--gray-600, #6b7280);
	}

	@media (max-width: 640px) {
		.toast-container {
			left: 1rem;
			right: 1rem;
			max-width: none;
		}

		.toast {
			min-width: 0;
		}
	}
</style>
