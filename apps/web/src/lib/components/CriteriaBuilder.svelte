<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { api } from '$lib/api/api';
	import type { MatcherGroup, MatcherInput, MatcherGroupInput } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';

	export let conceptId: string;
	export let groups: MatcherGroup[] = [];
	export let readonly: boolean = false;

	const dispatch = createEventDispatcher();

	const ELEMENT_TYPES = [
		{ value: 'sender', label: 'Sender' },
		{ value: 'receiver', label: 'Receiver' },
		{ value: 'subject', label: 'Subject' },
		{ value: 'body', label: 'Body' },
		{ value: 'attachment_name', label: 'Attachment' },
		{ value: 'any_text', label: 'Any Text' }
	];

	const MATCH_MODES = [
		{ value: 'contains', label: 'contains' },
		{ value: 'exact', label: 'equals' },
		{ value: 'starts_with', label: 'starts with' },
		{ value: 'ends_with', label: 'ends with' }
	];

	// Editable local copy
	let editGroups: Array<{
		id: string;
		matchers: Array<{
			id: string;
			element_type: string;
			match_value: string;
			match_mode: string;
			negate: boolean;
			case_sensitive: boolean;
		}>;
	}> = [];

	let saving = false;
	let scanning = false;

	$: {
		editGroups = groups.map(g => ({
			id: g.id,
			matchers: g.matchers.map(m => ({
				id: m.id,
				element_type: m.element_type,
				match_value: m.match_value,
				match_mode: m.match_mode,
				negate: m.negate,
				case_sensitive: m.case_sensitive
			}))
		}));
	}

	function addGroup() {
		editGroups = [...editGroups, {
			id: 'new-' + Date.now(),
			matchers: [{
				id: 'new-m-' + Date.now(),
				element_type: 'subject',
				match_value: '',
				match_mode: 'contains',
				negate: false,
				case_sensitive: false
			}]
		}];
	}

	function addMatcher(groupIdx: number) {
		editGroups[groupIdx].matchers = [...editGroups[groupIdx].matchers, {
			id: 'new-m-' + Date.now() + '-' + groupIdx,
			element_type: 'subject',
			match_value: '',
			match_mode: 'contains',
			negate: false,
			case_sensitive: false
		}];
		editGroups = editGroups;
	}

	function removeMatcher(groupIdx: number, matcherIdx: number) {
		editGroups[groupIdx].matchers.splice(matcherIdx, 1);
		if (editGroups[groupIdx].matchers.length === 0) {
			editGroups.splice(groupIdx, 1);
		}
		editGroups = editGroups;
	}

	function removeGroup(groupIdx: number) {
		editGroups.splice(groupIdx, 1);
		editGroups = editGroups;
	}

	function buildPayload(): MatcherGroupInput[] {
		return editGroups.map(g => ({
			matchers: g.matchers
				.filter(m => m.match_value.trim())
				.map(m => ({
					element_type: m.element_type,
					match_value: m.match_value,
					match_mode: m.match_mode,
					negate: m.negate,
					case_sensitive: m.case_sensitive
				}))
		})).filter(g => g.matchers.length > 0);
	}

	async function save() {
		saving = true;
		try {
			const payload = buildPayload();
			const result = await api.setConceptMatchers(conceptId, payload);
			groups = result.groups;
			toasts.success('Criteria saved');
			dispatch('save');
		} catch (e: any) {
			toasts.error(e.message || 'Failed to save');
		} finally {
			saving = false;
		}
	}

	async function saveAndScan() {
		saving = true;
		scanning = true;
		try {
			const payload = buildPayload();
			const result = await api.setConceptMatchers(conceptId, payload, true);
			groups = result.groups;
			toasts.success(`Criteria saved. ${result.scan_count} new suggestions found.`);
			dispatch('save');
			dispatch('scan');
		} catch (e: any) {
			toasts.error(e.message || 'Failed to save & scan');
		} finally {
			saving = false;
			scanning = false;
		}
	}
</script>

<div class="criteria-builder">
	{#if editGroups.length === 0}
		<div class="empty-state">
			<p>No criteria defined yet.</p>
			{#if !readonly}
				<button class="btn btn-primary" on:click={addGroup}>Add Criteria Group</button>
			{/if}
		</div>
	{:else}
		{#each editGroups as group, gi}
			{#if gi > 0}
				<div class="or-divider"><span>OR</span></div>
			{/if}
			<div class="group-card">
				<div class="group-header">
					<span class="group-label">Group {gi + 1}</span>
					<span class="logic-badge">AND logic</span>
					{#if !readonly}
						<button class="btn-icon danger" on:click={() => removeGroup(gi)} title="Remove group">
							&times;
						</button>
					{/if}
				</div>
				{#each group.matchers as matcher, mi}
					<div class="matcher-row">
						{#if matcher.negate}
							<span class="not-badge">NOT</span>
						{/if}
						<select bind:value={matcher.element_type} disabled={readonly} class="select-el">
							{#each ELEMENT_TYPES as et}
								<option value={et.value}>{et.label}</option>
							{/each}
						</select>
						<select bind:value={matcher.match_mode} disabled={readonly} class="select-mode">
							{#each MATCH_MODES as mm}
								<option value={mm.value}>{mm.label}</option>
							{/each}
						</select>
						<input
							type="text"
							bind:value={matcher.match_value}
							placeholder="Value to match..."
							disabled={readonly}
							class="input-value"
						/>
						{#if !readonly}
							<label class="toggle-label" title="Negate (NOT)">
								<input type="checkbox" bind:checked={matcher.negate} />
								NOT
							</label>
							<label class="toggle-label" title="Case sensitive">
								<input type="checkbox" bind:checked={matcher.case_sensitive} />
								Aa
							</label>
							<button class="btn-icon danger" on:click={() => removeMatcher(gi, mi)} title="Remove">
								&times;
							</button>
						{/if}
					</div>
				{/each}
				{#if !readonly}
					<button class="btn btn-sm" on:click={() => addMatcher(gi)}>+ Add Criterion</button>
				{/if}
			</div>
		{/each}
	{/if}

	{#if !readonly}
		<div class="builder-actions">
			<button class="btn btn-outline" on:click={addGroup}>+ Add OR Group</button>
			<div class="save-actions">
				<button class="btn btn-primary" on:click={save} disabled={saving}>
					{saving && !scanning ? 'Saving...' : 'Save'}
				</button>
				<button class="btn btn-secondary" on:click={saveAndScan} disabled={saving}>
					{scanning ? 'Scanning...' : 'Save & Scan'}
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.criteria-builder {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.empty-state {
		text-align: center;
		padding: 2rem;
		color: #6b7280;
	}

	.or-divider {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		color: #9ca3af;
		font-size: 0.8rem;
		font-weight: 600;
	}

	.or-divider::before,
	.or-divider::after {
		content: '';
		flex: 1;
		border-top: 1px dashed #d1d5db;
	}

	.group-card {
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 0.75rem;
		background: #fafafa;
	}

	.group-header {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
		font-size: 0.85rem;
	}

	.group-label {
		font-weight: 600;
		color: #374151;
	}

	.logic-badge {
		font-size: 0.7rem;
		background: #dbeafe;
		color: #1d4ed8;
		padding: 0.1rem 0.4rem;
		border-radius: 0.25rem;
	}

	.not-badge {
		font-size: 0.7rem;
		background: #fee2e2;
		color: #dc2626;
		padding: 0.1rem 0.4rem;
		border-radius: 0.25rem;
		font-weight: 600;
	}

	.matcher-row {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		margin-bottom: 0.4rem;
		flex-wrap: wrap;
	}

	.select-el {
		width: 110px;
		padding: 0.3rem 0.4rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		font-size: 0.85rem;
	}

	.select-mode {
		width: 100px;
		padding: 0.3rem 0.4rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		font-size: 0.85rem;
	}

	.input-value {
		flex: 1;
		min-width: 120px;
		padding: 0.3rem 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		font-size: 0.85rem;
	}

	.toggle-label {
		font-size: 0.75rem;
		color: #6b7280;
		display: flex;
		align-items: center;
		gap: 0.2rem;
		cursor: pointer;
		white-space: nowrap;
	}

	.toggle-label input[type="checkbox"] {
		width: 14px;
		height: 14px;
	}

	.btn-icon {
		border: none;
		background: none;
		cursor: pointer;
		font-size: 1.2rem;
		line-height: 1;
		padding: 0.1rem 0.3rem;
		border-radius: 0.25rem;
	}

	.btn-icon.danger {
		color: #ef4444;
	}

	.btn-icon.danger:hover {
		background: #fee2e2;
	}

	.builder-actions {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: 0.5rem;
	}

	.save-actions {
		display: flex;
		gap: 0.5rem;
	}

	.btn {
		padding: 0.4rem 0.8rem;
		border-radius: 0.375rem;
		font-size: 0.85rem;
		cursor: pointer;
		border: 1px solid #d1d5db;
		background: white;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-secondary:hover:not(:disabled) {
		background: #e5e7eb;
	}

	.btn-outline {
		background: transparent;
		color: #6b7280;
	}

	.btn-outline:hover {
		background: #f3f4f6;
	}

	.btn-sm {
		padding: 0.2rem 0.5rem;
		font-size: 0.8rem;
	}
</style>
