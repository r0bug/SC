<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let fromAddress: string = '';
	export let toAddress: string = '';
	export let subject: string = '';
	export let bodyText: string = '';
	export let attachmentNames: string[] = [];

	const dispatch = createEventDispatcher();

	let showPicker = false;
	let selectedText = '';
	let selectionSource = '';

	function handleTextSelection() {
		const selection = window.getSelection();
		if (!selection || selection.isCollapsed) {
			selectedText = '';
			return;
		}

		const text = selection.toString().trim();
		if (!text) {
			selectedText = '';
			return;
		}

		selectedText = text;

		// Determine source from selection anchor
		const anchor = selection.anchorNode?.parentElement;
		if (anchor) {
			const container = anchor.closest('[data-field]');
			selectionSource = container?.getAttribute('data-field') || 'body';
		} else {
			selectionSource = 'body';
		}
	}

	function addSelectedText() {
		if (!selectedText) return;
		dispatch('addCriterion', {
			element_type: selectionSource === 'subject' ? 'subject' : 'body',
			match_value: selectedText,
			match_mode: 'contains'
		});
		selectedText = '';
		window.getSelection()?.removeAllRanges();
	}

	function addElement(type: string, value: string) {
		dispatch('addCriterion', {
			element_type: type,
			match_value: value,
			match_mode: type === 'sender' || type === 'receiver' ? 'exact' : 'contains'
		});
	}
</script>

<svelte:window on:mouseup={handleTextSelection} />

<div class="element-selector">
	{#if selectedText}
		<div class="selection-toolbar">
			<span class="selected-preview">"{selectedText.substring(0, 40)}{selectedText.length > 40 ? '...' : ''}"</span>
			<button class="btn-tag" on:click={addSelectedText}>
				+ Tag as Criterion
			</button>
		</div>
	{/if}

	<button class="btn-toggle" on:click={() => showPicker = !showPicker}>
		{showPicker ? 'Hide Elements' : 'Pick Elements'}
	</button>

	{#if showPicker}
		<div class="element-pills">
			{#if fromAddress}
				<button class="pill sender" on:click={() => addElement('sender', fromAddress)} title="Add sender as criterion">
					Sender: {fromAddress}
				</button>
			{/if}
			{#if toAddress}
				<button class="pill receiver" on:click={() => addElement('receiver', toAddress)} title="Add receiver as criterion">
					To: {toAddress}
				</button>
			{/if}
			{#each attachmentNames as name}
				<button class="pill attachment" on:click={() => addElement('attachment_name', name)} title="Add attachment as criterion">
					Att: {name}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.element-selector {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
		margin-top: 0.5rem;
	}

	.selection-toolbar {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.4rem 0.6rem;
		background: #eff6ff;
		border: 1px solid #bfdbfe;
		border-radius: 0.375rem;
		font-size: 0.8rem;
	}

	.selected-preview {
		color: #1d4ed8;
		font-style: italic;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.btn-tag {
		padding: 0.2rem 0.5rem;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 0.25rem;
		cursor: pointer;
		font-size: 0.75rem;
		white-space: nowrap;
	}

	.btn-tag:hover {
		background: #2563eb;
	}

	.btn-toggle {
		align-self: flex-start;
		padding: 0.2rem 0.5rem;
		background: #f3f4f6;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		cursor: pointer;
		font-size: 0.75rem;
		color: #4b5563;
	}

	.btn-toggle:hover {
		background: #e5e7eb;
	}

	.element-pills {
		display: flex;
		flex-wrap: wrap;
		gap: 0.3rem;
	}

	.pill {
		padding: 0.2rem 0.5rem;
		border: 1px dashed #d1d5db;
		border-radius: 1rem;
		background: white;
		cursor: pointer;
		font-size: 0.75rem;
		color: #4b5563;
		transition: all 0.15s;
	}

	.pill:hover {
		border-style: solid;
		background: #f0fdf4;
		border-color: #86efac;
	}

	.pill.sender {
		border-color: #93c5fd;
	}

	.pill.receiver {
		border-color: #c4b5fd;
	}

	.pill.attachment {
		border-color: #fcd34d;
	}
</style>
