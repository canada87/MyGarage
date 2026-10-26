<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import Icon from '../Icon.svelte';
	import VehicleSelect from './VehicleSelect.svelte';
	import { api, qs } from '#lib/api.ts';
	import {
		categories,
		categoryLabel,
		centsToInput,
		fmtBytes,
		fmtKm,
		parseEuro,
		parseIntOrNull,
		todayIso
	} from '#lib/format.ts';
	import { app, closeDialog, deleteWithUndo, refresh, toast } from '#lib/state.svelte.ts';
	import type { CarEvent, Category, DocumentFile, ScheduleView } from '#lib/types.ts';

	const props: { vehicleId?: number; schedule?: ScheduleView; event?: CarEvent } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { vehicleId, schedule, event } = untrack(() => props);

	const editing = !!event;
	let vid = $state<number | null>(event?.vehicle_id ?? schedule?.vehicle_id ?? vehicleId ?? app.vehicles[0]?.id ?? null);
	let scheduleId = $state<number | null>(event ? event.schedule_id : (schedule?.id ?? null));
	let title = $state(event?.title ?? schedule?.title ?? '');
	let category = $state<Category>(event?.category ?? schedule?.category ?? 'manutenzione');
	let date = $state(event?.date ?? todayIso());
	let km = $state(event?.km != null ? String(event.km) : '');
	let cost = $state(centsToInput(event?.cost_cents ?? null));
	let provider = $state(event?.provider ?? '');
	let notes = $state(event?.notes ?? '');
	let files = $state<FileList | null>(null);
	let docKind = $state('fattura');
	let error = $state('');
	let saving = $state(false);

	let schedules = $state<ScheduleView[]>(schedule ? [schedule] : []);
	let docs = $state<DocumentFile[]>([]);

	$effect(() => {
		if (!vid) return;
		const v = vid;
		api.get<ScheduleView[]>(`/schedules${qs({ vehicle_id: v, include_inactive: true })}`).then((r) => {
			if (vid === v) schedules = r;
		});
	});
	$effect(() => {
		if (event) api.get<DocumentFile[]>(`/documents${qs({ event_id: event.id })}`).then((r) => (docs = r));
	});

	const vehicle = $derived(app.vehicles.find((v) => v.id === vid));

	function pickSchedule(value: string) {
		scheduleId = value === '' ? null : Number(value);
		const s = schedules.find((x) => x.id === scheduleId);
		if (s) {
			title = s.title;
			category = s.category;
		} else if (!editing) {
			title = '';
		}
	}

	async function save(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (!vid) return (error = 'Scegli un’auto');
		if (!title.trim()) return (error = 'Scrivi cosa è stato fatto');
		const kmValue = parseIntOrNull(km);
		if (km.trim() && kmValue == null) return (error = 'I km devono essere un numero');
		const costValue = parseEuro(cost);
		if (cost.trim() && costValue == null) return (error = 'Il costo deve essere un numero, es. 120,50');
		saving = true;
		const body = {
			vehicle_id: vid,
			schedule_id: scheduleId,
			title,
			category,
			date,
			km: kmValue,
			cost_cents: costValue,
			provider,
			notes
		};
		try {
			const saved = editing
				? await api.put<CarEvent>(`/events/${event!.id}`, body)
				: await api.post<CarEvent>('/events', body);
			for (const f of Array.from(files ?? [])) {
				const fd = new FormData();
				fd.set('file', f);
				fd.set('kind', docKind);
				fd.set('event_id', String(saved.id));
				fd.set('vehicle_id', String(vid));
				fd.set('title', `${docKind === 'fattura' ? 'Fattura' : 'Documento'} ${title}`);
				await api.post('/documents', fd);
			}
			toast(editing ? 'Intervento aggiornato' : 'Intervento registrato');
			refresh();
			closeDialog();
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}

	function remove() {
		deleteWithUndo('event', event!.id, 'Intervento', `/events/${event!.id}`);
		closeDialog();
	}
</script>

<Modal title={editing ? 'Modifica intervento' : schedule ? `${schedule.title}: segna fatto` : 'Registra intervento'} onclose={closeDialog}>
	<form class="form" onsubmit={save} id="event-form">
		{#if !vehicleId && !schedule && !editing}<VehicleSelect bind:value={vid} />{/if}

		{#if !schedule || editing}
			<div class="field">
				<label for="ev-sched">Scadenza collegata</label>
				<select id="ev-sched" class="input" value={scheduleId ?? ''} onchange={(e) => pickSchedule(e.currentTarget.value)}>
					<option value="">Nessuna (intervento libero)</option>
					{#each schedules as s (s.id)}<option value={s.id}>{s.title}</option>{/each}
				</select>
				<span class="hint">Se colleghi una scadenza, la prossima viene ricalcolata da questo intervento.</span>
			</div>
		{/if}

		<div class="form-grid cols-2">
			<div class="field">
				<label for="ev-title">Cosa è stato fatto <span class="req">*</span></label>
				<input id="ev-title" class="input" bind:value={title} required placeholder="Es. Cambio pastiglie" />
			</div>
			<div class="field">
				<label for="ev-cat">Categoria</label>
				<select id="ev-cat" class="input" bind:value={category}>
					{#each categories as c (c)}<option value={c}>{categoryLabel[c]}</option>{/each}
				</select>
			</div>
			<div class="field">
				<label for="ev-date">Data <span class="req">*</span></label>
				<input id="ev-date" class="input" type="date" bind:value={date} required />
			</div>
			<div class="field">
				<label for="ev-km">Km</label>
				<div class="input-affix">
					<input id="ev-km" class="input num" type="text" inputmode="numeric" bind:value={km} placeholder={vehicle?.current_km != null ? String(vehicle.current_km) : ''} />
					<span class="affix">km</span>
				</div>
				{#if vehicle?.current_km != null}<span class="hint">Ultimi registrati: {fmtKm(vehicle.current_km)}</span>{/if}
			</div>
			<div class="field">
				<label for="ev-cost">Costo</label>
				<div class="input-affix">
					<input id="ev-cost" class="input num" type="text" inputmode="decimal" bind:value={cost} placeholder="0,00" />
					<span class="affix">€</span>
				</div>
			</div>
			<div class="field">
				<label for="ev-prov">Dove / chi</label>
				<input id="ev-prov" class="input" bind:value={provider} placeholder="Officina, compagnia…" />
			</div>
		</div>
		<div class="field">
			<label for="ev-notes">Note</label>
			<textarea id="ev-notes" class="input" bind:value={notes}></textarea>
		</div>

		{#if docs.length}
			<div class="field">
				<span class="label">Documenti allegati</span>
				<ul class="list card" style="box-shadow:none">
					{#each docs as d (d.id)}
						<li>
							<span class="tile-ico"><Icon name="file" size={18} /></span>
							<div class="grow"><div class="t">{d.title}</div><div class="s">{fmtBytes(d.size)}</div></div>
							<a class="icon-btn ghost" href="/api/documents/{d.id}/file" target="_blank" rel="noopener" aria-label="Apri {d.title}"><Icon name="external" size={18} /></a>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
		<div class="form-grid cols-2">
			<div class="field">
				<label for="ev-files">Allega file (fattura, foto…)</label>
				<input id="ev-files" class="input" type="file" multiple accept="application/pdf,image/*" bind:files />
			</div>
			<div class="field">
				<label for="ev-kind">Tipo di file</label>
				<select id="ev-kind" class="input" bind:value={docKind}>
					<option value="fattura">Fattura</option>
					<option value="polizza">Polizza</option>
					<option value="foto">Foto</option>
					<option value="altro">Altro</option>
				</select>
			</div>
		</div>
		{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
	</form>
	{#snippet footer()}
		<div class="form-actions">
			{#if editing}
				<button type="button" class="btn btn-danger" onclick={remove}><Icon name="trash" size={18} />Elimina</button>
				<span class="spacer"></span>
			{/if}
			<button type="button" class="btn" onclick={closeDialog}>Annulla</button>
			<button type="submit" form="event-form" class="btn btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
		</div>
	{/snippet}
</Modal>
