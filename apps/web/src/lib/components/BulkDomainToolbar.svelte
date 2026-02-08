<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import DomainTagInput from './DomainTagInput.svelte';

	export let selectedEmailIds: string[] = [];

	const dispatch = createEventDispatcher();

	function handleClear() {
		dispatch('clear');
	}

	function handleDomainsChanged() {
		dispatch('assigned');
	}
</script>

{#if selectedEmailIds.length > 0}
	<div class="bulk-toolbar" role="toolbar" aria-label="Bulk domain assignment">
		<div class="toolbar-inner">
			<span class="selection-count">{selectedEmailIds.length} email{selectedEmailIds.length === 1 ? '' : 's'} selected</span>
			<div class="toolbar-input">
				<DomainTagInput
					emailIds={selectedEmailIds}
					currentDomains={[]}
					compact={false}
					on:domainsChanged={handleDomainsChanged}
				/>
			</div>
			<button class="btn btn-ghost btn-sm" on:click={handleClear}>Clear</button>
		</div>
	</div>
{/if}

<style>
	.bulk-toolbar {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		z-index: var(--z-fixed, 1030);
		background: var(--white, #fff);
		border-top: 2px solid var(--primary, #6366f1);
		box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.1);
		padding: 0.75rem 1rem;
		animation: slideUp 0.2s ease-out;
	}

	@keyframes slideUp {
		from {
			transform: translateY(100%);
		}
		to {
			transform: translateY(0);
		}
	}

	.toolbar-inner {
		display: flex;
		align-items: center;
		gap: 1rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	.selection-count {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--primary-700, #4338ca);
		white-space: nowrap;
	}

	.toolbar-input {
		flex: 1;
		min-width: 200px;
	}
</style>
