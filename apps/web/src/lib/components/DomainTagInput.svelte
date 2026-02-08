<script lang="ts">
	import { createEventDispatcher, onMount, tick } from 'svelte';
	import { api } from '$lib/api/api';
	import { domainStore } from '$lib/stores/domains';

	export let emailId: string = '';
	export let emailIds: string[] = [];
	export let currentDomains: Array<{ concept_id: string; name: string }> = [];
	export let compact: boolean = false;
	export let readonly: boolean = false;

	const dispatch = createEventDispatcher();

	let inputValue = '';
	let suggestions: Array<{ concept_id: string; name: string; keywords: string[] }> = [];
	let highlightedIndex = -1;
	let showDropdown = false;
	let showCompactInput = false;
	let inputEl: HTMLInputElement;
	let debounceTimer: ReturnType<typeof setTimeout>;
	let loading = false;

	const CHIP_COLORS = [
		'#6366f1', '#8b5cf6', '#ec4899', '#ef4444', '#f97316',
		'#eab308', '#22c55e', '#14b8a6', '#06b6d4', '#3b82f6'
	];

	function hashColor(name: string): string {
		let hash = 0;
		for (let i = 0; i < name.length; i++) {
			hash = name.charCodeAt(i) + ((hash << 5) - hash);
		}
		return CHIP_COLORS[Math.abs(hash) % CHIP_COLORS.length];
	}

	function isBulk(): boolean {
		return emailIds.length > 0;
	}

	onMount(() => {
		domainStore.load();
	});

	function handleInput() {
		const val = inputValue.trim();
		clearTimeout(debounceTimer);

		if (!val) {
			suggestions = [];
			showDropdown = false;
			return;
		}

		// First try local store
		const local = domainStore.search(val);
		if (local.length > 0) {
			suggestions = local.filter(
				(s) => !currentDomains.some((d) => d.concept_id === s.concept_id)
			);
			highlightedIndex = -1;
			showDropdown = true;
		}

		// Then fetch from API after debounce
		debounceTimer = setTimeout(async () => {
			loading = true;
			try {
				const result = await api.searchEmailDomains(val);
				suggestions = result.domains.filter(
					(s) => !currentDomains.some((d) => d.concept_id === s.concept_id)
				);
				highlightedIndex = -1;
				showDropdown = true;
			} catch {
				// Keep local results
			} finally {
				loading = false;
			}
		}, 200);
	}

	function hasExactMatch(): boolean {
		const val = inputValue.trim().toLowerCase();
		return suggestions.some((s) => s.name.toLowerCase() === val) ||
			currentDomains.some((d) => d.name.toLowerCase() === val);
	}

	async function addDomain(name: string) {
		const trimmed = name.trim();
		if (!trimmed) return;
		if (currentDomains.some((d) => d.name.toLowerCase() === trimmed.toLowerCase())) {
			inputValue = '';
			suggestions = [];
			showDropdown = false;
			return;
		}

		try {
			if (isBulk()) {
				const result = await api.bulkAssignDomains(emailIds, [trimmed]);
				if (result.created_domains.length > 0) {
					domainStore.invalidate();
				}
			} else if (emailId) {
				const result = await api.assignDomainsToEmail(emailId, [trimmed]);
				if (result.created?.length > 0) {
					domainStore.invalidate();
				}
				// Add new assignments to current list
				for (const a of result.assigned) {
					if (!currentDomains.some((d) => d.concept_id === a.concept_id)) {
						currentDomains = [...currentDomains, { concept_id: a.concept_id, name: a.name }];
					}
				}
			}
		} catch {
			// Could show toast here
		}

		inputValue = '';
		suggestions = [];
		showDropdown = false;
		dispatch('domainsChanged', { domains: currentDomains });
	}

	async function removeDomain(conceptId: string) {
		if (readonly) return;
		try {
			if (!isBulk() && emailId) {
				await api.unlinkDomainFromEmail(emailId, conceptId);
			}
			currentDomains = currentDomains.filter((d) => d.concept_id !== conceptId);
			dispatch('domainsChanged', { domains: currentDomains });
		} catch {
			// Could show toast here
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			showDropdown = false;
			highlightedIndex = -1;
			return;
		}

		if (e.key === 'Backspace' && inputValue === '' && currentDomains.length > 0) {
			const last = currentDomains[currentDomains.length - 1];
			removeDomain(last.concept_id);
			return;
		}

		if (e.key === 'ArrowDown') {
			e.preventDefault();
			const max = suggestions.length + (hasExactMatch() ? 0 : 1) - 1;
			highlightedIndex = Math.min(highlightedIndex + 1, max);
			return;
		}

		if (e.key === 'ArrowUp') {
			e.preventDefault();
			highlightedIndex = Math.max(highlightedIndex - 1, -1);
			return;
		}

		if (e.key === 'Enter' || e.key === ',') {
			e.preventDefault();
			if (highlightedIndex >= 0 && highlightedIndex < suggestions.length) {
				addDomain(suggestions[highlightedIndex].name);
			} else if (
				highlightedIndex === suggestions.length &&
				!hasExactMatch() &&
				inputValue.trim()
			) {
				addDomain(inputValue);
			} else if (inputValue.trim()) {
				addDomain(inputValue);
			}
			return;
		}
	}

	function selectSuggestion(name: string) {
		addDomain(name);
	}

	function handleBlur() {
		// Delay to allow click on dropdown
		setTimeout(() => {
			showDropdown = false;
			highlightedIndex = -1;
		}, 200);
	}

	async function openCompactInput() {
		showCompactInput = true;
		await tick();
		inputEl?.focus();
	}

	function closeCompactInput() {
		if (!inputValue.trim()) {
			showCompactInput = false;
		}
		showDropdown = false;
	}
</script>

{#if readonly}
	<div class="domain-chips readonly">
		{#each currentDomains as domain}
			<span class="chip" style="--chip-color: {hashColor(domain.name)}">
				{domain.name}
			</span>
		{/each}
		{#if currentDomains.length === 0}
			<span class="no-domains">No domains</span>
		{/if}
	</div>
{:else if compact}
	<div class="domain-tag-input compact">
		<div class="chip-row">
			{#each currentDomains as domain}
				<span class="chip compact-chip" style="--chip-color: {hashColor(domain.name)}">
					{domain.name}
					<button
						class="chip-remove"
						on:click|stopPropagation={() => removeDomain(domain.concept_id)}
						aria-label="Remove {domain.name}"
					>
						&times;
					</button>
				</span>
			{/each}
			{#if !showCompactInput}
				<button class="add-btn" on:click|stopPropagation={openCompactInput} aria-label="Add domain">
					+
				</button>
			{/if}
		</div>
		{#if showCompactInput}
			<div class="input-wrapper compact-popover">
				<input
					bind:this={inputEl}
					bind:value={inputValue}
					on:input={handleInput}
					on:keydown={handleKeydown}
					on:blur={closeCompactInput}
					type="text"
					placeholder="Type domain..."
					class="tag-input compact-input"
				/>
				{#if showDropdown && (suggestions.length > 0 || (inputValue.trim() && !hasExactMatch()))}
					<ul class="dropdown">
						{#each suggestions as suggestion, i}
							<li
								class:highlighted={i === highlightedIndex}
								on:mousedown|preventDefault={() => selectSuggestion(suggestion.name)}
							>
								{suggestion.name}
								{#if suggestion.keywords.length > 0}
									<span class="kw-hint">{suggestion.keywords.slice(0, 2).join(', ')}</span>
								{/if}
							</li>
						{/each}
						{#if inputValue.trim() && !hasExactMatch()}
							<li
								class="create-option"
								class:highlighted={highlightedIndex === suggestions.length}
								on:mousedown|preventDefault={() => selectSuggestion(inputValue)}
							>
								Create "{inputValue.trim()}"
							</li>
						{/if}
					</ul>
				{/if}
			</div>
		{/if}
	</div>
{:else}
	<div class="domain-tag-input full">
		<div class="chips-and-input">
			{#each currentDomains as domain}
				<span class="chip" style="--chip-color: {hashColor(domain.name)}">
					{domain.name}
					<button
						class="chip-remove"
						on:click|stopPropagation={() => removeDomain(domain.concept_id)}
						aria-label="Remove {domain.name}"
					>
						&times;
					</button>
				</span>
			{/each}
			<input
				bind:this={inputEl}
				bind:value={inputValue}
				on:input={handleInput}
				on:keydown={handleKeydown}
				on:blur={handleBlur}
				type="text"
				placeholder={currentDomains.length === 0 ? 'Type domain name...' : 'Add more...'}
				class="tag-input"
			/>
		</div>
		{#if showDropdown && (suggestions.length > 0 || (inputValue.trim() && !hasExactMatch()))}
			<ul class="dropdown">
				{#each suggestions as suggestion, i}
					<li
						class:highlighted={i === highlightedIndex}
						on:mousedown|preventDefault={() => selectSuggestion(suggestion.name)}
					>
						{suggestion.name}
						{#if suggestion.keywords.length > 0}
							<span class="kw-hint">{suggestion.keywords.slice(0, 2).join(', ')}</span>
						{/if}
					</li>
				{/each}
				{#if inputValue.trim() && !hasExactMatch()}
					<li
						class="create-option"
						class:highlighted={highlightedIndex === suggestions.length}
						on:mousedown|preventDefault={() => selectSuggestion(inputValue)}
					>
						Create "{inputValue.trim()}"
					</li>
				{/if}
			</ul>
		{/if}
	</div>
{/if}

<style>
	.domain-tag-input {
		position: relative;
	}

	.domain-chips.readonly {
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem;
	}

	.no-domains {
		font-size: 0.75rem;
		color: var(--gray-400, #9ca3af);
		font-style: italic;
	}

	.chips-and-input {
		display: flex;
		flex-wrap: wrap;
		gap: 0.375rem;
		align-items: center;
		padding: 0.375rem 0.5rem;
		border: 1px solid var(--gray-300, #d1d5db);
		border-radius: var(--radius-md, 6px);
		background: var(--white, #fff);
		min-height: 2.25rem;
		cursor: text;
	}

	.chips-and-input:focus-within {
		border-color: var(--primary, #6366f1);
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
	}

	.chip-row {
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem;
		align-items: center;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.125rem 0.5rem;
		border-radius: var(--radius-full, 9999px);
		font-size: 0.75rem;
		font-weight: 500;
		color: #fff;
		background-color: var(--chip-color, #6366f1);
		white-space: nowrap;
		line-height: 1.4;
	}

	.compact-chip {
		font-size: 0.6875rem;
		padding: 0.0625rem 0.375rem;
	}

	.chip-remove {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		color: rgba(255, 255, 255, 0.8);
		cursor: pointer;
		font-size: 0.875rem;
		line-height: 1;
		padding: 0;
		margin-left: 0.125rem;
		border-radius: 50%;
		width: 1rem;
		height: 1rem;
	}

	.chip-remove:hover {
		color: #fff;
		background: rgba(0, 0, 0, 0.2);
	}

	.tag-input {
		border: none;
		outline: none;
		font-size: 0.8125rem;
		flex: 1;
		min-width: 80px;
		padding: 0.125rem 0;
		background: transparent;
		color: var(--gray-900, #111827);
	}

	.compact-input {
		font-size: 0.75rem;
		min-width: 100px;
		padding: 0.25rem 0.375rem;
		border: 1px solid var(--gray-300, #d1d5db);
		border-radius: var(--radius-sm, 4px);
	}

	.compact-input:focus {
		border-color: var(--primary, #6366f1);
		outline: none;
		box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1);
	}

	.add-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 1.25rem;
		height: 1.25rem;
		border-radius: var(--radius-full, 9999px);
		border: 1px dashed var(--gray-400, #9ca3af);
		background: none;
		color: var(--gray-500, #6b7280);
		font-size: 0.75rem;
		cursor: pointer;
		padding: 0;
		line-height: 1;
	}

	.add-btn:hover {
		border-color: var(--primary, #6366f1);
		color: var(--primary, #6366f1);
		background: rgba(99, 102, 241, 0.05);
	}

	.compact-popover {
		position: relative;
		margin-top: 0.25rem;
	}

	.input-wrapper {
		position: relative;
	}

	.dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		z-index: var(--z-dropdown, 1000);
		background: var(--white, #fff);
		border: 1px solid var(--gray-200, #e5e7eb);
		border-radius: var(--radius-md, 6px);
		box-shadow: var(--shadow-lg, 0 10px 15px -3px rgba(0, 0, 0, 0.1));
		max-height: 200px;
		overflow-y: auto;
		list-style: none;
		margin: 0.25rem 0 0;
		padding: 0.25rem;
	}

	.dropdown li {
		padding: 0.375rem 0.625rem;
		font-size: 0.8125rem;
		cursor: pointer;
		border-radius: var(--radius-sm, 4px);
		display: flex;
		align-items: center;
		justify-content: space-between;
		color: var(--gray-800, #1f2937);
	}

	.dropdown li:hover,
	.dropdown li.highlighted {
		background: var(--primary-50, #eef2ff);
		color: var(--primary-700, #4338ca);
	}

	.kw-hint {
		font-size: 0.6875rem;
		color: var(--gray-400, #9ca3af);
		margin-left: 0.5rem;
		flex-shrink: 0;
	}

	.create-option {
		font-style: italic;
		color: var(--primary-600, #4f46e5) !important;
	}

	.full .dropdown {
		min-width: 100%;
	}
</style>
