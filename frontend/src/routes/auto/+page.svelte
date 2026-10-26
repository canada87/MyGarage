<script lang="ts">
	import Icon from '#lib/components/Icon.svelte';
	import VehicleCard from '#lib/components/VehicleCard.svelte';
	import { api } from '#lib/api.ts';
	import { app } from '#lib/state.svelte.ts';
	import type { VehicleSummary } from '#lib/types.ts';

	let showArchived = $state(false);
	let archived = $state<VehicleSummary[]>([]);

	$effect(() => {
		void app.version;
		if (showArchived) {
			api.get<VehicleSummary[]>('/vehicles?archived=true').then((all) => (archived = all.filter((v) => v.archived)));
		}
	});
</script>

<svelte:head><title>Auto · MyGarage</title></svelte:head>

<div class="page-head">
	<div>
		<p class="eyebrow">{app.vehicles.length} {app.vehicles.length === 1 ? 'auto' : 'auto'}</p>
		<h1>Le tue auto</h1>
	</div>
	<a class="btn btn-primary" href="/auto/nuova"><Icon name="plus" />Aggiungi auto</a>
</div>

{#if !app.vehiclesLoaded}
	<div class="cars">
		<div class="skeleton" style="height:240px"></div>
		<div class="skeleton" style="height:240px"></div>
	</div>
{:else if app.vehicles.length === 0}
	<div class="card empty">
		<span class="tile-ico"><Icon name="car" size={28} /></span>
		<h2>Nessuna auto</h2>
		<p>Aggiungi un’auto, nuova o usata: bastano il nome e i km attuali.</p>
		<a class="btn btn-primary" href="/auto/nuova"><Icon name="plus" />Aggiungi auto</a>
	</div>
{:else}
	<div class="cars">
		{#each app.vehicles as v (v.id)}<VehicleCard {v} />{/each}
	</div>
{/if}

<div class="section">
	<label class="check"><input type="checkbox" bind:checked={showArchived} />Mostra auto archiviate</label>
	{#if showArchived}
		{#if archived.length}
			<div class="cars">{#each archived as v (v.id)}<VehicleCard {v} />{/each}</div>
		{:else}
			<p class="subtle">Nessuna auto archiviata. Archivia un’auto venduta per tenerne lo storico senza vederla in dashboard.</p>
		{/if}
	{/if}
</div>
