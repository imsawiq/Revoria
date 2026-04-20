<script setup lang="ts">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import { computed } from 'vue'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	launcherLanguageTitle: {
		id: 'settings.appearance.launcher-language.title',
		defaultMessage: 'Launcher language',
	},
	launcherLanguageDescription: {
		id: 'settings.appearance.launcher-language.description',
		defaultMessage: 'Select the language used by Revoria UI.',
	},
	languageNote: {
		id: 'settings.language.note',
		defaultMessage: 'More languages are on the way.',
	},
	languageEnglish: {
		id: 'settings.appearance.launcher-language.option.english',
		defaultMessage: 'English',
	},
	languageRussian: {
		id: 'settings.appearance.launcher-language.option.russian',
		defaultMessage: 'Russian',
	},
	languageUkrainian: {
		id: 'settings.appearance.launcher-language.option.ukrainian',
		defaultMessage: 'Ukrainian',
	},
	languageGermanGermany: {
		id: 'settings.appearance.launcher-language.option.german-germany',
		defaultMessage: 'German (Germany)',
	},
	languageRomanian: {
		id: 'settings.appearance.launcher-language.option.romanian',
		defaultMessage: 'Romanian',
	},
})

const launcherLanguage = useStorage('launcher-language', 'en')
const languageOptions = computed(() => [
	{ value: 'en', label: formatMessage(messages.languageEnglish), nativeLabel: 'English' },
	{ value: 'ru', label: formatMessage(messages.languageRussian), nativeLabel: 'Русский' },
	{ value: 'uk', label: formatMessage(messages.languageUkrainian), nativeLabel: 'Українська' },
	{ value: 'de', label: formatMessage(messages.languageGermanGermany), nativeLabel: 'Deutsch (DE)' },
	{ value: 'ro', label: formatMessage(messages.languageRomanian), nativeLabel: 'Română' },
])

function selectLanguage(value: string) {
	launcherLanguage.value = value
}
</script>

<template>
	<div class="languages-list">
		<button
			v-for="option in languageOptions"
			:key="option.value"
			type="button"
			class="language-item"
			:class="{ active: launcherLanguage === option.value }"
			:aria-pressed="launcherLanguage === option.value"
			:aria-label="option.label"
			@click="selectLanguage(option.value)"
		>
			<component
				:is="launcherLanguage === option.value ? RadioButtonCheckedIcon : RadioButtonIcon"
				class="radio"
			/>
			<div class="language-names">
				<div class="language-name">{{ option.label }}</div>
				<div class="language-native">{{ option.nativeLabel }}</div>
			</div>
		</button>
	</div>
	<p class="languages-note mt-3">{{ formatMessage(messages.languageNote) }}</p>
</template>

<style lang="scss" scoped>
.languages-list {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
}

.language-item {
	display: flex;
	width: 100%;
	align-items: center;
	gap: 0.75rem;
	text-align: left;
	padding: 1rem 1.125rem;
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-xl);
	background: linear-gradient(
		180deg,
		color-mix(in oklch, var(--color-glass-bg-strong) 88%, transparent),
		color-mix(in oklch, var(--color-glass-bg) 94%, transparent)
	);
	box-shadow: var(--shadow-card);
	cursor: pointer;
	transition:
		border-color 200ms ease,
		background 200ms ease,
		box-shadow 200ms ease,
		color 200ms ease,
		transform 200ms ease;
}

.language-item:hover {
	border-color: color-mix(in oklch, var(--color-brand) 45%, var(--glass-border));
	transform: translateY(-1px);
}

.language-item.active {
	border-color: color-mix(in oklch, var(--color-brand) 70%, var(--glass-border));
	background: linear-gradient(
		180deg,
		color-mix(in oklch, var(--color-brand-highlight) 22%, var(--color-glass-bg-strong)),
		color-mix(in oklch, var(--color-brand-highlight) 16%, var(--color-glass-bg))
	);
	box-shadow:
		var(--shadow-card),
		0 0 0 1px color-mix(in oklch, var(--color-brand) 35%, transparent);
}

.radio {
	width: 1.1rem;
	height: 1.1rem;
	flex-shrink: 0;
	transition: transform 200ms ease, opacity 200ms ease;
}

.language-names {
	display: flex;
	flex-direction: column;
	gap: 0.1rem;
}

.language-name {
	font-weight: 700;
	color: var(--color-contrast);
}

.language-native {
	font-size: 0.85rem;
	color: var(--color-secondary);
}

.languages-note {
	color: var(--color-secondary);
	font-size: 0.85rem;
}
</style>
