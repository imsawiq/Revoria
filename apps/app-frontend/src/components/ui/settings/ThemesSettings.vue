<script setup lang="ts">
import { ThemeSelector } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { ref } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import { useTheming } from '@/store/state'
import type { ColorTheme } from '@/store/theme.ts'

const themeStore = useTheming()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	colorThemeTitle: {
		id: 'settings.themes.color-theme.title',
		defaultMessage: 'Themes',
	},
	colorThemeDescription: {
		id: 'settings.themes.color-theme.description',
		defaultMessage: 'Select your preferred color theme for Revoria.',
	},
})

const settings = ref(await get())

const lastSavedTheme = ref<ColorTheme>(settings.value.theme)

async function applyTheme(theme: ColorTheme) {
	const previous = lastSavedTheme.value

	themeStore.setThemeState(theme)
	settings.value.theme = theme

	try {
		await set(settings.value)
		lastSavedTheme.value = theme
	} catch (err) {
		console.error('Failed to save theme setting.', err)
		settings.value.theme = previous
		themeStore.setThemeState(previous)
	}
}
</script>

<template>
	<h2 class="m-0 text-lg font-extrabold text-contrast">
		{{ formatMessage(messages.colorThemeTitle) }}
	</h2>
	<p class="m-0 mt-1">{{ formatMessage(messages.colorThemeDescription) }}</p>

	<ThemeSelector
		:update-color-theme="
			(theme: ColorTheme) => {
				applyTheme(theme)
			}
		"
		:current-theme="themeStore.selectedTheme"
		:theme-options="themeStore.getThemeOptions()"
		system-theme-color="system"
	/>
</template>
