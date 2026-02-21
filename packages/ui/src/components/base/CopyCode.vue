<template>
	<button class="code" :class="{ copied }" :title="formatMessage(copiedMessage)" @click="copyText">
		<span>{{ text }}</span>
		<CheckIcon v-if="copied" />
		<ClipboardCopyIcon v-else />
	</button>
</template>

<script setup lang="ts">
import { CheckIcon, ClipboardCopyIcon } from '@modrinth/assets'
import { defineMessage, useVIntl } from '@vintl/vintl'
import { ref } from 'vue'

const copiedMessage = defineMessage({
	id: 'omorphia.component.copy.action.copy',
	defaultMessage: 'Copy code to clipboard',
})
const { formatMessage } = useVIntl()

const props = defineProps<{ text: string }>()

const copied = ref(false)

async function copyText() {
	await navigator.clipboard.writeText(props.text)
	copied.value = true
}
</script>

<style lang="scss" scoped>
.code {
	color: var(--color-text);
	display: inline-flex;
	grid-gap: 0.5rem;
	font-family: var(--mono-font);
	font-size: var(--font-size-sm);
	margin: 0;
	padding: 0.25rem 0.5rem;
	background-color: var(--color-button-bg);
	border: 1px solid var(--color-button-border);
	box-shadow: var(--shadow-inset-sm);
	width: fit-content;
	border-radius: 10px;
	user-select: text;
	transition:
		opacity 0.5s ease-in-out,
		transform 0.05s ease-in-out,
		outline 0.2s ease-in-out;

	@media (prefers-reduced-motion) {
		transition: none !important;
	}

	svg {
		width: 1em;
		height: 1em;
	}

	&:hover {
		background-color: var(--color-button-bg-hover);
		color: var(--color-button-text-hover);
	}

	&:focus-visible {
		outline: none;
		box-shadow:
			var(--shadow-inset-sm),
			0 0 0 0.2rem var(--color-brand-shadow);
	}

	&:active {
		transform: scale(0.97);
		background-color: var(--color-button-bg-active);
		color: var(--color-button-text-active);
	}
}
</style>
