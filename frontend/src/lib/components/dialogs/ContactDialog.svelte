<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import Icon from '../Icon.svelte';
	import VehicleSelect from './VehicleSelect.svelte';
	import { api } from '#lib/api.ts';
	import { contactCategories } from '#lib/format.ts';
	import { closeDialog, deleteWithUndo, refresh, toast } from '#lib/state.svelte.ts';
	import type { Contact } from '#lib/types.ts';

	const props: { vehicleId?: number | null; contact?: Contact } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { vehicleId, contact } = untrack(() => props);

	const editing = !!contact;
	let vid = $state<number | null>(contact ? contact.vehicle_id : (vehicleId ?? null));
	let name = $state(contact?.name ?? '');
	let category = $state(contact?.category ?? 'assicurazione');
	let phone = $state(contact?.phone ?? '');
	let email = $state(contact?.email ?? '');
	let reference = $state(contact?.reference ?? '');
	let notes = $state(contact?.notes ?? '');
	let error = $state('');
	let saving = $state(false);

	async function save(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (!name.trim()) return (error = 'Il nome è obbligatorio');
		saving = true;
		const body = { vehicle_id: vid, name, category, phone, email, reference, notes };
		try {
			if (editing) await api.put(`/contacts/${contact!.id}`, body);
			else await api.post('/contacts', body);
			toast(editing ? 'Contatto aggiornato' : 'Contatto aggiunto');
			refresh();
			closeDialog();
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}

	function remove() {
		deleteWithUndo('contact', contact!.id, 'Contatto', `/contacts/${contact!.id}`);
		closeDialog();
	}
</script>

<Modal title={editing ? 'Modifica contatto' : 'Nuovo numero utile'} onclose={closeDialog}>
	<form class="form" onsubmit={save} id="contact-form">
		<div class="form-grid cols-2">
			<div class="field">
				<label for="ct-name">Nome <span class="req">*</span></label>
				<input id="ct-name" class="input" bind:value={name} required placeholder="Es. Soccorso stradale" />
			</div>
			<div class="field">
				<label for="ct-cat">Tipo</label>
				<select id="ct-cat" class="input" bind:value={category}>
					{#each Object.entries(contactCategories) as [k, label] (k)}<option value={k}>{label}</option>{/each}
				</select>
			</div>
			<div class="field">
				<label for="ct-phone">Telefono</label>
				<input id="ct-phone" class="input num" type="tel" autocomplete="tel" bind:value={phone} />
			</div>
			<div class="field">
				<label for="ct-email">Email</label>
				<input id="ct-email" class="input" type="email" autocomplete="email" bind:value={email} />
			</div>
		</div>
		<div class="field">
			<label for="ct-ref">Riferimento</label>
			<input id="ct-ref" class="input" bind:value={reference} placeholder="N. polizza, n. tessera, codice cliente…" />
		</div>
		<VehicleSelect bind:value={vid} label="Vale per" allowNone />
		<div class="field">
			<label for="ct-notes">Note</label>
			<textarea id="ct-notes" class="input" bind:value={notes}></textarea>
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
			<button type="submit" form="contact-form" class="btn btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
		</div>
	{/snippet}
</Modal>
