<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { type AppSettingsTabId, appSettingsTabs } from '@/components/ui/settings/app-settings-tabs'
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
			<aside class="settings-sidebar">
				<div class="settings-sidebar__brand">
					<button
						class="settings-sidebar__logo button-animation"
						type="button"
						@click="devModeCount"
					>
						<img :src="settingsLogoUrl" alt="Revoria" class="settings-sidebar__logo-image" />
					</button>
					<div class="settings-sidebar__brand-copy">
						<h1>{{ formatMessage(messages.title) }}</h1>
						<p>{{ formatMessage(messages.appVersion, { version }) }}</p>
					</div>
				</div>
				<nav class="settings-tabs" role="tablist" :aria-label="formatMessage(messages.title)">
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
				</nav>
			</aside>
			<div class="settings-stage">
				<header class="settings-stage__header">
					<h2>{{ activeTab ? formatMessage(activeTab.name) : formatMessage(messages.title) }}</h2>
				</header>
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
	min-height: 100%;
	padding: 1rem 1.25rem 1.6rem;
}

.settings-surface {
	position: relative;
}

.settings-workspace {
	display: grid;
	grid-template-columns: minmax(13rem, 16rem) minmax(0, 1fr);
	min-width: 0;
	gap: 1rem;
	align-items: start;
}

.settings-sidebar {
	position: sticky;
	top: 1rem;
	display: flex;
	flex-direction: column;
	gap: 0.85rem;
	min-width: 0;
	max-height: calc(100vh - var(--top-bar-height) - 4rem);
	padding: 0.85rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 72%, transparent);
	border-radius: 0.95rem;
	background: color-mix(in srgb, var(--color-glass-bg-strong) 70%, transparent);
	box-shadow:
		inset 0 1px 0 color-mix(in srgb, white 4%, transparent),
		0 10px 26px color-mix(in srgb, black 17%, transparent);
	overflow: hidden;
}

.settings-sidebar__brand {
	display: grid;
	grid-template-columns: auto minmax(0, 1fr);
	gap: 0.7rem;
	align-items: center;
	padding: 0.25rem 0.25rem 0.75rem;
	border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 64%, transparent);
}

.settings-sidebar__brand-copy {
	min-width: 0;

	h1,
	p {
		margin: 0;
	}

	h1 {
		color: var(--color-contrast);
		font-size: 1rem;
		font-weight: 800;
		line-height: 1.15;
	}

	p {
		margin-top: 0.2rem;
		color: var(--color-secondary);
		font-size: 0.72rem;
		line-height: 1.15;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
}

.settings-sidebar__logo {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 2.65rem;
	height: 2.65rem;
	padding: 0;
	border: 1px solid color-mix(in srgb, var(--glass-border) 78%, transparent);
	border-radius: 0.75rem;
	background: color-mix(in srgb, var(--color-button-bg) 58%, transparent);
	box-shadow: none;
	cursor: pointer;
	transition:
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
		background 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.settings-sidebar__logo:hover {
	transform: translateY(-1px);
	border-color: color-mix(in srgb, var(--color-brand) 22%, var(--glass-border));
	background: color-mix(in srgb, var(--color-button-bg-hover) 62%, transparent);
}

.settings-sidebar__logo-image {
	width: 1.75rem;
	height: 1.75rem;
	border-radius: 0.55rem;
	object-fit: cover;
}

.settings-tabs {
	display: flex;
	flex-direction: column;
	gap: 0.18rem;
	min-height: 0;
	overflow-y: auto;
	padding-right: 0.15rem;
}

.settings-tab {
	position: relative;
	display: flex;
	align-items: center;
	gap: 0.7rem;
	width: 100%;
	min-height: 2.55rem;
	padding: 0.58rem 0.72rem;
	border: 1px solid transparent;
	border-radius: 0.68rem;
	background: transparent;
	color: var(--color-base);
	font-weight: 720;
	text-align: left;
	cursor: pointer;
	transition:
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
		background 180ms cubic-bezier(0.22, 1, 0.36, 1),
		color 180ms cubic-bezier(0.22, 1, 0.36, 1);

	&::before {
		content: '';
		position: absolute;
		left: 0.38rem;
		top: 50%;
		width: 0.22rem;
		height: 1.15rem;
		border-radius: 999px;
		background: var(--color-brand);
		opacity: 0;
		transform: translateY(-50%) scaleY(0.5);
		transition:
			opacity 180ms cubic-bezier(0.22, 1, 0.36, 1),
			transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
	}
}

.settings-tab:hover {
	transform: translateX(2px);
	color: var(--color-contrast);
	background: color-mix(in srgb, var(--color-button-bg-hover) 48%, transparent);
	border-color: color-mix(in srgb, var(--glass-border) 58%, transparent);
}

.settings-tab--active {
	color: var(--color-contrast);
	border-color: color-mix(in srgb, var(--color-brand) 24%, var(--glass-border) 76%);
	background: color-mix(in srgb, var(--color-button-bg-selected) 30%, var(--color-button-bg) 70%);

	&::before {
		opacity: 1;
		transform: translateY(-50%) scaleY(1);
	}
}

.settings-tab__icon {
	width: 1rem;
	height: 1rem;
	flex-shrink: 0;
}

.settings-tab__label {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
}

.settings-stage {
	min-width: 0;
	padding: 0.95rem 1rem 1.15rem;
	border-radius: 0.95rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 72%, transparent);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 62%, transparent);
	box-shadow:
		inset 0 1px 0 color-mix(in srgb, white 4%, transparent),
		0 10px 26px color-mix(in srgb, black 16%, transparent);
}

.settings-stage__header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1rem;
	padding: 0.15rem 0.15rem 0.9rem;
	margin-bottom: 0.15rem;
	border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 62%, transparent);

	h2 {
		margin: 0;
		color: var(--color-contrast);
		font-size: 1.22rem;
		font-weight: 850;
		line-height: 1.2;
	}
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
	.settings-workspace {
		grid-template-columns: 1fr;
	}

	.settings-sidebar {
		position: relative;
		top: auto;
		max-height: none;
	}

	.settings-tabs {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
		overflow: visible;
	}
}

@media (max-width: 720px) {
	.settings-page {
		padding-inline: 0.75rem;
	}

	.settings-stage__footer {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
	}
}

.settings-stage__content :deep(.settings-row) {
	min-height: 4.1rem;
	padding: 0.78rem 0.9rem !important;
	border: 1px solid color-mix(in srgb, var(--glass-border) 64%, transparent) !important;
	border-radius: 0.78rem !important;
	background: color-mix(in srgb, var(--color-button-bg) 34%, transparent) !important;
	box-shadow: none !important;
	transition:
		background 180ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.settings-stage__content :deep(.settings-row + .settings-row) {
	margin-top: 0.45rem !important;
}

.settings-stage__content :deep(.settings-row > div:first-child) {
	min-width: 0;
	flex: 1 1 auto;
}

.settings-stage__content :deep(.settings-row:hover) {
	transform: translateY(-1px);
	border-color: color-mix(in srgb, var(--color-brand) 16%, var(--glass-border) 84%) !important;
	background: color-mix(in srgb, var(--color-button-bg-hover) 38%, transparent) !important;
}

.settings-stage__content :deep(.settings-row h2) {
	font-size: 0.98rem !important;
	line-height: 1.2;
}

.settings-stage__content :deep(.settings-row h3) {
	margin-top: 0 !important;
	font-size: 0.88rem !important;
	line-height: 1.2;
}

.settings-stage__content :deep(.settings-row p) {
	max-width: 62ch;
	margin-top: 0.24rem !important;
	margin-bottom: 0 !important;
	color: var(--color-secondary);
	font-size: 0.84rem;
	line-height: 1.35;
}

.settings-stage__content :deep(.settings-select) {
	flex: 0 0 clamp(11rem, 26vw, 15.5rem);
	width: clamp(11rem, 26vw, 15.5rem) !important;
	min-width: 0;
}

.settings-stage__content :deep(.settings-select [role='button']) {
	min-height: 2.35rem;
	padding: 0.45rem 0.7rem 0.45rem 0.85rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 58%, transparent);
	border-radius: 0.68rem;
	background: color-mix(in srgb, var(--color-bg) 42%, transparent);
	box-shadow: none;
}

.settings-stage__content :deep(.settings-select [role='button']:hover) {
	border-color: color-mix(in srgb, var(--color-brand) 18%, var(--glass-border) 82%);
	background: color-mix(in srgb, var(--color-button-bg-hover) 40%, transparent);
}

.settings-stage__content :deep(.settings-select [role='button'] > div:first-child) {
	min-width: 0;
}

.settings-stage__content :deep(.settings-select [role='button'] span) {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
	font-size: 0.9rem;
}

.settings-stage__content :deep(.settings-select svg) {
	width: 1rem;
	height: 1rem;
}

.settings-stage__content :deep(.iconified-input) {
	display: flex;
	align-items: center;
	gap: 0.2rem;
	min-height: 2.45rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 64%, transparent) !important;
	border-radius: 0.72rem !important;
	background: color-mix(in srgb, var(--color-bg) 42%, transparent) !important;
	box-shadow: none !important;
	overflow: hidden;
}

.settings-stage__content :deep(.iconified-input > svg) {
	position: static !important;
	left: auto !important;
	z-index: auto !important;
	width: 1rem;
	height: 1rem;
	margin-left: 0.68rem;
	color: var(--color-secondary);
	flex-shrink: 0;
}

.settings-stage__content :deep(.iconified-input .text-input) {
	position: relative !important;
	flex: 1 1 auto;
	min-width: 0;
	min-height: 2.35rem;
	padding: 0.48rem 0.65rem 0.48rem 0.35rem !important;
	background: transparent !important;
	border: 0 !important;
	color: var(--color-contrast);
	font-size: 0.9rem;
}

.settings-stage__content :deep(.iconified-input .r-btn) {
	position: static !important;
	right: auto !important;
	z-index: auto !important;
	align-self: stretch;
	min-height: 2.35rem;
	margin: 0.04rem;
	padding: 0 0.72rem;
	border-radius: 0.58rem;
	box-shadow: none;
}

.settings-stage__content :deep(.iconified-input .r-btn svg) {
	width: 1rem;
	height: 1rem;
}

.settings-stage__content :deep(.settings-row .r-btn),
.settings-stage__content :deep(.btn) {
	min-height: 2.25rem;
	border-radius: 0.68rem;
}

.settings-stage__content :deep(.settings-input),
.settings-stage__content :deep(textarea) {
	min-height: 2.35rem !important;
	padding: 0.5rem 0.75rem !important;
	border-radius: 0.68rem !important;
	border-color: color-mix(in srgb, var(--glass-border) 64%, transparent) !important;
	background: color-mix(in srgb, var(--color-bg) 42%, transparent) !important;
}

.settings-stage__content :deep(input.settings-input:not(.w-full)) {
	width: clamp(7.25rem, 15vw, 12.5rem);
	text-align: left;
}

.settings-stage__content :deep(.root-container) {
	gap: 0.75rem;
	align-items: center;
	margin-top: 0.55rem;
}

.settings-stage__content :deep(.slider-component) {
	min-width: 0;
}

.settings-stage__content :deep(.slider-input) {
	flex: 0 0 5.4rem;
	width: 5.4rem !important;
	min-height: 2.25rem;
	margin-left: 0 !important;
	padding: 0.45rem 0.72rem !important;
	border: 1px solid color-mix(in srgb, var(--glass-border) 64%, transparent) !important;
	border-radius: 0.68rem !important;
	background: color-mix(in srgb, var(--color-bg) 42%, transparent) !important;
	color: var(--color-contrast);
	font-weight: 700;
}

.settings-stage__content :deep(.slider-range) {
	margin-top: 0.32rem;
	color: var(--color-secondary);
}

.settings-stage__content :deep(.slider-range span) {
	font-size: 0.72rem;
	line-height: 1;
}

@media (max-width: 760px) {
	.settings-stage__content :deep(.settings-row) {
		align-items: stretch !important;
		flex-direction: column;
	}

	.settings-stage__content :deep(.settings-select),
	.settings-stage__content :deep(input.settings-input:not(.w-full)),
	.settings-stage__content :deep(.slider-input) {
		width: 100% !important;
		flex-basis: auto;
	}

	.settings-stage__content :deep(.root-container) {
		align-items: stretch;
		flex-direction: column;
	}
}
</style>
