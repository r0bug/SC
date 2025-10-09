<script lang="ts">
	import { getPageNumbers } from '$lib/utils/pagination';
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher<{ pageChange: number }>();

	export let currentPage: number = 1;
	export let totalPages: number = 1;
	export let maxVisible: number = 7;

	$: pageNumbers = getPageNumbers(currentPage, totalPages, maxVisible);
	$: showFirst = pageNumbers[0] > 1;
	$: showLast = pageNumbers[pageNumbers.length - 1] < totalPages;

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages && page !== currentPage) {
			dispatch('pageChange', page);
		}
	}
</script>

<div class="pagination" role="navigation" aria-label="Pagination">
	<button
		class="pagination-btn"
		disabled={currentPage === 1}
		on:click={() => goToPage(currentPage - 1)}
		aria-label="Previous page"
	>
		←
	</button>

	{#if showFirst}
		<button class="pagination-btn" on:click={() => goToPage(1)}>
			1
		</button>
		<span class="pagination-ellipsis">…</span>
	{/if}

	{#each pageNumbers as page (page)}
		<button
			class="pagination-btn"
			class:active={page === currentPage}
			on:click={() => goToPage(page)}
			aria-label={`Page ${page}`}
			aria-current={page === currentPage ? 'page' : undefined}
		>
			{page}
		</button>
	{/each}

	{#if showLast}
		<span class="pagination-ellipsis">…</span>
		<button class="pagination-btn" on:click={() => goToPage(totalPages)}>
			{totalPages}
		</button>
	{/if}

	<button
		class="pagination-btn"
		disabled={currentPage === totalPages}
		on:click={() => goToPage(currentPage + 1)}
		aria-label="Next page"
	>
		→
	</button>
</div>

<style>
	.pagination {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		justify-content: center;
		padding: 1rem 0;
	}

	.pagination-btn {
		min-width: 2.5rem;
		height: 2.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid var(--gray-300, #d1d5db);
		background: white;
		color: var(--gray-700, #374151);
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.875rem;
		font-weight: 500;
		transition: all 0.2s;
	}

	.pagination-btn:hover:not(:disabled):not(.active) {
		background: var(--gray-50, #f9fafb);
		border-color: var(--gray-400, #9ca3af);
	}

	.pagination-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.pagination-btn.active {
		background: var(--primary, #3b82f6);
		color: white;
		border-color: var(--primary, #3b82f6);
	}

	.pagination-ellipsis {
		color: var(--gray-500, #6b7280);
		padding: 0 0.25rem;
	}

	@media (max-width: 640px) {
		.pagination {
			gap: 0.25rem;
		}

		.pagination-btn {
			min-width: 2rem;
			height: 2rem;
			font-size: 0.8125rem;
		}
	}
</style>
