<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import type { AiInsight } from '$lib/api/types';
	import { onMount } from 'svelte';
	import { createEventDispatcher } from 'svelte';
	import { toasts } from '$lib/stores/toast';
	import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';

	export let entityType: string | undefined = undefined;
	export let entityId: string | undefined = undefined;

	const dispatch = createEventDispatcher<{ applied: AiInsight }>();

	let insights: AiInsight[] = [];
	let loading = true;
	let error: string | null = null;
	let applyingInsightId: string | null = null;

	onMount(async () => {
		await loadInsights();
	});

	async function loadInsights() {
		try {
			loading = true;
			error = null;
			insights = await api.getInsights(entityType, entityId);
		} catch (e) {
			if (e instanceof ApiError) {
				error = e.getUserMessage();
			} else {
				error = e instanceof Error ? e.message : 'Failed to load AI suggestions';
			}
			console.error('Failed to load insights:', e);
		} finally {
			loading = false;
		}
	}

	function getInsightIcon(type: string): string {
		switch (type) {
			case 'TagSuggestion':
				return '🏷';
			case 'ChannelRecommendation':
				return '📢';
			case 'NextAction':
				return '✅';
			case 'RelationshipStrength':
				return '🤝';
			case 'ContentSummary':
				return '📝';
			default:
				return '💡';
		}
	}

	function formatInsightType(type: string): string {
		return type.replace(/([A-Z])/g, ' $1').trim();
	}

	async function handleApply(insight: AiInsight) {
		try {
			applyingInsightId = insight.id;
			await api.applyInsight(insight.id);
			insight.applied = true;
			insight.applied_at = new Date().toISOString();
			insights = insights;
			dispatch('applied', insight);
			toasts.success('AI suggestion applied successfully', 'Success');
		} catch (e) {
			if (e instanceof ApiError) {
				toasts.error(e.getUserMessage(), 'Failed to Apply Suggestion');
			} else {
				toasts.error('Failed to apply suggestion. Please try again.', 'Error');
			}
			console.error('Failed to apply insight:', e);
		} finally {
			applyingInsightId = null;
		}
	}

	async function handleFeedback(insight: AiInsight, helpful: boolean) {
		try {
			await api.feedbackInsight(insight.id, helpful);
			insight.feedback = {
				helpful,
				submitted_at: new Date().toISOString()
			};
			insights = insights;
			toasts.success('Thank you for your feedback!', 'Feedback Received', { duration: 3000 });
		} catch (e) {
			if (e instanceof ApiError) {
				toasts.error(e.getUserMessage(), 'Failed to Submit Feedback');
			} else {
				toasts.error('Failed to submit feedback. Please try again.', 'Error');
			}
			console.error('Failed to submit feedback:', e);
		}
	}

	function getConfidenceColor(confidence: number): string {
		if (confidence >= 0.8) return '#4caf50';
		if (confidence >= 0.6) return '#ff9800';
		return '#757575';
	}
</script>

<div class="ai-suggestions">
	<div class="header">
		<h3>💡 AI Suggestions</h3>
		{#if insights.length > 0}
			<span class="count">{insights.length}</span>
		{/if}
	</div>

	{#if loading}
		<div class="loading-container">
			<LoadingSpinner size="sm" message="Generating AI insights..." inline={true} />
		</div>
	{:else if error}
		<div class="error-container">
			<div class="error-icon">⚠</div>
			<div class="error-content">
				<strong>Unable to load suggestions</strong>
				<p>{error}</p>
			</div>
			<button class="retry-button" on:click={loadInsights}>
				Retry
			</button>
		</div>
	{:else if insights.length === 0}
		<div class="empty-state">
			<div class="empty-icon">💡</div>
			<p>No AI suggestions at the moment</p>
			<small>Check back later for personalized insights</small>
		</div>
	{:else}
		<div class="suggestions-list">
			{#each insights as insight (insight.id)}
				<div class="suggestion-card" class:applied={insight.applied}>
					<div class="suggestion-header">
						<div class="suggestion-title">
							<span class="insight-icon">{getInsightIcon(insight.insight_type)}</span>
							<span class="insight-type">{formatInsightType(insight.insight_type)}</span>
						</div>
						<div
							class="confidence-badge"
							style="background-color: {getConfidenceColor(insight.confidence)}"
						>
							{Math.round(insight.confidence * 100)}%
						</div>
					</div>

					<div class="suggestion-content">
						{insight.content}
					</div>

					<div class="suggestion-actions">
						{#if !insight.applied}
							<button
								class="apply-button"
								on:click={() => handleApply(insight)}
								disabled={applyingInsightId === insight.id}
							>
								{#if applyingInsightId === insight.id}
									<span class="button-spinner"></span>
									Applying...
								{:else}
									Apply
								{/if}
							</button>
						{:else}
							<span class="applied-badge">✓ Applied</span>
						{/if}

						{#if !insight.feedback}
							<div class="feedback-buttons">
								<button
									class="feedback-button"
									on:click={() => handleFeedback(insight, true)}
									title="Helpful"
								>
									👍
								</button>
								<button
									class="feedback-button"
									on:click={() => handleFeedback(insight, false)}
									title="Not helpful"
								>
									👎
								</button>
							</div>
						{:else}
							<span class="feedback-given">
								{insight.feedback.helpful ? '👍' : '👎'} Feedback given
							</span>
						{/if}
					</div>

					{#if insight.applied_at}
						<div class="applied-timestamp">
							Applied {new Date(insight.applied_at).toLocaleDateString()}
						</div>
					{/if}

					{#if insight.response_cached}
						<div class="cached-badge" title="This suggestion was generated from cached data">
							⚡ Instant
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.ai-suggestions {
		background: white;
		border: 1px solid #e0e0e0;
		border-radius: 8px;
		padding: 1rem;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
	}

	.count {
		padding: 0.25rem 0.5rem;
		background: #e3f2fd;
		color: #1976d2;
		border-radius: 12px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.loading-container {
		display: flex;
		justify-content: center;
		align-items: center;
		padding: 2rem;
	}

	.error-container {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
		background: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 6px;
	}

	.error-icon {
		font-size: 1.5rem;
		color: #ef4444;
		flex-shrink: 0;
	}

	.error-content {
		flex: 1;
	}

	.error-content strong {
		display: block;
		color: #991b1b;
		margin-bottom: 0.25rem;
		font-size: 0.875rem;
	}

	.error-content p {
		margin: 0;
		color: #dc2626;
		font-size: 0.8125rem;
		line-height: 1.4;
	}

	.retry-button {
		padding: 0.5rem 1rem;
		background: white;
		border: 1px solid #dc2626;
		color: #dc2626;
		border-radius: 4px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
		flex-shrink: 0;
	}

	.retry-button:hover {
		background: #fef2f2;
	}

	.empty-state {
		text-align: center;
		padding: 3rem 2rem;
		color: #757575;
	}

	.empty-icon {
		font-size: 2.5rem;
		margin-bottom: 1rem;
		opacity: 0.5;
	}

	.empty-state p {
		margin: 0.5rem 0;
		font-size: 0.875rem;
		color: #424242;
	}

	.empty-state small {
		font-size: 0.75rem;
		color: #757575;
	}

	.suggestions-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.suggestion-card {
		position: relative;
		padding: 1rem;
		background: #f9f9f9;
		border: 1px solid #e0e0e0;
		border-radius: 8px;
		transition: all 0.2s;
	}

	.suggestion-card.applied {
		background: #e8f5e9;
		border-color: #4caf50;
	}

	.suggestion-card:hover {
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.suggestion-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.75rem;
	}

	.suggestion-title {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.insight-icon {
		font-size: 1.25rem;
	}

	.insight-type {
		font-weight: 600;
		font-size: 0.875rem;
		color: #424242;
	}

	.confidence-badge {
		padding: 0.25rem 0.5rem;
		color: white;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.suggestion-content {
		margin-bottom: 1rem;
		line-height: 1.5;
		color: #424242;
	}

	.suggestion-actions {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.apply-button {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: #0066cc;
		color: white;
		border: none;
		border-radius: 4px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.2s;
		font-size: 0.875rem;
	}

	.apply-button:hover:not(:disabled) {
		background: #0052a3;
	}

	.apply-button:disabled {
		background: #9ca3af;
		cursor: not-allowed;
	}

	.button-spinner {
		display: inline-block;
		width: 14px;
		height: 14px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-top-color: white;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.applied-badge {
		color: #4caf50;
		font-weight: 600;
		font-size: 0.875rem;
	}

	.feedback-buttons {
		display: flex;
		gap: 0.5rem;
		margin-left: auto;
	}

	.feedback-button {
		padding: 0.375rem 0.75rem;
		background: white;
		border: 1px solid #e0e0e0;
		border-radius: 4px;
		font-size: 1rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.feedback-button:hover {
		background: #f5f5f5;
		border-color: #bdbdbd;
	}

	.feedback-given {
		margin-left: auto;
		font-size: 0.875rem;
		color: #757575;
	}

	.applied-timestamp {
		margin-top: 0.5rem;
		font-size: 0.75rem;
		color: #757575;
		font-style: italic;
	}

	.cached-badge {
		position: absolute;
		top: 0.5rem;
		right: 0.5rem;
		padding: 0.25rem 0.5rem;
		background: #fff3e0;
		color: #f57c00;
		border-radius: 4px;
		font-size: 0.625rem;
		font-weight: 600;
		cursor: help;
	}
</style>
