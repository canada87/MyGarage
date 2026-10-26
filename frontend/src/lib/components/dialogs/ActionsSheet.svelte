<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import Icon from '../Icon.svelte';
	import { goto } from '$app/navigation';
	import { app, closeDialog, openDialog } from '#lib/state.svelte.ts';

	const props: { vehicleId?: number } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { vehicleId } = untrack(() => props);

	const items = [
		{ icon: 'gauge', label: 'Aggiorna km', hint: 'Il contachilometri di oggi', run: () => openDialog({ kind: 'km', vehicleId }) },
		{
			icon: 'wrench',
			label: 'Registra intervento o spesa',
			hint: 'Tagliando, RCA pagata, riparazione…',
			run: () => openDialog({ kind: 'event', vehicleId })
		},
		{ icon: 'upload', label: 'Carica documento', hint: 'Libretto, fattura, polizza', run: () => openDialog({ kind: 'document', vehicleId }) }
	];
</script>

<Modal title="Registra" onclose={closeDialog}>
	{#if app.vehicles.length === 0}
		<div class="empty">
			<span class="tile-ico"><Icon name="car" size={28} /></span>
			<p>Prima aggiungi un’auto.</p>
			<button
				class="btn btn-primary"
				onclick={() => {
					closeDialog();
					goto('/auto/nuova');
				}}><Icon name="plus" />Aggiungi auto</button
			>
		</div>
	{:else}
		<ul class="list actions">
			{#each items as it (it.label)}
				<li>
					<button type="button" onclick={it.run}>
						<span class="tile-ico"><Icon name={it.icon} /></span>
						<span class="grow"><span class="t">{it.label}</span><span class="s">{it.hint}</span></span>
						<Icon name="chevron-right" size={18} />
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</Modal>

<style>
	.actions {
		padding: 0;
	}
	.actions li {
		padding: 0;
	}
	.actions button {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 12px 8px;
		min-height: 64px;
		border: 0;
		background: transparent;
		border-radius: 12px;
		cursor: pointer;
		text-align: left;
		color: inherit;
	}
	.actions button:hover {
		background: var(--surface-2);
	}
	.grow {
		display: grid;
	}
</style>
