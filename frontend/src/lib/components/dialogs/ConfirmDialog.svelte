<script lang="ts">
	import Modal from '../Modal.svelte';
	import { closeDialog } from '#lib/state.svelte.ts';

	let {
		title,
		message,
		confirmLabel,
		danger = false,
		onConfirm
	}: { title: string; message: string; confirmLabel: string; danger?: boolean; onConfirm: () => Promise<void> | void } = $props();

	let busy = $state(false);
	let error = $state('');

	async function confirm() {
		busy = true;
		error = '';
		try {
			await onConfirm();
			closeDialog();
		} catch (e) {
			error = (e as Error).message;
		} finally {
			busy = false;
		}
	}
</script>

<Modal {title} onclose={closeDialog}>
	<p style="margin:0" class="muted">{message}</p>
	{#if error}<div class="alert danger" role="alert" style="margin-top:12px">{error}</div>{/if}
	{#snippet footer()}
		<div class="form-actions">
			<button type="button" class="btn" onclick={closeDialog}>Annulla</button>
			<button type="button" class="btn {danger ? 'btn-danger' : 'btn-primary'}" onclick={confirm} disabled={busy}>{confirmLabel}</button>
		</div>
	{/snippet}
</Modal>
