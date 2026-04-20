<script setup lang="ts" generic="T extends string">
import { MoonIcon, RadioButtonCheckedIcon, RadioButtonIcon, SunIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const { updateColorTheme, currentTheme, themeOptions, systemThemeColor } = defineProps<{
	updateColorTheme: (theme: T) => void
	currentTheme: T
	themeOptions: readonly T[]
	systemThemeColor: T
}>()

const colorTheme = defineMessages({
	title: {
		id: 'settings.display.theme.title',
		defaultMessage: 'Color theme',
	},
	description: {
		id: 'settings.display.theme.description',
		defaultMessage: 'Select your preferred color theme for Modrinth on this device.',
	},
	system: {
		id: 'settings.display.theme.system',
		defaultMessage: 'Sync with system',
	},
	light: {
		id: 'settings.display.theme.light',
		defaultMessage: 'Light',
	},
	dark: {
		id: 'settings.display.theme.dark',
		defaultMessage: 'Dark',
	},
	oled: {
		id: 'settings.display.theme.oled',
		defaultMessage: 'OLED',
	},
	retro: {
		id: 'settings.display.theme.retro',
		defaultMessage: 'Retro',
	},
	sapphire: {
		id: 'settings.display.theme.sapphire',
		defaultMessage: 'Sapphire',
	},
	amethyst: {
		id: 'settings.display.theme.amethyst',
		defaultMessage: 'Amethyst',
	},
	sunset: {
		id: 'settings.display.theme.sunset',
		defaultMessage: 'Sunset',
	},
	aurora: {
		id: 'settings.display.theme.aurora',
		defaultMessage: 'Aurora',
	},
	nord: {
		id: 'settings.display.theme.nord',
		defaultMessage: 'Nord',
	},
	'cherry-cola': {
		id: 'settings.display.theme.cherry-cola',
		defaultMessage: 'Cherry Cola',
	},
	slate: {
		id: 'settings.display.theme.slate',
		defaultMessage: 'Slate',
	},
	'rose-gold': {
		id: 'settings.display.theme.rose-gold',
		defaultMessage: 'Rose Gold',
	},
	'obsidian-gold': {
		id: 'settings.display.theme.obsidian-gold',
		defaultMessage: 'Obsidian Gold',
	},
	'cherry-blossom': {
		id: 'settings.display.theme.cherry-blossom',
		defaultMessage: 'Cherry Blossom',
	},
	preferredLight: {
		id: 'settings.display.theme.preferred-light-theme',
		defaultMessage: 'Preferred light theme',
	},
	preferredDark: {
		id: 'settings.display.theme.preferred-dark-theme',
		defaultMessage: 'Preferred dark theme',
	},
})

function asString(theme: T): string {
	return theme
}

const supportedThemeOptions = themeOptions as readonly T[]

function getPreviewClass(option: T): string {
	const base = option === 'system' ? systemThemeColor : option
	return base.endsWith('-mode') ? base : `${base}-mode`
}
</script>

<template>
	<div class="theme-options mt-4">
		<button
			v-for="option in supportedThemeOptions"
			:key="option"
			class="preview-radio button-base"
			:class="{ selected: currentTheme === option }"
			@click="() => updateColorTheme(option)"
		>
			<div class="preview" :class="getPreviewClass(option)">
				<div class="example-card card card">
					<div class="example-icon"></div>
					<div class="example-text-1"></div>
					<div class="example-text-2"></div>
				</div>
			</div>
			<div class="label">
				<RadioButtonCheckedIcon v-if="currentTheme === option" class="radio shrink-0" />
				<RadioButtonIcon v-else class="radio shrink-0" />
				{{ colorTheme[asString(option)] ? formatMessage(colorTheme[asString(option)]) : option }}
				<SunIcon
					v-if="'light' === option"
					v-tooltip="formatMessage(colorTheme.preferredLight)"
					class="theme-icon shrink-0"
				/>
				<MoonIcon
					v-else-if="'dark' === option"
					v-tooltip="formatMessage(colorTheme.preferredDark)"
					class="theme-icon shrink-0"
				/>
			</div>
		</button>
	</div>
</template>

<style scoped lang="scss">
.theme-options {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
	gap: var(--gap-lg);

	.preview {
		&.light-mode {
			@extend .light-mode;
		}

		&.dark-mode {
			@extend .dark-mode;
		}
	}

	.preview .example-card {
		margin: 0;
		padding: 1rem;
		display: grid;
		grid-template: 'icon text1' 'icon text2';
		grid-template-columns: auto 1fr;
		gap: 0.5rem;
		outline: 2px solid transparent;

		.example-icon {
			grid-area: icon;
			width: 2rem;
			height: 2rem;
			background-color: var(--color-button-bg);
			border-radius: var(--radius-sm);
			outline: 2px solid transparent;
		}

		.example-text-1,
		.example-text-2 {
			height: 0.5rem;
			border-radius: var(--radius-sm);
			outline: 2px solid transparent;
		}

		.example-text-1 {
			grid-area: text1;
			width: 100%;
			background-color: var(--color-base);
		}

		.example-text-2 {
			grid-area: text2;
			width: 60%;
			background-color: var(--color-secondary);
		}
	}
}
</style>
