<script lang="ts">
	import Icon from './Icon.svelte';
	import { contactCategories } from '#lib/format.ts';
	import { openDialog } from '#lib/state.svelte.ts';
	import type { Contact } from '#lib/types.ts';

	let { contacts, vehicleId, compact = false }: { contacts: Contact[]; vehicleId?: number | null; compact?: boolean } = $props();

	const icon: Record<string, string> = {
		assicurazione: 'shield',
		soccorso: 'truck',
		officina: 'wrench',
		gommista: 'tyre',
		carrozzeria: 'car',
		concessionario: 'car',
		altro: 'phone'
	};
</script>

<div class="card">
	{#if contacts.length}
		<ul class="list">
			{#each contacts as c (c.id)}
				<li>
					<span class="tile-ico" class:urgent={c.category === 'soccorso'}><Icon name={icon[c.category] ?? 'phone'} size={18} /></span>
					<button class="grow info" type="button" onclick={() => openDialog({ kind: 'contact', vehicleId, contact: c })}>
						<span class="t">{c.name}</span>
						<span class="s num">
							{contactCategories[c.category] ?? c.category}{c.vehicle_id == null && vehicleId ? ' · tutte le auto' : ''}{c.reference ? ` · ${c.reference}` : ''}{!compact && c.phone ? ` · ${c.phone}` : ''}
						</span>
					</button>
					{#if c.email && !compact}
						<a class="icon-btn" href="mailto:{c.email}" aria-label="Scrivi a {c.name}"><Icon name="mail" size={18} /></a>
					{/if}
					{#if c.phone}
						<a class="call" href="tel:{c.phone.replace(/[^\d+]/g, '')}" aria-label="Chiama {c.name}"><Icon name="phone" size={18} /></a>
					{/if}
				</li>
			{/each}
		</ul>
	{:else}
		<div class="empty">
			<p>Salva qui i numeri da avere a portata di mano: assicurazione, soccorso stradale, officina.</p>
			<button class="btn btn-sm" type="button" onclick={() => openDialog({ kind: 'contact', vehicleId })}><Icon name="plus" size={18} />Aggiungi numero</button>
		</div>
	{/if}
</div>

<style>
	.info {
		display: grid;
		text-align: left;
		background: transparent;
		border: 0;
		padding: 4px 0;
		cursor: pointer;
		min-height: 44px;
		align-content: center;
	}
	.urgent {
		background: var(--danger-soft);
		color: var(--danger);
	}
	.call {
		width: 44px;
		height: 44px;
		border-radius: 12px;
		display: grid;
		place-items: center;
		background: var(--ok-soft);
		color: var(--ok);
		flex: none;
	}
</style>
