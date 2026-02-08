<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';
	import { hashColor } from '$lib/utils/colors';

	export let communicationType: string = 'email';
	export let communicationId: string = '';
	export let showSuggested: boolean = true;

	const dispatch = createEventDispatcher();

	type LabelEntry = {
		id: string;
		concept_id: string;
		concept_name: string;
		status: string;
		confidence: number | null | undefined;
		detection_method: string;
	};

	let labels: LabelEntry[] = [];
	let loading = true;

	$: if (communicationId) {
		loadLabels();
	}

	async function loadLabels() {
		if (!communicationId) {
			labels = [];
			return;
		}
		loading = true;
		try {
			const concepts = await api.getCommunicationConcepts(communicationType, communicationId);
			// Get concept names
			const allConcepts = await api.getConcepts();
			const conceptMap = new Map(allConcepts.map(c => [c.id, c.name]));

			labels = concepts.map(cc => ({
				id: cc.id,
				concept_id: cc.concept_id,
				concept_name: conceptMap.get(cc.concept_id) || 'Unknown',
				status: cc.status.toString().toLowerCase(),
				confidence: cc.confidence ?? null,
				detection_method: cc.detection_method.toString().toLowerCase()
			}));
		} catch {
			labels = [];
		} finally {
			loading = false;
		}
	}

	async function confirm(label: LabelEntry) {
		try {
			await api.updateConceptSuggestion(label.id, 'confirmed');
			label.status = 'confirmed';
			labels = labels;
			dispatch('statusChanged');
			toasts.success(`"${label.concept_name}" confirmed`);
		} catch (e: unknown) {
			toasts.error(e instanceof Error ? e.message : 'Failed to confirm');
		}
	}

	async function deny(label: LabelEntry) {
		try {
			await api.updateConceptSuggestion(label.id, 'denied');
			labels = labels.filter(l => l.id !== label.id);
			dispatch('statusChanged');
		} catch (e: unknown) {
			toasts.error(e instanceof Error ? e.message : 'Failed to deny');
		}
	}

	function handleClick(label: LabelEntry) {
		dispatch('labelClick', { conceptId: label.concept_id });
	}

	$: confirmedLabels = labels.filter(l => l.status === 'confirmed');
	$: suggestedLabels = labels.filter(l => l.status === 'suggested');
</script>

{#if !loading && labels.length > 0}
	<div class="label-badges">
		{#each confirmedLabels as label}
			<button
				class="badge confirmed"
				style="background-color: {hashColor(label.concept_name)}20; border-color: {hashColor(label.concept_name)}; color: {hashColor(label.concept_name)}"
				on:click={() => handleClick(label)}
				title="Confirmed label - click to view"
			>
				{label.concept_name}
			</button>
		{/each}
		{#if showSuggested}
			{#each suggestedLabels as label}
				<span class="badge suggested" style="border-color: {hashColor(label.concept_name)}80; color: {hashColor(label.concept_name)}">
					<button class="badge-text" on:click={() => handleClick(label)} title="Suggested ({(label.confidence ? (label.confidence * 100).toFixed(0) : '?')}% confidence)">
						{label.concept_name}
					</button>
					<button class="badge-action confirm" on:click={() => confirm(label)} title="Confirm">
						&#10003;
					</button>
					<button class="badge-action deny" on:click={() => deny(label)} title="Deny">
						&#10005;
					</button>
				</span>
			{/each}
		{/if}
	</div>
{/if}

<style>
	.label-badges {
		display: flex;
		flex-wrap: wrap;
		gap: 0.3rem;
		margin: 0.25rem 0;
	}

	.badge {
		display: inline-flex;
		align-items: center;
		padding: 0.15rem 0.5rem;
		border-radius: 1rem;
		font-size: 0.75rem;
		font-weight: 500;
		border: 1.5px solid;
		cursor: pointer;
		transition: all 0.15s;
	}

	.badge.confirmed {
		border-style: solid;
	}

	.badge.confirmed:hover {
		filter: brightness(0.95);
	}

	.badge.suggested {
		background: white;
		border-style: dashed;
		padding: 0;
		cursor: default;
		gap: 0;
	}

	.badge-text {
		border: none;
		background: none;
		color: inherit;
		cursor: pointer;
		padding: 0.15rem 0.3rem 0.15rem 0.5rem;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.badge-action {
		border: none;
		background: none;
		cursor: pointer;
		padding: 0.15rem 0.3rem;
		font-size: 0.7rem;
		opacity: 0.6;
		transition: opacity 0.15s;
	}

	.badge-action:hover {
		opacity: 1;
	}

	.badge-action.confirm {
		color: #16a34a;
	}

	.badge-action.deny {
		color: #dc2626;
	}
</style>
