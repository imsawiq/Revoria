<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import {
	type AppSettingsTabId,
	appSettingsTabs,
} from '@/components/ui/settings/app-settings-tabs'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { getThemeIconUrl } from '@/helpers/theme-icons'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useTheming } from '@/store/state'

const route = useRoute()
const router = useRouter()
const breadcrumbs = useBreadcrumbs()
const themeStore = useTheming()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'app.settings.title',
		defaultMessage: 'Настройки',
	},
	appVersion: {
		id: 'app.settings.footer.version',
		defaultMessage: 'Revoria {version}',
	},
	baseVersion: {
		id: 'app.settings.footer.base-version',
		defaultMessage: 'Based on Modrinth App {version}',
	},
	developerModeEnabled: {
		id: 'app.settings.developer-mode-enabled',
		defaultMessage: 'Developer mode enabled.',
	},
})

breadcrumbs.setRootContext({ name: formatMessage(messages.title), link: '/settings' })

const version = await getVersion()
const modrinthBaseVersion = '0.10.2401'
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const systemThemeMediaQuery =
	typeof window !== 'undefined' ? window.matchMedia('(prefers-color-scheme: dark)') : null
const settingsLogoUrl = computed(() =>
	getThemeIconUrl(themeStore.selectedTheme, systemThemeMediaQuery?.matches ?? false),
)

const persistentSettings = ref(await getSettings())
const devModeCounter = ref(0)
const lastSettingsTab = useStorage<AppSettingsTabId>('revoria-settings-tab', 'appearance')

const visibleTabs = computed(() =>
	appSettingsTabs.filter((tab) => !tab.developerOnly || themeStore.devMode),
)

const requestedTabId = computed(() =>
	typeof route.params.tab === 'string' ? route.params.tab : lastSettingsTab.value,
)

const activeTab = computed(
	() => visibleTabs.value.find((tab) => tab.id === requestedTabId.value) ?? visibleTabs.value[0],
)

watch(
	[requestedTabId, visibleTabs],
	([requested, tabs]) => {
		if (!tabs.length) return
		const fallback = tabs.find((tab) => tab.id === requested) ?? tabs[0]
		lastSettingsTab.value = fallback.id
		if (route.params.tab !== fallback.id) {
			router.replace({ path: `/settings/${fallback.id}` })
		}
	},
	{ immediate: true },
)

function openTab(tabId: AppSettingsTabId) {
	lastSettingsTab.value = tabId
	router.push({ path: `/settings/${tabId}` })
}

async function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value <= 5) return

	themeStore.devMode = !themeStore.devMode
	persistentSettings.value.developer_mode = !!themeStore.devMode
	await setSettings(persistentSettings.value)
	devModeCounter.value = 0

	if (activeTab.value?.developerOnly && !themeStore.devMode) {
		openTab('appearance')
	}
}

function formatPlatform(platform: string) {
	return platform === 'macos' ? 'macOS' : platform.charAt(0).toUpperCase() + platform.slice(1)
}
</script>

<template>
	<div class="settings-page">
		<section class="settings-workspace settings-surface">
			<header class="settings-tabs-wrap">
				<button class="settings-tabs-wrap__logo button-animation" type="button" @click="devModeCount">
					<img :src="settingsLogoUrl" alt="Revoria" class="settings-tabs-wrap__logo-image" />
				</button>
				<div class="settings-tabs" role="tablist">
					<button
						v-for="tab in visibleTabs"
						:key="tab.id"
						type="button"
						class="settings-tab"
						:class="{ 'settings-tab--active': activeTab?.id === tab.id }"
						@click="openTab(tab.id)"
					>
						<component :is="tab.icon" class="settings-tab__icon" />
						<span class="settings-tab__label">{{ formatMessage(tab.name) }}</span>
					</button>
				</div>
			</header>

			<div class="settings-stage">
				<div class="settings-stage__content">
					<Suspense>
						<component :is="activeTab?.content" v-if="activeTab" />
					</Suspense>
				</div>
				<footer class="settings-stage__footer">
					<div class="settings-stage__footer-meta">
						<span>{{ formatMessage(messages.appVersion, { version }) }}</span>
						<span>{{ formatPlatform(osPlatform) }} {{ osVersion }}</span>
						<span>{{ formatMessage(messages.baseVersion, { version: modrinthBaseVersion }) }}</span>
					</div>
					<p v-if="themeStore.devMode" class="settings-stage__footer-debug">
						{{ formatMessage(messages.developerModeEnabled) }}
					</p>
				</footer>
			</div>
		</section>
	</div>
</template>

<style scoped lang="scss">
.settings-page {
	display: flex;
	flex-direction: column;
	padding: 1rem 1.25rem 1.6rem;
}

.settings-surface {
	position: relative;
}

.settings-workspace {
	display: flex;
	flex-direction: column;
	min-width: 0;
	gap: 0.9rem;
}

.settings-tabs-wrap {
	display: flex;
	align-items: center;
	gap: 1rem;
	padding: 0.95rem 1rem 0.7rem;
	border-radius: 1.35rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 96%, transparent);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(calc(var(--glass-blur) * 0.84)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(calc(var(--glass-blur) * 0.84)) saturate(var(--glass-saturate));
}

.settings-tabs-wrap__logo {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 3.35rem;
	height: 3.35rem;
	padding: 0;
	border: 1px solid color-mix(in srgb, var(--glass-border) 85%, transparent);
	border-radius: 1rem;
	background: color-mix(in srgb, var(--color-button-bg) 92%, transparent);
	box-shadow: var(--shadow-card);
	cursor: pointer;
	transition:
		transform 160ms ease,
		border-color 160ms ease,
		background 160ms ease,
		box-shadow 160ms ease;
}

.settings-tabs-wrap__logo:hover {
	transform: translateY(-1px);
	border-color: color-mix(in srgb, var(--color-brand) 24%, var(--glass-border));
	background: color-mix(in srgb, var(--color-button-bg-hover) 92%, transparent);
	box-shadow:
		inset 0 0 0 1px color-mix(in srgb, var(--color-brand-highlight) 20%, transparent),
		var(--shadow-card);
}

.settings-tabs-wrap__logo-image {
	width: 2rem;
	height: 2rem;
	border-radius: 0.7rem;
	object-fit: cover;
}

.settings-tabs {
	display: flex;
	flex: 1 1 auto;
	gap: 0.45rem;
	overflow-x: auto;
	padding: 0.28rem 0.08rem 0.34rem;
	margin: -0.28rem -0.08rem -0.12rem;
}

.settings-tab {
	display: inline-flex;
	align-items: center;
	gap: 0.65rem;
	padding: 0.84rem 1rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 34%, transparent);
	border-radius: 999px;
	background: color-mix(in srgb, var(--color-button-bg) 86%, transparent);
	color: var(--color-base);
	font-weight: 700;
	white-space: nowrap;
	cursor: pointer;
	transition: transform 160ms ease, border-color 160ms ease, background 160ms ease, color 160ms ease, box-shadow 160ms ease;
}

.settings-tab:hover {
	transform: translateY(-1px);
	color: var(--color-contrast);
	background: color-mix(in srgb, var(--color-button-bg-hover) 92%, transparent);
	border-color: color-mix(in srgb, var(--color-brand) 18%, var(--glass-border));
	box-shadow: 0 12px 24px color-mix(in srgb, var(--color-brand-shadow) 8%, transparent);
}

.settings-tab--active {
	color: var(--color-contrast);
	border-color: color-mix(in srgb, var(--color-brand) 30%, var(--glass-border));
	background: color-mix(in srgb, var(--color-button-bg-selected) 34%, var(--color-button-bg) 66%);
	box-shadow:
		inset 0 0 0 1px color-mix(in srgb, var(--color-brand) 18%, transparent),
		0 12px 26px color-mix(in srgb, var(--color-brand-shadow) 10%, transparent);
}

.settings-tab__icon {
	width: 1rem;
	height: 1rem;
	flex-shrink: 0;
}

.settings-stage {
	min-width: 0;
	padding: 1rem 1.2rem 1.25rem;
	border-radius: 1.35rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 96%, transparent);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(calc(var(--glass-blur) * 0.84)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(calc(var(--glass-blur) * 0.84)) saturate(var(--glass-saturate));
}

.settings-stage__content {
	min-width: 0;
}

.settings-stage__footer {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1rem;
	margin-top: 1.15rem;
	padding-top: 0.95rem;
	border-top: 1px solid color-mix(in srgb, var(--glass-border) 70%, transparent);
}

.settings-stage__footer-meta {
	display: flex;
	flex-wrap: wrap;
	gap: 0.45rem 0.8rem;
	color: var(--color-secondary);
	font-size: 0.82rem;
}

.settings-stage__footer-debug {
	margin: 0;
	color: var(--color-brand);
	font-size: 0.82rem;
	font-weight: 700;
}

@media (max-width: 1024px) {
	.settings-tabs-wrap {
		align-items: flex-start;
		flex-direction: column;
	}
}

@media (max-width: 720px) {
	.settings-page {
		padding-inline: 0.75rem;
	}

	.settings-stage__footer {
		display: flex;
		flex-direction: column;
	}
}
</style>
