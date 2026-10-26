<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Icon from '#lib/components/Icon.svelte';
	import VehicleFields, { type VehicleFormData } from '#lib/components/VehicleFields.svelte';
	import { api } from '#lib/api.ts';
	import { parseIntOrNull } from '#lib/format.ts';
	import { refresh, toast } from '#lib/state.svelte.ts';
	import type { VehicleDetail } from '#lib/types.ts';

	const id = $derived(Number(page.params.id));
	let data = $state<VehicleFormData | null>(null);
	let archived = false;
	let error = $state('');
	let saving = $state(false);

	$effect(() => {
		api.get<VehicleDetail>(`/vehicles/${id}`)
			.then((v) => {
				archived = v.archived;
				data = {
					name: v.name,
					make: v.make ?? '',
					model: v.model ?? '',
					plate: v.plate ?? '',
					vin: v.vin ?? '',
					year: v.year != null ? String(v.year) : '',
					registration_date: v.registration_date ?? '',
					purchase_date: v.purchase_date ?? '',
					purchased_used: v.purchased_used,
					color: v.color,
					notes: v.notes ?? ''
				};
			})
			.catch((e) => (error = e.message));
	});

	async function save(e: SubmitEvent) {
		e.preventDefault();
		if (!data) return;
		error = '';
		if (!data.name.trim()) return (error = 'Dai un nome all’auto');
		saving = true;
		try {
			await api.put(`/vehicles/${id}`, {
				...data,
				year: parseIntOrNull(data.year),
				registration_date: data.registration_date || null,
				purchase_date: data.purchase_date || null,
				archived
			});
			toast('Dati aggiornati');
			refresh();
			await goto(`/auto/${id}`);
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head><title>Modifica auto · MyGarage</title></svelte:head>

<a class="back" href="/auto/{id}"><Icon name="chevron-left" />Indietro</a>
<div class="page-head"><h1>Modifica {data?.name ?? 'auto'}</h1></div>

{#if data}
	<form class="card form wrap" onsubmit={save}>
		<VehicleFields bind:data />
		{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
		<div class="form-actions">
			<a class="btn" href="/auto/{id}">Annulla</a>
			<button class="btn btn-primary" type="submit" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
		</div>
	</form>
{:else if error}
	<div class="alert danger" role="alert">{error}</div>
{:else}
	<div class="skeleton" style="height:400px"></div>
{/if}

<style>
	.back {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--fg-muted);
		font-weight: 600;
		min-height: 44px;
	}
	.wrap {
		padding: 20px;
		max-width: 760px;
	}
</style>
