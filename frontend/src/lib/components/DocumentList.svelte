<script lang="ts">
	import Icon from './Icon.svelte';
	import { documentKinds, fmtBytes, fmtDate } from '#lib/format.ts';
	import { openDialog } from '#lib/state.svelte.ts';
	import type { DocumentFile } from '#lib/types.ts';

	let { docs }: { docs: DocumentFile[] } = $props();
</script>

<div class="card">
	{#if docs.length}
		<ul class="list">
			{#each docs as d (d.id)}
				<li>
					<span class="tile-ico"><Icon name={d.mime.startsWith('image/') ? 'image' : 'file'} size={18} /></span>
					<a class="grow info" href="/api/documents/{d.id}/file" target="_blank" rel="noopener" data-sveltekit-reload>
						<span class="t">{d.title}</span>
						<span class="s">{documentKinds[d.kind] ?? d.kind} · {fmtBytes(d.size)} · {fmtDate(d.uploaded_at.slice(0, 10))}{d.event_id ? ' · da un intervento' : ''}</span>
					</a>
					<a class="icon-btn ghost" href="/api/documents/{d.id}/file?download=true" download data-sveltekit-reload aria-label="Scarica {d.title}">
						<Icon name="download" size={18} />
					</a>
					<button class="icon-btn ghost" type="button" aria-label="Modifica {d.title}" onclick={() => openDialog({ kind: 'document', doc: d })}>
						<Icon name="pencil" size={18} />
					</button>
				</li>
			{/each}
		</ul>
	{:else}
		<div class="empty">
			<p>Nessun documento. Carica libretto, polizze e fatture: li ritrovi qui anche dal telefono.</p>
		</div>
	{/if}
</div>

<style>
	.info {
		display: grid;
		min-height: 44px;
		align-content: center;
	}
</style>
