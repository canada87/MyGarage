<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import Icon from '../Icon.svelte';
	import VehicleSelect from './VehicleSelect.svelte';
	import { api } from '#lib/api.ts';
	import { documentKinds } from '#lib/format.ts';
	import { app, closeDialog, deleteWithUndo, refresh, toast } from '#lib/state.svelte.ts';
	import type { DocumentFile } from '#lib/types.ts';

	const props: { vehicleId?: number; eventId?: number; doc?: DocumentFile } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { vehicleId, eventId, doc } = untrack(() => props);

	const editing = !!doc;
	let vid = $state<number | null>(doc ? doc.vehicle_id : (vehicleId ?? app.vehicles[0]?.id ?? null));
	let title = $state(doc?.title ?? '');
	let kind = $state<string>(doc?.kind ?? 'libretto');
	let files = $state<FileList | null>(null);
	let error = $state('');
	let saving = $state(false);

	async function save(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		saving = true;
		try {
			if (editing) {
				if (!title.trim()) throw new Error('Il titolo è obbligatorio');
				await api.put(`/documents/${doc!.id}`, { title, kind, vehicle_id: vid, event_id: doc!.event_id });
				toast('Documento aggiornato');
			} else {
				const list = Array.from(files ?? []);
				if (!list.length) throw new Error('Scegli almeno un file');
				for (const f of list) {
					const fd = new FormData();
					fd.set('file', f);
					fd.set('kind', kind);
					if (vid) fd.set('vehicle_id', String(vid));
					if (eventId) fd.set('event_id', String(eventId));
					if (title.trim()) fd.set('title', list.length > 1 ? `${title} (${f.name})` : title);
					await api.post('/documents', fd);
				}
				toast(list.length > 1 ? `${list.length} documenti caricati` : 'Documento caricato');
			}
			refresh();
			closeDialog();
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}

	function remove() {
		deleteWithUndo('document', doc!.id, 'Documento', `/documents/${doc!.id}`);
		closeDialog();
	}
</script>

<Modal title={editing ? 'Modifica documento' : 'Carica documento'} onclose={closeDialog}>
	<form class="form" onsubmit={save} id="doc-form">
		{#if !editing}
			<div class="field">
				<label for="doc-file">File <span class="req">*</span></label>
				<input id="doc-file" class="input" type="file" multiple accept="application/pdf,image/*" bind:files required />
				<span class="hint">PDF o foto, anche più file insieme. Dal telefono puoi scattare una foto.</span>
			</div>
		{/if}
		{#if !vehicleId || editing}<VehicleSelect bind:value={vid} allowNone />{/if}
		<div class="form-grid cols-2">
			<div class="field">
				<label for="doc-title">Titolo</label>
				<input id="doc-title" class="input" bind:value={title} placeholder={editing ? '' : 'Dal nome del file'} />
			</div>
			<div class="field">
				<label for="doc-kind">Tipo</label>
				<select id="doc-kind" class="input" bind:value={kind}>
					{#each Object.entries(documentKinds) as [k, label] (k)}<option value={k}>{label}</option>{/each}
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
			<button type="submit" form="doc-form" class="btn btn-primary" disabled={saving}>
				{saving ? 'Caricamento…' : editing ? 'Salva' : 'Carica'}
			</button>
		</div>
	{/snippet}
</Modal>
