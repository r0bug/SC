<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Contact, Pick, MatchResult, EntityRelationship } from '$lib/api/types';

	let contacts: Contact[] = [];
	let picks: Pick[] = [];
	let loading = true;
	let selectedTab: 'contacts' | 'picks' = 'contacts';

	let selectedContactId = '';
	let selectedPickId = '';
	let contactMatches: MatchResult[] = [];
	let pickMatches: MatchResult[] = [];
	let matchLoading = false;

	onMount(async () => {
		try {
			[contacts, picks] = await Promise.all([
				api.getContacts(),
				api.getPicks()
			]);
		} catch (error) {
			console.error('Failed to load data:', error);
		} finally {
			loading = false;
		}
	});

	async function searchContactMatches() {
		if (!selectedContactId) return;
		matchLoading = true;
		try {
			contactMatches = await api.getContactMatches(selectedContactId);
		} catch (error) {
			console.error('Failed to get matches:', error);
			contactMatches = [];
		} finally {
			matchLoading = false;
		}
	}

	async function searchPickMatches() {
		if (!selectedPickId) return;
		matchLoading = true;
		try {
			pickMatches = await api.getPickMatches(selectedPickId);
		} catch (error) {
			console.error('Failed to get matches:', error);
			pickMatches = [];
		} finally {
			matchLoading = false;
		}
	}

	function getContactName(id: string): string {
		const c = contacts.find(c => c.id === id);
		return c ? `${c.first_name} ${c.last_name || ''}`.trim() : id.substring(0, 8);
	}

	function getPickName(id: string): string {
		const p = picks.find(p => p.id === id);
		return p ? p.name : id.substring(0, 8);
	}

	function getEntityLabel(rel: EntityRelationship): string {
		if (rel.target_type === 'Contact') return getContactName(rel.target_id);
		if (rel.target_type === 'Pick') return getPickName(rel.target_id);
		return `${rel.target_type} ${rel.target_id.substring(0, 8)}`;
	}
</script>

<div class="page">
	<div class="header">
		<h1>Match Dashboard</h1>
		<p class="subtitle">Find want/have matches across contacts and picks</p>
	</div>

	{#if loading}
		<p class="loading">Loading...</p>
	{:else}
		<div class="tabs">
			<button class="tab" class:active={selectedTab === 'contacts'} on:click={() => selectedTab = 'contacts'}>
				Contact Matches
			</button>
			<button class="tab" class:active={selectedTab === 'picks'} on:click={() => selectedTab = 'picks'}>
				Pick Matches
			</button>
		</div>

		{#if selectedTab === 'contacts'}
			<div class="search-bar">
				<select bind:value={selectedContactId} on:change={searchContactMatches}>
					<option value="">Select a contact...</option>
					{#each contacts as contact}
						<option value={contact.id}>{contact.first_name} {contact.last_name || ''}</option>
					{/each}
				</select>
			</div>

			{#if matchLoading}
				<p class="loading">Finding matches...</p>
			{:else if selectedContactId && contactMatches.length === 0}
				<div class="empty-state">
					<p>No matches found for this contact. Add WANT and HAVE relationships to find matches.</p>
				</div>
			{:else}
				<div class="match-list">
					{#each contactMatches as match}
						<div class="match-card">
							<div class="match-want">
								<span class="label">WANTS</span>
								<span class="entity">{match.want.target_type} {match.want.target_id.substring(0, 8)}</span>
								{#if match.want.relationship_type_name}
									<span class="rel-type">({match.want.relationship_type_name})</span>
								{/if}
							</div>
							<div class="match-haves">
								<span class="label">Matched by {match.haves.length} source(s):</span>
								{#each match.haves as have}
									<div class="have-item">
										<span class="have-source">{have.source_type}: {getEntityLabel(have)}</span>
										<span class="have-arrow">HAS</span>
									</div>
								{/each}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{:else}
			<div class="search-bar">
				<select bind:value={selectedPickId} on:change={searchPickMatches}>
					<option value="">Select a pick...</option>
					{#each picks as pick}
						<option value={pick.id}>{pick.name}</option>
					{/each}
				</select>
			</div>

			{#if matchLoading}
				<p class="loading">Finding matches...</p>
			{:else if selectedPickId && pickMatches.length === 0}
				<div class="empty-state">
					<p>No matches found for this pick. Link concepts to contacts and picks to find matches.</p>
				</div>
			{:else}
				<div class="match-list">
					{#each pickMatches as match}
						<div class="match-card">
							<div class="match-want">
								<span class="label">HAS</span>
								<span class="entity">{match.want.target_type} {match.want.target_id.substring(0, 8)}</span>
							</div>
							<div class="match-haves">
								<span class="label">Wanted by {match.haves.length} contact(s):</span>
								{#each match.haves as have}
									<div class="have-item">
										<span class="have-source">{getEntityLabel(have)}</span>
										<span class="have-arrow">WANTS</span>
									</div>
								{/each}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	{/if}
</div>

<style>
	.page { padding: 1.5rem; max-width: 1200px; }
	.header { margin-bottom: 1.5rem; }
	.header h1 { margin: 0; font-size: 1.5rem; }
	.subtitle { color: #6b7280; margin: 0.25rem 0 0; }
	.loading { color: #6b7280; }
	.empty-state { text-align: center; padding: 3rem; color: #6b7280; background: #f9fafb; border-radius: 8px; }
	.tabs { display: flex; gap: 0; border-bottom: 2px solid #e5e7eb; margin-bottom: 1.5rem; }
	.tab { padding: 0.75rem 1.5rem; background: none; border: none; border-bottom: 2px solid transparent; cursor: pointer; font-size: 0.9rem; color: #6b7280; margin-bottom: -2px; }
	.tab.active { color: #3b82f6; border-bottom-color: #3b82f6; font-weight: 500; }
	.tab:hover { color: #374151; }
	.search-bar { margin-bottom: 1.5rem; }
	.search-bar select { padding: 0.5rem 1rem; border: 1px solid #d1d5db; border-radius: 6px; font-size: 0.9rem; min-width: 300px; }
	.match-list { display: flex; flex-direction: column; gap: 1rem; }
	.match-card { background: white; border: 1px solid #e5e7eb; border-radius: 8px; padding: 1rem; }
	.match-want { margin-bottom: 0.75rem; display: flex; align-items: center; gap: 0.5rem; }
	.match-want .label { background: #dbeafe; color: #1d4ed8; padding: 2px 8px; border-radius: 4px; font-size: 0.75rem; font-weight: 600; }
	.match-want .entity { font-weight: 500; }
	.match-want .rel-type { color: #9ca3af; font-size: 0.8rem; }
	.match-haves .label { display: block; font-size: 0.8rem; color: #6b7280; margin-bottom: 0.5rem; }
	.have-item { display: flex; align-items: center; gap: 0.5rem; padding: 0.375rem 0.5rem; background: #f0fdf4; border-radius: 4px; margin-bottom: 0.25rem; }
	.have-source { font-size: 0.875rem; }
	.have-arrow { background: #dcfce7; color: #15803d; padding: 1px 6px; border-radius: 3px; font-size: 0.7rem; font-weight: 600; }
</style>
