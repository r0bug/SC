<script lang="ts">
	export let items: Array<{ label: string; href?: string }> = [];
</script>

<nav aria-label="Breadcrumb" class="breadcrumb">
	<ol class="breadcrumb-list">
		{#each items as item, index}
			<li class="breadcrumb-item">
				{#if item.href && index < items.length - 1}
					<a href={item.href} class="breadcrumb-link">
						{item.label}
					</a>
				{:else}
					<span class="breadcrumb-current" aria-current="page">
						{item.label}
					</span>
				{/if}
				{#if index < items.length - 1}
					<span class="breadcrumb-separator" aria-hidden="true">/</span>
				{/if}
			</li>
		{/each}
	</ol>
</nav>

<style>
	.breadcrumb {
		margin-bottom: var(--space-4);
	}

	.breadcrumb-list {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		list-style: none;
		padding: 0;
		margin: 0;
		font-size: var(--text-sm);
	}

	.breadcrumb-item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.breadcrumb-link {
		color: var(--color-neutral-600);
		text-decoration: none;
		transition: color var(--transition-fast);
	}

	.breadcrumb-link:hover {
		color: var(--color-primary-600);
		text-decoration: underline;
	}

	.breadcrumb-current {
		color: var(--color-neutral-900);
		font-weight: var(--font-medium);
	}

	.breadcrumb-separator {
		color: var(--color-neutral-400);
		user-select: none;
	}
</style>
