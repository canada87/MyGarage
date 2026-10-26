<script lang="ts">
	import Icon from '#lib/components/Icon.svelte';
	import ContactList from '#lib/components/ContactList.svelte';
	import { api } from '#lib/api.ts';
	import { app, isPending, openDialog } from '#lib/state.svelte.ts';
	import type { Contact } from '#lib/types.ts';

	let contacts = $state<Contact[] | null>(null);
	let error = $state('');

	$effect(() => {
		void app.version;
		api.get<Contact[]>('/contacts')
			.then((c) => (contacts = c))
			.catch((e) => (error = e.message));
	});

	const visible = $derived((contacts ?? []).filter((c) => !isPending('contact', c.id)));
	const general = $derived(visible.filter((c) => c.vehicle_id == null));
</script>

<svelte:head><title>Rubrica · MyGarage</title></svelte:head>

<div class="page-head">
	<div>
		<p class="eyebrow">Da avere a portata di mano</p>
		<h1>Numeri utili</h1>
	</div>
	<button class="btn btn-primary" type="button" onclick={() => openDialog({ kind: 'contact' })}><Icon name="plus" />Aggiungi</button>
</div>

{#if error}
	<div class="alert danger" role="alert">{error}</div>
{:else if !contacts}
	<div class="skeleton" style="height:240px"></div>
{:else}
	<section class="section" aria-labelledby="h-gen">
		<div class="section-head"><h2 id="h-gen">Per tutte le auto</h2></div>
		<ContactList contacts={general} vehicleId={null} />
	</section>
	{#each app.vehicles as v (v.id)}
		<section class="section" aria-labelledby="h-v{v.id}">
			<div class="section-head">
				<h2 id="h-v{v.id}" class="row"><span class="dot" style="background:{v.color}"></span>{v.name}</h2>
				<button class="link" type="button" onclick={() => openDialog({ kind: 'contact', vehicleId: v.id })}><Icon name="plus" size={16} />Aggiungi</button>
			</div>
			<ContactList contacts={visible.filter((c) => c.vehicle_id === v.id)} vehicleId={v.id} />
		</section>
	{/each}
{/if}
