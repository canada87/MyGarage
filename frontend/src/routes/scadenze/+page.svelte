<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Icon from '#lib/components/Icon.svelte';
	import DueRow from '#lib/components/DueRow.svelte';
	import { api } from '#lib/api.ts';
	import { statusLabel } from '#lib/format.ts';
	import { app, isPending } from '#lib/state.svelte.ts';
	import type { ScheduleView, Status } from '#lib/types.ts';

	let all = $state<ScheduleView[] | null>(null);
	let error = $state('');

	const status = $derived((page.url.searchParams.get('stato') ?? '') as Status | '');
	const vehicle = $derived(Number(page.url.searchParams.get('auto')) || null);

	$effect(() => {
		void app.version;
		api.get<ScheduleView[]>('/schedules')
			.then((r) => (all = r))
			.catch((e) => (error = e.message));
	});

	function setFilter(key: string, value: string | number | null) {
		const url = new URL(page.url.href);
		if (value === null || value === '') url.searchParams.delete(key);
		else url.searchParams.set(key, String(value));
		goto(url, { replace: true, reset: false });
	}

	const filtered = $derived(
		(all ?? []).filter(
			(s) => !isPending('schedule', s.id) && (!status || s.status === status) && (!vehicle || s.vehicle_id === vehicle)
		)
	);
	const groups: Status[] = ['overdue', 'soon', 'setup', 'ok'];
	const counts = $derived(
		Object.fromEntries(groups.map((g) => [g, (all ?? []).filter((s) => s.status === g && (!vehicle || s.vehicle_id === vehicle)).length]))
	);
</script>

<svelte:head><title>Scadenze · MyGarage</title></svelte:head>

<div class="page-head">
	<div>
		<p class="eyebrow">Tutte le auto</p>
		<h1>Scadenze</h1>
	</div>
</div>

<div class="filters" role="group" aria-label="Filtra per stato">
	<button class="chip" type="button" aria-pressed={!status} onclick={() => setFilter('stato', null)}>Tutte</button>
	{#each groups as g (g)}
		<button class="chip" type="button" aria-pressed={status === g} onclick={() => setFilter('stato', g)}>
			{statusLabel[g]} <span class="num count">{counts[g] ?? 0}</span>
		</button>
	{/each}
</div>
{#if app.vehicles.length > 1}
	<div class="filters" role="group" aria-label="Filtra per auto">
		<button class="chip" type="button" aria-pressed={!vehicle} onclick={() => setFilter('auto', null)}>Tutte le auto</button>
		{#each app.vehicles as v (v.id)}
			<button class="chip" type="button" aria-pressed={vehicle === v.id} onclick={() => setFilter('auto', v.id)}>
				<span class="dot" style="background:{v.color}"></span>{v.name}
			</button>
		{/each}
	</div>
{/if}

{#if error}
	<div class="alert danger" role="alert">{error}</div>
{:else if !all}
	<div class="skeleton" style="height:320px"></div>
{:else if filtered.length === 0}
	<div class="card empty">
		<span class="tile-ico"><Icon name="ok" size={26} /></span>
		<p>{status ? `Nessuna scadenza “${statusLabel[status].toLowerCase()}”.` : 'Nessuna scadenza. Aggiungile dalla pagina di un’auto.'}</p>
	</div>
{:else}
	{#each groups.filter((g) => filtered.some((s) => s.status === g)) as g (g)}
		<section class="section">
			<h2 class="group">{statusLabel[g]}</h2>
			<div class="card">
				<ul class="list rows">
					{#each filtered.filter((s) => s.status === g) as s (s.id)}<DueRow {s} showVehicle={!vehicle} />{/each}
				</ul>
			</div>
		</section>
	{/each}
{/if}

<style>
	.filters {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		margin-bottom: 10px;
	}
	.count {
		opacity: 0.7;
	}
	.group {
		margin-bottom: 10px;
	}
	.rows :global(li) {
		padding: 0;
		display: block;
	}
</style>
