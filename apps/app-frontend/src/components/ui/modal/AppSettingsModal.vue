<script setup lang="ts">
import {
	CoffeeIcon,
	GameIcon,
	GaugeIcon,
	GlobeIcon,
	LanguagesIcon,
	PaintbrushIcon,
	PaletteIcon,
	ReportIcon,
	SettingsIcon,
	ShieldIcon,
} from '@modrinth/assets'
import { TabbedModal } from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { defineMessage, defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import ThemesSettings from '@/components/ui/settings/ThemesSettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/FeatureFlagSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import LanguageSettings from '@/components/ui/settings/LanguageSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import ProxySettings from '@/components/ui/settings/ProxySettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'
import { get, set } from '@/helpers/settings.ts'
import { getThemeIconUrl } from '@/helpers/theme-icons'
import { useTheming } from '@/store/state'

const themeStore = useTheming()

const { formatMessage } = useVIntl()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.themes',
			defaultMessage: 'Themes',
		}),
		icon: PaletteIcon,
		content: ThemesSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Language',
		}),
		icon: LanguagesIcon,
		content: LanguageSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java installations',
		}),
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.default-instance-options',
			defaultMessage: 'Default instance options',
		}),
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Resource management',
		}),
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.proxy',
			defaultMessage: 'Proxy',
		}),
		icon: GlobeIcon,
		content: ProxySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.feature-flags',
			defaultMessage: 'Feature flags',
		}),
		icon: ReportIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
]

const modal = ref()

function show() {
	modal.value.show()
}

const isOpen = computed(() => modal.value?.isOpen)

defineExpose({ show, isOpen })

const version = await getVersion()
const modrinthBaseVersion = '0.10.2401'
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const settings = ref(await get())
const systemThemeMediaQuery =
	typeof window !== 'undefined' ? window.matchMedia('(prefers-color-scheme: dark)') : null
const settingsLogoUrl = computed(() =>
	getThemeIconUrl(themeStore.selectedTheme, systemThemeMediaQuery?.matches ?? false),
)

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		themeStore.devMode = !themeStore.devMode
		settings.value.developer_mode = !!themeStore.devMode
		devModeCounter.value = 0

		if (!themeStore.devMode && tabs[modal.value.selectedTab].developerOnly) {
			modal.value.setTab(0)
		}
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	settingsTitle: {
		id: 'app.settings.title',
		defaultMessage: 'Settings',
	},
	appVersion: {
		id: 'app.settings.footer.version',
		defaultMessage: 'Revoria {version}',
	},
	baseVersion: {
		id: 'app.settings.footer.base-version',
		defaultMessage: 'Based on Modrinth App {version}',
	},
})
</script>
<template>
	<ModalWrapper
		ref="modal"
		:width="'928px'"
		:min-width="'928px'"
		:max-width="'928px'"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
				<SettingsIcon /> {{ formatMessage(messages.settingsTitle) }}
			</span>
		</template>

		<TabbedModal
			class="app-settings-tabs"
			:tabs="tabs.filter((t) => !t.developerOnly || themeStore.devMode)"
		>
			<template #footer>
				<div class="mt-auto text-secondary text-sm">
					<div class="mb-3"></div>
					<p v-if="themeStore.devMode" class="text-brand font-semibold m-0 mb-2">
						{{ formatMessage(developerModeEnabled) }}
					</p>
					<div class="flex items-center gap-3">
						<button
							class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
							:class="{
								'text-brand': themeStore.devMode,
								'text-secondary': !themeStore.devMode,
							}"
							@click="devModeCount"
						>
							<img :src="settingsLogoUrl" alt="Revoria" class="settings-logo-image" />
						</button>
						<div>
							<p class="m-0">{{ formatMessage(messages.appVersion, { version }) }}</p>
							<p class="m-0 text-xs">
								{{ formatMessage(messages.baseVersion, { version: modrinthBaseVersion }) }}
							</p>
							<p class="m-0">
								<span v-if="osPlatform === 'macos'">macOS</span>
								<span v-else class="capitalize">{{ osPlatform }}</span>
								{{ osVersion }}
							</p>
						</div>
					</div>
				</div>
			</template>
		</TabbedModal>
	</ModalWrapper>
</template>

<style lang="scss" scoped>
code {
	color: var(--color-brand);
}

:deep(.app-settings-tabs .tabbed-modal) {
	grid-template-columns: 220px minmax(0, 1fr);
}

:deep(.app-settings-tabs .tabbed-modal__sidebar) {
	width: 220px;
	min-width: 220px;
	max-width: 220px;
	overflow: hidden;
}

:deep(.app-settings-tabs .tabbed-modal__sidebar button) {
	white-space: normal !important;
	line-height: 1.2;
	max-width: 100%;
}

.settings-logo-image {
	display: block;
	width: 1.5rem;
	height: 1.5rem;
	border-radius: 0.4rem;
	object-fit: cover;
	box-shadow: 0 0 0 1px var(--button-border);
}
</style>
