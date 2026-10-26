<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import Icon from '../Icon.svelte';
	import { api } from '#lib/api.ts';
	import { parseIntOrNull, todayIso } from '#lib/format.ts';
	import { app, closeDialog, deleteWithUndo, refresh, toast } from '#lib/state.svelte.ts';
	import type { TyreSet, TyreSetView } from '#lib/types.ts';

	const props: { vehicleId: number; set?: TyreSetView } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { vehicleId, set } = untrack(() => props);

	const editing = !!set;
	const vehicle = $derived(app.vehicles.find((v) => v.id === vehicleId));
	let name = $state(set?.name ?? '');
	let season = $state(set?.season ?? '');
	let brand = $state(set?.brand ?? '');
	let model = $state(set?.model ?? '');
	let size = $state(set?.size ?? '');
	let dot = $state(set?.dot ?? '');
	let purchaseDate = $state(set?.purchase_date ?? '');
	let initialKm = $state(set?.initial_km != null ? String(set.initial_km) : '');
	let notes = $state(set?.notes ?? '');
	let retired = $state(set?.retired ?? false);
	let mountNow = $state(false);
	let error = $state('');
	let saving = $state(false);

	function pickSeason(s: string) {
		season = s;
		if (!name.trim() || ['Estive', 'Invernali', '4 stagioni'].includes(name)) {
			name = { estive: 'Estive', invernali: 'Invernali', '4stagioni': '4 stagioni' }[s] ?? name;
		}
	}

	async function save(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (!name.trim()) return (error = 'Dai un nome al treno, es. “Invernali”');
		saving = true;
		const body = {
			name,
			season: season || null,
			brand,
			model,
			size,
			dot,
			purchase_date: purchaseDate || null,
			initial_km: parseIntOrNull(initialKm),
			notes,
			retired
		};
		try {
			if (editing) {
				await api.put(`/tyre-sets/${set!.id}`, body);
			} else {
				const created = await api.post<TyreSet>(`/vehicles/${vehicleId}/tyres`, body);
				if (mountNow) {
					await api.post(`/tyre-sets/${created.id}/mount`, { date: todayIso(), km: vehicle?.current_km ?? null });
				}
			}
			toast(editing ? 'Gomme aggiornate' : 'Treno di gomme aggiunto');
			refresh();
			closeDialog();
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}

	function remove() {
		deleteWithUndo('tyreSet', set!.id, 'Treno di gomme', `/tyre-sets/${set!.id}`);
		closeDialog();
	}
</script>

<Modal title={editing ? 'Modifica gomme' : 'Nuovo treno di gomme'} onclose={closeDialog}>
	<form class="form" onsubmit={save} id="tyre-form">
		<div class="field">
			<span class="label">Stagione</span>
			<div class="segmented" role="group" aria-label="Stagione">
				<button type="button" aria-pressed={season === 'estive'} onclick={() => pickSeason('estive')}>Estive</button>
				<button type="button" aria-pressed={season === 'invernali'} onclick={() => pickSeason('invernali')}>Invernali</button>
				<button type="button" aria-pressed={season === '4stagioni'} onclick={() => pickSeason('4stagioni')}>4 stagioni</button>
			</div>
		</div>
		<div class="field">
			<label for="ty-name">Nome <span class="req">*</span></label>
			<input id="ty-name" class="input" bind:value={name} required />
		</div>
		<p class="subtle" style="margin:0">Tutto il resto è facoltativo.</p>
		<div class="form-grid cols-2">
			<div class="field"><label for="ty-brand">Marca</label><input id="ty-brand" class="input" bind:value={brand} /></div>
			<div class="field"><label for="ty-model">Modello</label><input id="ty-model" class="input" bind:value={model} /></div>
			<div class="field"><label for="ty-size">Misura</label><input id="ty-size" class="input" bind:value={size} placeholder="Es. 205/55 R16" /></div>
			<div class="field"><label for="ty-dot">DOT</label><input id="ty-dot" class="input" bind:value={dot} placeholder="Es. 2423" /></div>
			<div class="field"><label for="ty-buy">Acquistate il</label><input id="ty-buy" class="input" type="date" bind:value={purchaseDate} /></div>
			<div class="field">
				<label for="ty-init">Km già percorsi</label>
				<div class="input-affix">
					<input id="ty-init" class="input num" type="text" inputmode="numeric" bind:value={initialKm} />
					<span class="affix">km</span>
				</div>
				<span class="hint">Se le gomme erano già usate prima di inserirle qui.</span>
			</div>
		</div>
		<div class="field"><label for="ty-notes">Note</label><textarea id="ty-notes" class="input" bind:value={notes}></textarea></div>
		{#if editing}
			<label class="check"><input type="checkbox" bind:checked={retired} />Dismesse (non più in uso)</label>
		{:else}
			<label class="check"><input type="checkbox" bind:checked={mountNow} />Sono montate adesso</label>
		{/if}
		{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
	</form>
	{#snippet footer()}
		<div class="form-actions">
			{#if editing}
				<button type="button" class="btn btn-danger" onclick={remove}><Icon name="trash" size={18} />Elimina</button>
				<span class="spacer"></span>
			{/if}
			<button type="button" class="btn" onclick={closeDialog}>Annulla</button>
			<button type="submit" form="tyre-form" class="btn btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
		</div>
	{/snippet}
</Modal>
