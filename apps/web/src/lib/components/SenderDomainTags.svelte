<script lang="ts">
	import { createEventDispatcher, onMount, tick } from 'svelte';
	import { api } from '$lib/api/api';
	import { domainStore } from '$lib/stores/domains';
	import { hashColor } from '$lib/utils/colors';

	export let senderIdentifier: string = '';
	export let mode: 'email' | 'sms' = 'email';

	const dispatch = createEventDispatcher();

	let currentDomains: Array<{ concept_id: string; name: string }> = [];
	let inputValue = '';
	let suggestions: Array<{ concept_id: string; name: string; keywords: string[] }> = [];
	let highlightedIndex = -1;
	let showDropdown = false;
	let showInput = false;
	let inputEl: HTMLInputElement;
	let debounceTimer: ReturnType<typeof setTimeout>;
	let loading = false;
	let assigning = false;

	onMount(() => {
		domainStore.load();
	});

	$: if (senderIdentifier) {
		loadDomains();
	}

	async function loadDomains() {
		if (!senderIdentifier) {
			currentDomains = [];
			return;
		}
		loading = true;
		try {
			const result = mode === 'email'
				? await api.getSenderDomains(senderIdentifier)
				: await api.getSmsSenderDomains(senderIdentifier);
			currentDomains = result.domains.map(d => ({ concept_id: d.concept_id, name: d.name }));
		} catch {
			currentDomains = [];
		} finally {
			loading = false;
		}
	}

	function handleInput() {
		const val = inputValue.trim();
		clearTimeout(debounceTimer);

		if (!val) {
			suggestions = [];
			showDropdown = false;
			return;
		}

		const local = domainStore.search(val);
		if (local.length > 0) {
			suggestions = local.filter(
				(s) => !currentDomains.some((d) => d.concept_id === s.concept_id)
			);
			highlightedIndex = -1;
			showDropdown = true;
		}

		debounceTimer = setTimeout(async () => {
			try {
				const result = await api.searchEmailDomains(val);
				suggestions = result.domains.filter(
					(s) => !currentDomains.some((d) => d.concept_id === s.concept_id)
				);
				highlightedIndex = -1;
				showDropdown = true;
			} catch {
				// Keep local results
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
		if (!trimmed || assigning) return;
		if (currentDomains.some((d) => d.name.toLowerCase() === trimmed.toLowerCase())) {
			inputValue = '';
			suggestions = [];
			showDropdown = false;
			return;
		}

		assigning = true;
		try {
			if (mode === 'email') {
				const result = await api.assignDomainsToSender(senderIdentifier, [trimmed]);
				if (result.created_domains.length > 0) {
					domainStore.invalidate();
				}
			} else {
				const result = await api.assignDomainsToSmsSender(senderIdentifier, [trimmed]);
				if (result.created_domains.length > 0) {
					domainStore.invalidate();
				}
			}
			// Reload domains to get accurate state
			await loadDomains();
		} catch {
			// Could show toast
		} finally {
			assigning = false;
		}

		inputValue = '';
		suggestions = [];
		showDropdown = false;
		dispatch('domainsChanged', { domains: currentDomains });
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			showDropdown = false;
			highlightedIndex = -1;
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
		setTimeout(() => {
			showDropdown = false;
			highlightedIndex = -1;
			if (!inputValue.trim()) {
				showInput = false;
			}
		}, 200);
	}

	async function openInput() {
		showInput = true;
		await tick();
		inputEl?.focus();
	}
</script>

<div class="sender-domain-tags">
	<div class="domain-label">
		<span class="label-text">Domains</span>
		<span class="label-hint">
			{#if mode === 'email'}
				(applies to all emails from this sender)
			{:else}
				(applies to all messages from this number)
			{/if}
		</span>
	</div>
	<div class="chip-row">
		{#if loading}
			<span class="loading-text">Loading...</span>
		{:else}
			{#each currentDomains as domain}
				<span class="chip" style="--chip-color: {hashColor(domain.name)}">
					{domain.name}
				</span>
			{/each}
			{#if !showInput}
				<button class="add-btn" on:click|stopPropagation={openInput} aria-label="Add domain">
					+
				</button>
			{/if}
		{/if}
	</div>
	{#if showInput}
		<div class="input-wrapper">
			<input
				bind:this={inputEl}
				bind:value={inputValue}
				on:input={handleInput}
				on:keydown={handleKeydown}
				on:blur={handleBlur}
				type="text"
				placeholder="Type domain name..."
				class="domain-input"
				disabled={assigning}
			/>
			{#if assigning}
				<span class="assigning-indicator"></span>
			{/if}
			{#if showDropdown && (suggestions.length > 0 || (inputValue.trim() && !hasExactMatch()))}
				<ul class="dropdown" role="listbox">
					{#each suggestions as suggestion, i}
						<li
							role="option"
							aria-selected={i === highlightedIndex}
							tabindex="-1"
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
							role="option"
							aria-selected={highlightedIndex === suggestions.length}
							tabindex="-1"
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

<style>
	.sender-domain-tags {
		padding: 0.5rem 0;
	}

	.domain-label {
		display: flex;
		align-items: baseline;
		gap: 0.375rem;
		margin-bottom: 0.375rem;
	}

	.label-text {
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--gray-600, #4b5563);
	}

	.label-hint {
		font-size: 0.6875rem;
		color: var(--gray-400, #9ca3af);
		font-style: italic;
	}

	.chip-row {
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem;
		align-items: center;
	}

	.loading-text {
		font-size: 0.75rem;
		color: var(--gray-400, #9ca3af);
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.125rem 0.5rem;
		border-radius: 9999px;
		font-size: 0.6875rem;
		font-weight: 500;
		color: #fff;
		background-color: var(--chip-color, #6366f1);
		white-space: nowrap;
		line-height: 1.4;
	}

	.add-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 1.25rem;
		height: 1.25rem;
		border-radius: 9999px;
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

	.input-wrapper {
		position: relative;
		margin-top: 0.375rem;
	}

	.domain-input {
		width: 100%;
		max-width: 220px;
		padding: 0.25rem 0.5rem;
		border: 1px solid var(--gray-300, #d1d5db);
		border-radius: 0.25rem;
		font-size: 0.75rem;
		box-sizing: border-box;
	}

	.domain-input:focus {
		outline: none;
		border-color: var(--primary, #6366f1);
		box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1);
	}

	.domain-input:disabled {
		opacity: 0.6;
	}

	.assigning-indicator {
		position: absolute;
		right: 0.5rem;
		top: 50%;
		transform: translateY(-50%);
		width: 0.75rem;
		height: 0.75rem;
		border: 2px solid #e5e7eb;
		border-top-color: #6366f1;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: translateY(-50%) rotate(360deg); }
	}

	.dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		max-width: 220px;
		z-index: 1000;
		background: #fff;
		border: 1px solid var(--gray-200, #e5e7eb);
		border-radius: 0.375rem;
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
		max-height: 180px;
		overflow-y: auto;
		list-style: none;
		margin: 0.25rem 0 0;
		padding: 0.25rem;
	}

	.dropdown li {
		padding: 0.375rem 0.5rem;
		font-size: 0.75rem;
		cursor: pointer;
		border-radius: 0.25rem;
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
		font-size: 0.625rem;
		color: var(--gray-400, #9ca3af);
		margin-left: 0.375rem;
		flex-shrink: 0;
	}

	.create-option {
		font-style: italic;
		color: var(--primary-600, #4f46e5) !important;
	}
</style>
