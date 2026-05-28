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
	{
		value: 'de',
		label: formatMessage(messages.languageGermanGermany),
		nativeLabel: 'Deutsch (DE)',
	},
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
	gap: 0.45rem;
	margin-top: 0.35rem;
}

.language-item {
	position: relative;
	display: flex;
	width: 100%;
	align-items: center;
	gap: 0.72rem;
	text-align: left;
	min-height: 3.55rem;
	padding: 0.72rem 0.85rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 64%, transparent);
	border-radius: 0.78rem;
	background: color-mix(in srgb, var(--color-button-bg) 34%, transparent);
	box-shadow: none;
	color: var(--color-base);
	cursor: pointer;
	transition:
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
		background 180ms cubic-bezier(0.22, 1, 0.36, 1),
		color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.language-item:hover {
	transform: translateY(-1px);
	border-color: color-mix(in srgb, var(--color-brand) 16%, var(--glass-border) 84%);
	background: color-mix(in srgb, var(--color-button-bg-hover) 38%, transparent);
	color: var(--color-contrast);
}

.language-item.active {
	border-color: color-mix(in srgb, var(--color-brand) 30%, var(--glass-border) 70%);
	background: color-mix(in srgb, var(--color-button-bg-selected) 24%, var(--color-button-bg) 76%);
	color: var(--color-contrast);
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-brand) 12%, transparent);
}

.radio {
	width: 1rem;
	height: 1rem;
	flex-shrink: 0;
	color: var(--color-secondary);
	transition:
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
		color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.language-item.active .radio {
	color: var(--color-brand);
	transform: scale(1.04);
}

.language-names {
	display: flex;
	flex-direction: column;
	gap: 0.08rem;
	min-width: 0;
}

.language-name {
	overflow: hidden;
	color: var(--color-contrast);
	font-weight: 700;
	line-height: 1.2;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.language-native {
	color: var(--color-secondary);
	font-size: 0.8rem;
	line-height: 1.2;
}

.languages-note {
	margin: 0.75rem 0 0;
	color: var(--color-secondary);
	font-size: 0.82rem;
	line-height: 1.35;
}
</style>
