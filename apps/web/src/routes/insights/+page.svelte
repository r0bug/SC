<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { AiInsight } from '$lib/api/types';

	let insights: AiInsight[] = [];
	let loading = true;

	onMount(async () => {
		await loadInsights();
	});

	async function loadInsights() {
		try {
			insights = await api.getInsights();
		} catch (error) {
			console.error('Failed to load insights:', error);
		} finally {
			loading = false;
		}
	}

	async function handleApply(id: string) {
		try {
			await api.applyInsight(id);
			insights = insights.map(i => i.id === id ? {...i, applied: true} : i);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleFeedback(id: string, helpful: boolean) {
		try {
			await api.feedbackInsight(id, helpful);
			insights = insights.map(i => i.id === id ? {...i, feedback: { helpful, submitted_at: new Date().toISOString() }} : i);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<h1>AI Insights</h1>

	{#if loading}
		<div>Loading...</div>
	{:else if insights.length === 0}
		<div class="empty">No insights available</div>
	{:else}
		<div class="insights-list">
			{#each insights as insight}
				<div class="insight-card card">
					<div class="insight-type">{insight.insight_type}</div>
					<p class="content">{insight.content}</p>
					<div class="confidence">Confidence: {(insight.confidence * 100).toFixed(0)}%</div>
					{#if !insight.applied && !insight.feedback}
						<div class="actions">
							<button on:click={() => handleApply(insight.id)} class="btn btn-sm btn-primary">Apply</button>
							<button on:click={() => handleFeedback(insight.id, true)} class="btn btn-sm">👍 Helpful</button>
							<button on:click={() => handleFeedback(insight.id, false)} class="btn btn-sm">👎 Not Helpful</button>
						</div>
					{:else if insight.applied}
						<div class="status">✅ Applied</div>
					{:else if insight.feedback}
						<div class="status">Feedback submitted</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1000px; }
	.insights-list { display: flex; flex-direction: column; gap: 1rem; }
	.insight-card { padding: 1.5rem; }
	.insight-type { font-size: 0.75rem; font-weight: 600; color: var(--primary); text-transform: uppercase; margin-bottom: 0.5rem; }
	.content { color: var(--gray-700); margin: 0.5rem 0; }
	.confidence { font-size: 0.875rem; color: var(--gray-600); margin: 0.5rem 0; }
	.actions { display: flex; gap: 0.5rem; margin-top: 1rem; }
	.status { font-size: 0.875rem; color: var(--gray-600); margin-top: 1rem; }
	.empty { text-align: center; padding: 3rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
</style>
