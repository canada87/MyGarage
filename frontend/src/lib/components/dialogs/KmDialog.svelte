<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import VehicleSelect from './VehicleSelect.svelte';
	import { api } from '#lib/api.ts';
	import { daysAgo, fmtKm, parseIntOrNull, todayIso } from '#lib/format.ts';
	import { app, closeDialog, refresh, toast } from '#lib/state.svelte.ts';

	const props: { vehicleId?: number } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { vehicleId } = untrack(() => props);

	let vid = $state<number | null>(vehicleId ?? app.vehicles[0]?.id ?? null);
	let date = $state(todayIso());
	let km = $state('');
	let note = $state('');
	let error = $state('');
	let saving = $state(false);

	const current = $derived(app.vehicles.find((v) => v.id === vid));
	const kmValue = $derived(parseIntOrNull(km));
	const lower = $derived(kmValue != null && current?.current_km != null && kmValue < current.current_km);

	async function save(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (!vid) return (error = 'Scegli un’auto');
		if (kmValue == null || kmValue < 0) return (error = 'Inserisci i km del contachilometri');
		saving = true;
		try {
			await api.post(`/vehicles/${vid}/odometer`, { date, km: kmValue, note });
			toast('Km aggiornati');
			refresh();
			closeDialog();
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}
</script>

<Modal title="Aggiorna km" onclose={closeDialog}>
	<form class="form" onsubmit={save} id="km-form">
		{#if !vehicleId}<VehicleSelect bind:value={vid} />{/if}
		{#if current}
			<p class="subtle" style="margin:0">
				Ultima lettura: <strong class="num">{fmtKm(current.current_km)}</strong>, {daysAgo(current.current_km_date)}
			</p>
		{/if}
		<div class="form-grid cols-2">
			<div class="field">
				<label for="km-val">Km attuali <span class="req">*</span></label>
				<div class="input-affix">
					<input id="km-val" class="input num" type="text" inputmode="numeric" autocomplete="off" bind:value={km} required aria-invalid={!!error} />
					<span class="affix">km</span>
				</div>
			</div>
			<div class="field">
				<label for="km-date">Data</label>
				<input id="km-date" class="input" type="date" bind:value={date} max={todayIso()} required />
			</div>
		</div>
		{#if lower}
			<div class="alert warn">Il valore è più basso dell’ultima lettura ({fmtKm(current?.current_km)}). Controlla prima di salvare.</div>
		{/if}
		<div class="field">
			<label for="km-note">Nota</label>
			<input id="km-note" class="input" type="text" bind:value={note} placeholder="Facoltativa" />
		</div>
		{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
	</form>
	{#snippet footer()}
		<div class="form-actions">
			<button type="button" class="btn" onclick={closeDialog}>Annulla</button>
			<button type="submit" form="km-form" class="btn btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
		</div>
	{/snippet}
</Modal>
