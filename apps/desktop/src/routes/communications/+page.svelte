<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { Communication } from '$lib/api/types';

	let communications: Communication[] = [];
	let filteredComms: Communication[] = [];
	let loading = true;
	let filterType: 'all' | 'Sms' | 'Call' | 'Email' = 'all';
	let filterDirection: 'all' | 'Inbound' | 'Outbound' | 'Missed' = 'all';
	let searchQuery = '';

	onMount(async () => {
		await loadCommunications();
	});

	async function loadCommunications() {
		try {
			loading = true;
			communications = await tauriApi.getAllCommunications(500, 0);
			applyFilters();
		} catch (error) {
			console.error('Failed to load communications:', error);
		} finally {
			loading = false;
		}
	}

	function applyFilters() {
		let filtered = communications;

		// Filter by type
		if (filterType !== 'all') {
			filtered = filtered.filter(c => c.communication_type === filterType);
		}

		// Filter by direction
		if (filterDirection !== 'all') {
			filtered = filtered.filter(c => c.direction === filterDirection);
		}

		// Filter by search query
		if (searchQuery.trim()) {
			const query = searchQuery.toLowerCase();
			filtered = filtered.filter(c =>
				(c.content && c.content.toLowerCase().includes(query)) ||
				(c.phone_number && c.phone_number.includes(query))
			);
		}

		filteredComms = filtered;
	}

	function handleFilterChange() {
		applyFilters();
	}

	function getTypeStats(type: 'Sms' | 'Call' | 'Email'): number {
		return communications.filter(c => c.communication_type === type).length;
	}

	function getDirectionStats(direction: 'Inbound' | 'Outbound' | 'Missed'): number {
		return communications.filter(c => c.direction === direction).length;
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Communications</h1>
			<p class="subtitle">SMS and call history</p>
		</div>
	</div>

	<div class="filters card">
		<input
			type="text"
			placeholder="Search messages or phone numbers..."
			bind:value={searchQuery}
			on:input={handleFilterChange}
			class="search-input"
		/>

		<div class="filter-row">
			<div class="filter-group">
				<label>Type</label>
				<div class="filter-buttons">
					<button
						class="filter-btn {filterType === 'all' ? 'active' : ''}"
						on:click={() => { filterType = 'all'; handleFilterChange(); }}
					>
						All ({communications.length})
					</button>
					<button
						class="filter-btn {filterType === 'Sms' ? 'active' : ''}"
						on:click={() => { filterType = 'Sms'; handleFilterChange(); }}
					>
						📱 SMS ({getTypeStats('Sms')})
					</button>
					<button
						class="filter-btn {filterType === 'Call' ? 'active' : ''}"
						on:click={() => { filterType = 'Call'; handleFilterChange(); }}
					>
						📞 Calls ({getTypeStats('Call')})
					</button>
				</div>
			</div>

			<div class="filter-group">
				<label>Direction</label>
				<div class="filter-buttons">
					<button
						class="filter-btn {filterDirection === 'all' ? 'active' : ''}"
						on:click={() => { filterDirection = 'all'; handleFilterChange(); }}
					>
						All
					</button>
					<button
						class="filter-btn {filterDirection === 'Inbound' ? 'active' : ''}"
						on:click={() => { filterDirection = 'Inbound'; handleFilterChange(); }}
					>
						⬇️ Inbound ({getDirectionStats('Inbound')})
					</button>
					<button
						class="filter-btn {filterDirection === 'Outbound' ? 'active' : ''}"
						on:click={() => { filterDirection = 'Outbound'; handleFilterChange(); }}
					>
						⬆️ Outbound ({getDirectionStats('Outbound')})
					</button>
					<button
						class="filter-btn {filterDirection === 'Missed' ? 'active' : ''}"
						on:click={() => { filterDirection = 'Missed'; handleFilterChange(); }}
					>
						❌ Missed ({getDirectionStats('Missed')})
					</button>
				</div>
			</div>
		</div>
	</div>

	{#if loading}
		<div class="loading">Loading communications...</div>
	{:else if filteredComms.length === 0}
		<div class="empty card">
			{#if searchQuery || filterType !== 'all' || filterDirection !== 'all'}
				<p>No communications match your filters.</p>
				<button on:click={() => { searchQuery = ''; filterType = 'all'; filterDirection = 'all'; handleFilterChange(); }} class="btn btn-primary">
					Clear Filters
				</button>
			{:else}
				<p>No communications found.</p>
			{/if}
		</div>
	{:else}
		<div class="communications-list">
			{#each filteredComms as comm}
				<div class="comm-card card {comm.direction.toLowerCase()}">
					<div class="comm-header">
						<span class="comm-type">
							{#if comm.communication_type === 'Sms'}📱 SMS
							{:else if comm.communication_type === 'Call'}📞 Call
							{:else if comm.communication_type === 'Email'}📧 Email
							{:else}{comm.communication_type}
							{/if}
						</span>
						<span class="comm-direction {comm.direction.toLowerCase()}">
							{#if comm.direction === 'Inbound'}⬇️ Inbound
							{:else if comm.direction === 'Outbound'}⬆️ Outbound
							{:else}❌ Missed
							{/if}
						</span>
						<span class="comm-date">{new Date(comm.timestamp).toLocaleString()}</span>
					</div>
					{#if comm.phone_number}
						<div class="comm-phone">📱 {comm.phone_number}</div>
					{/if}
					{#if comm.content}
						<p class="comm-content">{comm.content}</p>
					{/if}
					{#if comm.duration_seconds}
						<span class="comm-duration">Duration: {Math.floor(comm.duration_seconds / 60)}m {comm.duration_seconds % 60}s</span>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.header h1 {
		margin: 0 0 0.5rem;
		font-size: 2rem;
		color: var(--gray-900);
	}

	.subtitle {
		margin: 0;
		color: var(--gray-600);
		font-size: 1rem;
	}

	.filters {
		margin-bottom: 2rem;
		padding: 1.5rem;
	}

	.search-input {
		width: 100%;
		padding: 0.75rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
		margin-bottom: 1.5rem;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.filter-row {
		display: flex;
		gap: 2rem;
		flex-wrap: wrap;
	}

	.filter-group {
		flex: 1;
		min-width: 250px;
	}

	.filter-group label {
		display: block;
		margin-bottom: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--gray-700);
	}

	.filter-buttons {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}

	.filter-btn {
		padding: 0.5rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		background: white;
		cursor: pointer;
		font-size: 0.875rem;
		font-weight: 500;
		transition: all 0.2s;
	}

	.filter-btn:hover {
		background: var(--gray-50);
	}

	.filter-btn.active {
		background: var(--primary);
		color: white;
		border-color: var(--primary);
	}

	.loading {
		text-align: center;
		padding: 3rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.empty {
		text-align: center;
		padding: 3rem;
	}

	.empty p {
		margin: 0 0 1rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.communications-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.comm-card {
		padding: 1.5rem;
		border-left: 4px solid var(--gray-400);
		transition: transform 0.2s, box-shadow 0.2s;
	}

	.comm-card:hover {
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.comm-card.inbound {
		border-left-color: #3b82f6;
		background: linear-gradient(to right, #eff6ff 0%, white 50%);
	}

	.comm-card.outbound {
		border-left-color: #10b981;
		background: linear-gradient(to right, #ecfdf5 0%, white 50%);
	}

	.comm-card.missed {
		border-left-color: #ef4444;
		background: linear-gradient(to right, #fef2f2 0%, white 50%);
	}

	.comm-header {
		display: flex;
		gap: 1rem;
		align-items: center;
		margin-bottom: 0.75rem;
		flex-wrap: wrap;
	}

	.comm-type {
		font-weight: 600;
		font-size: 1rem;
		color: var(--gray-900);
	}

	.comm-direction {
		font-size: 0.75rem;
		padding: 0.25rem 0.75rem;
		border-radius: 999px;
		font-weight: 600;
	}

	.comm-direction.inbound {
		background: #dbeafe;
		color: #1e40af;
	}

	.comm-direction.outbound {
		background: #d1fae5;
		color: #065f46;
	}

	.comm-direction.missed {
		background: #fee2e2;
		color: #991b1b;
	}

	.comm-date {
		font-size: 0.75rem;
		color: var(--gray-500);
		margin-left: auto;
	}

	.comm-phone {
		font-size: 0.875rem;
		color: var(--gray-700);
		margin-bottom: 0.5rem;
		font-weight: 500;
	}

	.comm-content {
		margin: 0.75rem 0;
		color: var(--gray-800);
		font-size: 0.9375rem;
		line-height: 1.6;
		padding: 0.75rem;
		background: rgba(255, 255, 255, 0.7);
		border-radius: 4px;
	}

	.comm-duration {
		font-size: 0.875rem;
		color: var(--gray-600);
		font-weight: 600;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-weight: 500;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.btn-primary:hover {
		background: var(--primary-dark);
	}
</style>
