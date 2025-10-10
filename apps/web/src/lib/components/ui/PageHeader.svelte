<script lang="ts">
	import Breadcrumb from './Breadcrumb.svelte';

	export let title: string;
	export let description: string | undefined = undefined;
	export let breadcrumbs: Array<{ label: string; href?: string }> | undefined = undefined;
</script>

<div class="page-header">
	{#if breadcrumbs}
		<Breadcrumb items={breadcrumbs} />
	{/if}

	<div class="page-header-content">
		<div class="page-header-text">
			<h1 class="page-title">{title}</h1>
			{#if description}
				<p class="page-description">{description}</p>
			{/if}
		</div>

		{#if $$slots.actions}
			<div class="page-header-actions">
				<slot name="actions" />
			</div>
		{/if}
	</div>
</div>

<style>
	.page-header {
		margin-bottom: var(--space-8);
	}

	.page-header-content {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-6);
		flex-wrap: wrap;
	}

	.page-header-text {
		flex: 1;
		min-width: 0;
	}

	.page-title {
		font-size: var(--text-3xl);
		font-weight: var(--font-bold);
		color: var(--color-neutral-900);
		margin: 0 0 var(--space-2) 0;
		line-height: var(--leading-tight);
	}

	.page-description {
		font-size: var(--text-base);
		color: var(--color-neutral-600);
		margin: 0;
		line-height: var(--leading-relaxed);
		max-width: 65ch;
	}

	.page-header-actions {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		flex-shrink: 0;
	}

	@media (max-width: 768px) {
		.page-title {
			font-size: var(--text-2xl);
		}

		.page-header-content {
			flex-direction: column;
			align-items: stretch;
		}

		.page-header-actions {
			width: 100%;
			justify-content: stretch;
		}
	}
</style>
