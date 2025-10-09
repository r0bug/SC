<script lang="ts">
	import { toast, type Toast } from '$lib/stores/toast';
	import { flip } from 'svelte/animate';
	import { fly } from 'svelte/transition';

	let toasts: Toast[] = [];
	toast.subscribe(value => toasts = value);

	function getIcon(type: Toast['type']): string {
		switch (type) {
			case 'success': return '✓';
			case 'error': return '✕';
			case 'warning': return '⚠';
			case 'info': return 'ℹ';
			default: return 'ℹ';
		}
	}
</script>

<div class="toast-container" aria-live="polite" aria-atomic="true">
	{#each toasts as t (t.id)}
		<div
			class="toast toast-{t.type}"
			animate:flip={{ duration: 300 }}
			in:fly={{ x: 300, duration: 300 }}
			out:fly={{ x: 300, duration: 200 }}
			role="alert"
		>
			<span class="toast-icon">{getIcon(t.type)}</span>
			<span class="toast-message">{t.message}</span>
			<button
				class="toast-close"
				on:click={() => toast.dismiss(t.id)}
				aria-label="Dismiss notification"
			>
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
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 1rem 1.25rem;
		border-radius: 8px;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1), 0 2px 4px rgba(0, 0, 0, 0.06);
		pointer-events: auto;
		min-width: 300px;
		max-width: 400px;
	}

	.toast-success {
		background: #10b981;
		color: white;
	}

	.toast-error {
		background: #ef4444;
		color: white;
	}

	.toast-warning {
		background: #f59e0b;
		color: white;
	}

	.toast-info {
		background: #3b82f6;
		color: white;
	}

	.toast-icon {
		font-size: 1.25rem;
		font-weight: bold;
		flex-shrink: 0;
	}

	.toast-message {
		flex: 1;
		font-size: 0.9375rem;
		line-height: 1.5;
	}

	.toast-close {
		background: none;
		border: none;
		color: white;
		font-size: 1.5rem;
		line-height: 1;
		cursor: pointer;
		padding: 0;
		width: 1.5rem;
		height: 1.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		transition: background 0.2s;
		flex-shrink: 0;
	}

	.toast-close:hover {
		background: rgba(0, 0, 0, 0.2);
	}

	@media (max-width: 640px) {
		.toast-container {
			top: auto;
			bottom: 1rem;
			right: 1rem;
			left: 1rem;
			max-width: none;
		}

		.toast {
			min-width: auto;
			max-width: none;
		}
	}
</style>
