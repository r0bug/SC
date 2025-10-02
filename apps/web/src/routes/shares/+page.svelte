<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { ShareInvite } from '$lib/api/types';

	let shares: ShareInvite[] = [];
	let loading = true;

	onMount(async () => {
		await loadShares();
	});

	async function loadShares() {
		try {
			shares = await api.getShares();
		} catch (error) {
			console.error('Failed to load shares:', error);
		} finally {
			loading = false;
		}
	}

	async function handleAccept(id: string) {
		try {
			await api.acceptShare(id);
			shares = shares.map(s => s.id === id ? {...s, accepted: true} : s);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleRevoke(id: string) {
		if (!confirm('Revoke this share?')) return;
		try {
			await api.revokeShare(id);
			shares = shares.filter(s => s.id !== id);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<h1>Shared Items</h1>

	{#if loading}
		<div>Loading...</div>
	{:else if shares.length === 0}
		<div class="empty">No shared items</div>
	{:else}
		<div class="shares-list">
			{#each shares as share}
				<div class="share-card card">
					<div>
						<h3>{share.entity_type} Share</h3>
						<p>Shared with: {share.shared_with_email}</p>
						<p>Permissions: {share.permissions.join(', ')}</p>
						<p class="status">{share.accepted ? 'Accepted' : 'Pending'}</p>
					</div>
					<div class="actions">
						{#if !share.accepted}
							<button on:click={() => handleAccept(share.id)} class="btn btn-primary">Accept</button>
						{/if}
						<button on:click={() => handleRevoke(share.id)} class="btn btn-secondary">Revoke</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1000px; }
	.shares-list { display: flex; flex-direction: column; gap: 1rem; }
	.share-card { padding: 1.5rem; display: flex; justify-content: space-between; align-items: center; }
	.actions { display: flex; gap: 0.5rem; }
	.status { font-size: 0.875rem; color: var(--gray-600); }
	.empty { text-align: center; padding: 3rem; }
</style>
