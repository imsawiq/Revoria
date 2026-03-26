<template>
	<button
		class="big-option-button group flex w-full hover:cursor-pointer items-center gap-3 rounded-[20px] p-3 text-left transition-all active:scale-[0.98] border-none"
		:data-selected="selected ? 'true' : 'false'"
		@click="$emit('click')"
	>
		<div class="big-option-button__icon-shell flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl border border-solid">
			<component
				:is="icon"
				class="big-option-button__icon size-8 text-secondary"
				stroke-width="1.5"
			/>
		</div>
		<div class="flex flex-1 flex-col gap-1">
			<span class="text-base font-semibold text-contrast">{{ title }}</span>
			<span class="text-sm font-medium text-primary">{{ description }}</span>
		</div>
		<ChevronRightIcon
			class="size-5 shrink-0 text-secondary opacity-0 transition-opacity duration-100 group-hover:opacity-100"
		/>
	</button>
</template>

<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import type { Component } from 'vue'

defineProps<{
	icon: Component
	title: string
	description: string
	selected?: boolean
}>()

defineEmits<{
	(e: 'click'): void
}>()
</script>

<style scoped lang="scss">
.big-option-button {
	background: color-mix(in srgb, var(--color-glass-bg) 92%, transparent);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));

	&:hover {
		background: color-mix(in srgb, var(--color-glass-bg-strong) 88%, transparent);
		border-color: color-mix(in srgb, var(--color-brand) 28%, var(--glass-border));
		transform: translateY(-1px);
	}

	&[data-selected='true'] {
		background:
			linear-gradient(
				135deg,
				color-mix(in srgb, var(--color-brand-highlight) 84%, transparent),
				color-mix(in srgb, var(--color-glass-bg-strong) 92%, transparent)
			);
		border-color: color-mix(in srgb, var(--color-brand) 44%, var(--glass-border));
	}
}

.big-option-button__icon-shell {
	border-color: var(--glass-border);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 84%, transparent);
	transition:
		border-color 120ms ease,
		background-color 120ms ease,
		transform 120ms ease;
}

.big-option-button__icon {
	transition: color 120ms ease, stroke 120ms ease, transform 120ms ease;
}

.big-option-button:hover .big-option-button__icon-shell,
.big-option-button[data-selected='true'] .big-option-button__icon-shell {
	border-color: color-mix(in srgb, var(--color-brand) 36%, var(--glass-border));
	background: color-mix(in srgb, var(--color-brand-highlight) 22%, var(--color-glass-bg-strong));
}

.big-option-button[data-selected='true'] .big-option-button__icon {
	color: var(--color-brand);
	stroke: var(--color-brand);
}

:global(.light-mode),
:global(.rose-gold-mode),
:global(.cherry-blossom-mode) {
	.big-option-button {
		color: var(--color-base);
	}

	.big-option-button .text-contrast {
		color: var(--color-contrast) !important;
	}

	.big-option-button .text-primary,
	.big-option-button .text-secondary,
	.big-option-button__icon,
	.big-option-button .size-5 {
		color: var(--color-secondary) !important;
		stroke: currentColor;
	}
}
</style>
