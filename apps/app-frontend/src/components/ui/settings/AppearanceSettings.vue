<script setup lang="ts">
import { Combobox, ThemeSelector, Toggle } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils'
import { useTheming } from '@/store/state'
import type { ColorTheme } from '@/store/theme.ts'

const themeStore = useTheming()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	colorThemeTitle: {
		id: 'settings.appearance.color-theme.title',
		defaultMessage: 'Color theme',
	},
	colorThemeDescription: {
		id: 'settings.appearance.color-theme.description',
		defaultMessage: 'Select your preferred color theme for Modrinth App.',
	},
	advancedRenderingTitle: {
		id: 'settings.appearance.advanced-rendering.title',
		defaultMessage: 'Advanced rendering',
	},
	advancedRenderingDescription: {
		id: 'settings.appearance.advanced-rendering.description',
		defaultMessage:
			'Enables advanced rendering such as blur effects that may cause performance issues without hardware-accelerated rendering.',
	},
	launcherLanguageTitle: {
		id: 'settings.appearance.launcher-language.title',
		defaultMessage: 'Launcher language',
	},
	launcherLanguageDescription: {
		id: 'settings.appearance.launcher-language.description',
		defaultMessage: 'Select the language used by Revoria UI.',
	},
	hideNametagTitle: {
		id: 'settings.appearance.hide-nametag.title',
		defaultMessage: 'Hide nametag',
	},
	hideNametagDescription: {
		id: 'settings.appearance.hide-nametag.description',
		defaultMessage: 'Disables the nametag above your player on the skins page.',
	},
	nativeDecorationsTitle: {
		id: 'settings.appearance.native-decorations.title',
		defaultMessage: 'Native decorations',
	},
	nativeDecorationsDescription: {
		id: 'settings.appearance.native-decorations.description',
		defaultMessage: 'Use system window frame (app restart required).',
	},
	minimizeLauncherTitle: {
		id: 'settings.appearance.minimize-launcher.title',
		defaultMessage: 'Minimize launcher',
	},
	minimizeLauncherDescription: {
		id: 'settings.appearance.minimize-launcher.description',
		defaultMessage: 'Minimize the launcher when a Minecraft process starts.',
	},
	defaultLandingTitle: {
		id: 'settings.appearance.default-landing.title',
		defaultMessage: 'Default landing page',
	},
	defaultLandingDescription: {
		id: 'settings.appearance.default-landing.description',
		defaultMessage: 'Change the page to which the launcher opens on.',
	},
	defaultLandingPlaceholder: {
		id: 'settings.appearance.default-landing.placeholder',
		defaultMessage: 'Select an option',
	},
	jumpBackTitle: {
		id: 'settings.appearance.jump-back.title',
		defaultMessage: 'Jump back into worlds',
	},
	jumpBackDescription: {
		id: 'settings.appearance.jump-back.description',
		defaultMessage: 'Includes recent worlds in the "Jump back in" section on the Home page.',
	},
	toggleSidebarTitle: {
		id: 'settings.appearance.toggle-sidebar.title',
		defaultMessage: 'Toggle sidebar',
	},
	toggleSidebarDescription: {
		id: 'settings.appearance.toggle-sidebar.description',
		defaultMessage: 'Enables the ability to toggle the sidebar.',
	},
	defaultLandingHome: {
		id: 'settings.appearance.default-landing.home',
		defaultMessage: 'Home',
	},
	defaultLandingLibrary: {
		id: 'settings.appearance.default-landing.library',
		defaultMessage: 'Library',
	},
	languageEnglish: {
		id: 'settings.appearance.launcher-language.option.english',
		defaultMessage: 'English',
	},
	languageRussian: {
		id: 'settings.appearance.launcher-language.option.russian',
		defaultMessage: 'Russian',
	},
})

const os = ref(await getOS())
const settings = ref(await get())
const launcherLanguage = useStorage('launcher-language', 'en')
const languageOptions = computed(() => [
	{ value: 'en', label: formatMessage(messages.languageEnglish) },
	{ value: 'ru', label: formatMessage(messages.languageRussian) },
])
const selectedLanguageLabel = computed(
	() =>
		languageOptions.value.find((option) => option.value === launcherLanguage.value)?.label ??
		formatMessage(messages.languageEnglish),
)

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)
</script>
<template>
	<h2 class="m-0 text-lg font-extrabold text-contrast">
		{{ formatMessage(messages.colorThemeTitle) }}
	</h2>
	<p class="m-0 mt-1">{{ formatMessage(messages.colorThemeDescription) }}</p>

	<ThemeSelector
		:update-color-theme="
			(theme: ColorTheme) => {
				themeStore.setThemeState(theme)
				settings.theme = theme
			}
		"
		:current-theme="settings.theme"
		:theme-options="themeStore.getThemeOptions()"
		system-theme-color="system"
	/>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.advancedRenderingTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.advancedRenderingDescription) }}</p>
		</div>

		<Toggle
			id="advanced-rendering"
			:model-value="themeStore.advancedRendering"
			@update:model-value="
				(e) => {
					themeStore.advancedRendering = !!e
					settings.advanced_rendering = themeStore.advancedRendering
				}
			"
		/>
	</div>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.launcherLanguageTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.launcherLanguageDescription) }}</p>
		</div>
		<Combobox
			id="launcher-language"
			v-model="launcherLanguage"
			:name="formatMessage(messages.launcherLanguageTitle)"
			class="w-40"
			:options="languageOptions"
			:display-value="selectedLanguageLabel"
		/>
	</div>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.hideNametagTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.hideNametagDescription) }}</p>
		</div>
		<Toggle id="hide-nametag-skins-page" v-model="settings.hide_nametag_skins_page" />
	</div>

	<div v-if="os !== 'MacOS'" class="settings-row mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.nativeDecorationsTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.nativeDecorationsDescription) }}</p>
		</div>
		<Toggle id="native-decorations" v-model="settings.native_decorations" />
	</div>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.minimizeLauncherTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.minimizeLauncherDescription) }}</p>
		</div>
		<Toggle id="minimize-launcher" v-model="settings.hide_on_process_start" />
	</div>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.defaultLandingTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.defaultLandingDescription) }}</p>
		</div>
		<Combobox
			id="opening-page"
			v-model="settings.default_page"
			:name="formatMessage(messages.defaultLandingTitle)"
			class="w-40"
			:options="[
				{ value: 'Home', label: formatMessage(messages.defaultLandingHome) },
				{ value: 'Library', label: formatMessage(messages.defaultLandingLibrary) },
			]"
			:display-value="
				settings.default_page
					? formatMessage(
							settings.default_page === 'Library'
								? messages.defaultLandingLibrary
								: messages.defaultLandingHome,
						)
					: formatMessage(messages.defaultLandingPlaceholder)
			"
		/>
	</div>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.jumpBackTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.jumpBackDescription) }}</p>
		</div>
		<Toggle
			:model-value="themeStore.getFeatureFlag('worlds_in_home')"
			@update:model-value="
				() => {
					const newValue = !themeStore.getFeatureFlag('worlds_in_home')
					themeStore.featureFlags['worlds_in_home'] = newValue
					settings.feature_flags['worlds_in_home'] = newValue
				}
			"
		/>
	</div>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.toggleSidebarTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.toggleSidebarDescription) }}</p>
		</div>
		<Toggle
			id="toggle-sidebar"
			:model-value="settings.toggle_sidebar"
			@update:model-value="
				(e) => {
					settings.toggle_sidebar = !!e
					themeStore.toggleSidebar = settings.toggle_sidebar
				}
			"
		/>
	</div>
</template>

<style lang="scss" scoped>
.settings-row {
	padding: 1rem 1.125rem;
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-xl);
	background: linear-gradient(
		180deg,
		color-mix(in oklch, var(--color-glass-bg-strong) 88%, transparent),
		color-mix(in oklch, var(--color-glass-bg) 94%, transparent)
	);
	box-shadow: var(--shadow-card);
}
</style>
