<template>
	<div>
		<section class="universal-card">
			<h2 class="text-2xl">{{ formatMessage(colorTheme.title) }}</h2>
			<p>{{ formatMessage(colorTheme.description) }}</p>
			<ThemeSelector
				:update-color-theme="updateColorTheme"
				:current-theme="theme.preferred"
				:theme-options="themeOptions"
				:system-theme-color="systemTheme"
			/>
		</section>
	</div>
</template>

<script setup lang="ts">
import { ThemeSelector } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'

import { isDarkTheme, type Theme } from '~/plugins/theme/index.ts'

useHead({
	title: 'Theme settings - Modrinth',
})

const { formatMessage } = useVIntl()

const colorTheme = defineMessages({
	title: {
		id: 'settings.display.theme.title',
		defaultMessage: 'Color theme',
	},
	description: {
		id: 'settings.display.theme.description',
		defaultMessage: 'Select your preferred color theme for Modrinth on this device.',
	},
})

const theme = useTheme()

const serverSystemTheme = useState(() => {
	const theme_ = theme.native
	if (theme_ === 'unknown') return 'light'
	return theme_
})

const systemTheme = useMountedValue((mounted): Theme => {
	const systemTheme_ = mounted ? theme.native : serverSystemTheme.value
	return systemTheme_ === 'light' ? theme.preferences.light : theme.preferences.dark
})

const themeOptions = computed(() => {
	const options: ('system' | Theme)[] = [
		'system',
		'light',
		'dark',
		'oled',
		'retro',
		'sapphire',
		'amethyst',
		'sunset',
		'aurora',
	]
	return options
})

function updateColorTheme(value: Theme | 'system') {
	if (value !== 'system') {
		if (isDarkTheme(value)) {
			theme.preferences.dark = value
		} else {
			theme.preferences.light = value
		}
	}

	theme.preferred = value
}
</script>
