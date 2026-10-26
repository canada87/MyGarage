<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import { api } from '#lib/api.ts';
	import { parseIntOrNull, todayIso } from '#lib/format.ts';
	import { app, closeDialog, refresh, toast } from '#lib/state.svelte.ts';
	import type { TyreSetView } from '#lib/types.ts';

	const props: { set: TyreSetView; action: 'mount' | 'unmount' } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { set, action } = untrack(() => props);

	const vehicle = $derived(app.vehicles.find((v) => v.id === set.vehicle_id));
	let date = $state(todayIso());
	let km = $state('');
	let error = $state('');
	let saving = $state(false);

	async function save(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		saving = true;
		try {
			await api.post(`/tyre-sets/${set.id}/${action}`, { date, km: parseIntOrNull(km) });
			toast(action === 'mount' ? `${set.name} montate` : `${set.name} smontate`);
			refresh();
			closeDialog();
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}
</script>

<Modal title={action === 'mount' ? `Monta ${set.name}` : `Smonta ${set.name}`} onclose={closeDialog}>
	<form class="form" onsubmit={save} id="mount-form">
		{#if action === 'mount'}
			<p class="subtle" style="margin:0">Le gomme montate ora sull’auto verranno segnate come smontate alla stessa data.</p>
		{/if}
		<div class="form-grid cols-2">
			<div class="field">
				<label for="mt-date">Data</label>
				<input id="mt-date" class="input" type="date" bind:value={date} required />
			</div>
			<div class="field">
				<label for="mt-km">Km</label>
				<div class="input-affix">
					<input id="mt-km" class="input num" type="text" inputmode="numeric" bind:value={km} placeholder={vehicle?.current_km != null ? String(vehicle.current_km) : ''} />
					<span class="affix">km</span>
				</div>
				<span class="hint">Se non li sai, li stimo dalle letture dei km.</span>
			</div>
		</div>
		{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
	</form>
	{#snippet footer()}
		<div class="form-actions">
			<button type="button" class="btn" onclick={closeDialog}>Annulla</button>
			<button type="submit" form="mount-form" class="btn btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Conferma'}</button>
		</div>
	{/snippet}
</Modal>
