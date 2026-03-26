<script setup lang="ts">
import { FileImageIcon, TrashIcon } from '@modrinth/assets'
import { Button, Combobox, Slider, Toggle } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils'
import { useTheming } from '@/store/state'

const themeStore = useTheming()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	advancedRenderingTitle: {
		id: 'settings.appearance.advanced-rendering.title',
		defaultMessage: 'Advanced rendering',
	},
	advancedRenderingDescription: {
		id: 'settings.appearance.advanced-rendering.description',
		defaultMessage:
			'Enables advanced rendering such as blur effects that may cause performance issues without hardware-accelerated rendering.',
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
	glassBlurTitle: {
		id: 'settings.appearance.glass-blur.title',
		defaultMessage: 'Glass blur',
	},
	glassBlurDescription: {
		id: 'settings.appearance.glass-blur.description',
		defaultMessage: 'Adjust the blur strength used for glass surfaces.',
	},
	glassBorderOpacityTitle: {
		id: 'settings.appearance.glass-border-opacity.title',
		defaultMessage: 'Glass border opacity',
	},
	glassBorderOpacityDescription: {
		id: 'settings.appearance.glass-border-opacity.description',
		defaultMessage: 'Controls how visible glass borders are.',
	},
	backgroundEffectsTitle: {
		id: 'settings.appearance.background-effects.title',
		defaultMessage: 'Background effects',
	},
	backgroundEffectsDescription: {
		id: 'settings.appearance.background-effects.description',
		defaultMessage: 'Decorative effects rendered behind the UI.',
	},
	backgroundEffectsIntensityTitle: {
		id: 'settings.appearance.background-effects-intensity.title',
		defaultMessage: 'Effect density',
	},
	backgroundEffectsIntensityDescription: {
		id: 'settings.appearance.background-effects-intensity.description',
		defaultMessage: 'Controls how often particles appear and how dense the effect feels.',
	},
	pageBackgroundImageTitle: {
		id: 'settings.appearance.page-background-image.title',
		defaultMessage: 'Page background image',
	},
	pageBackgroundImageDescription: {
		id: 'settings.appearance.page-background-image.description',
		defaultMessage:
			'Use a PNG, JPG, or WebP image behind page content. The image is loaded from a local file on your device.',
	},
	pageBackgroundImageSelect: {
		id: 'settings.appearance.page-background-image.select',
		defaultMessage: 'Choose image',
	},
	pageBackgroundImageReplace: {
		id: 'settings.appearance.page-background-image.replace',
		defaultMessage: 'Replace image',
	},
	pageBackgroundImageRemove: {
		id: 'settings.appearance.page-background-image.remove',
		defaultMessage: 'Remove',
	},
	pageBackgroundImageEmpty: {
		id: 'settings.appearance.page-background-image.empty',
		defaultMessage: 'No image selected',
	},
	pageBackgroundOpacityTitle: {
		id: 'settings.appearance.page-background-opacity.title',
		defaultMessage: 'Page background opacity',
	},
	pageBackgroundOpacityDescription: {
		id: 'settings.appearance.page-background-opacity.description',
		defaultMessage: 'Control how visible the page background image is behind content.',
	},
	backgroundEffectsOff: {
		id: 'settings.appearance.background-effects.off',
		defaultMessage: 'Off',
	},
	backgroundEffectsSnow: {
		id: 'settings.appearance.background-effects.snow',
		defaultMessage: 'Snow',
	},
	backgroundEffectsStars: {
		id: 'settings.appearance.background-effects.stars',
		defaultMessage: 'Stardust',
	},
	backgroundEffectsRain: {
		id: 'settings.appearance.background-effects.rain',
		defaultMessage: 'Rain',
	},
})

const os = ref(await getOS())
const settings = ref(await get())

themeStore.glassBlur = settings.value.glass_blur
themeStore.glassBorderOpacity = settings.value.glass_border_opacity
themeStore.backgroundEffect = settings.value.background_effect
themeStore.backgroundEffectIntensity = settings.value.background_effect_intensity ?? 100
themeStore.setPageBackground(
	settings.value.page_background_path,
	settings.value.page_background_opacity ?? 0.22,
)
themeStore.applyGlassSettings()

const glassBorderOpacityPercent = computed({
	get: () => Math.round((settings.value.glass_border_opacity ?? 0.075) * 1000) / 10,
	set: (v: number) => {
		const next = Math.max(0, Math.min(100, v)) / 100
		settings.value.glass_border_opacity = next
		themeStore.setGlassBorderOpacity(next)
	},
})

const backgroundEffectOptions = computed(() => [
	{ value: 'off', label: formatMessage(messages.backgroundEffectsOff) },
	{ value: 'snow', label: formatMessage(messages.backgroundEffectsSnow) },
	{ value: 'stars', label: formatMessage(messages.backgroundEffectsStars) },
	{ value: 'rain', label: formatMessage(messages.backgroundEffectsRain) },
])

const backgroundEffectIntensityPercent = computed({
	get: () => settings.value.background_effect_intensity ?? 100,
	set: (v: number) => {
		const next = Math.max(10, Math.min(400, Math.round(v)))
		settings.value.background_effect_intensity = next
		themeStore.setBackgroundEffectIntensity(next)
	},
})

const pageBackgroundOpacityPercent = computed({
	get: () => Math.round((settings.value.page_background_opacity ?? 0.22) * 100),
	set: (v: number) => {
		const next = Math.max(0, Math.min(100, Math.round(v))) / 100
		settings.value.page_background_opacity = next
		themeStore.setPageBackgroundOpacity(next)
	},
})

const pageBackgroundLabel = computed(() => {
	const path = settings.value.page_background_path?.trim()
	if (!path) return formatMessage(messages.pageBackgroundImageEmpty)
	return path.split(/[/\\]/).pop() ?? path
})

async function pickPageBackground() {
	const picked = await open({
		multiple: false,
		title: formatMessage(messages.pageBackgroundImageTitle),
		filters: [
			{
				name: 'Images',
				extensions: ['png', 'jpg', 'jpeg', 'webp', 'avif'],
			},
		],
	})

	if (typeof picked === 'string' && picked.length > 0) {
		settings.value.page_background_path = picked
		themeStore.setPageBackground(picked, settings.value.page_background_opacity ?? 0.22)
	}
}

function clearPageBackground() {
	settings.value.page_background_path = ''
	themeStore.setPageBackground('', settings.value.page_background_opacity ?? 0.22)
}

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)
</script>
<template>
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

	<div class="settings-row mt-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.pageBackgroundImageTitle) }}
			</h2>
			<p class="m-0 mt-1">
				{{ formatMessage(messages.pageBackgroundImageDescription) }}
			</p>
		</div>
		<div class="mt-3 flex items-center gap-2">
			<div class="iconified-input flex-grow">
				<FileImageIcon />
				<input
					readonly
					:value="pageBackgroundLabel"
					type="text"
					class="text-input"
				/>
			</div>
			<Button class="r-btn" @click="pickPageBackground">
				<FileImageIcon />
				{{
					settings.page_background_path
						? formatMessage(messages.pageBackgroundImageReplace)
						: formatMessage(messages.pageBackgroundImageSelect)
				}}
			</Button>
			<Button
				v-if="settings.page_background_path"
				color="danger"
				class="r-btn"
				@click="clearPageBackground"
			>
				<TrashIcon />
				{{ formatMessage(messages.pageBackgroundImageRemove) }}
			</Button>
		</div>
	</div>

	<div class="settings-row mt-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.pageBackgroundOpacityTitle) }}
			</h2>
			<p class="m-0 mt-1">
				{{ formatMessage(messages.pageBackgroundOpacityDescription) }}
			</p>
		</div>
		<Slider
			id="page-background-opacity"
			v-model="pageBackgroundOpacityPercent"
			:min="0"
			:max="100"
			:step="1"
			unit="%"
		/>
	</div>

	<div class="settings-row mt-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.backgroundEffectsIntensityTitle) }}
			</h2>
			<p class="m-0 mt-1">
				{{ formatMessage(messages.backgroundEffectsIntensityDescription) }}
			</p>
		</div>
		<Slider
			id="background-effects-intensity"
			v-model="backgroundEffectIntensityPercent"
			:min="10"
			:max="400"
			:step="10"
			unit="%"
		/>
	</div>

	<div class="settings-row mt-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.glassBlurTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.glassBlurDescription) }}</p>
		</div>
		<Slider
			id="glass-blur"
			v-model="settings.glass_blur"
			:min="0"
			:max="72"
			:step="1"
			unit="px"
			@update:model-value="(v) => themeStore.setGlassBlur(Number(v))"
		/>
	</div>

	<div class="settings-row mt-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.glassBorderOpacityTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.glassBorderOpacityDescription) }}</p>
		</div>
		<Slider
			id="glass-border-opacity"
			v-model="glassBorderOpacityPercent"
			:min="0"
			:max="20"
			:step="0.5"
			unit="%"
		/>
	</div>

	<div class="settings-row mt-4 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.backgroundEffectsTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.backgroundEffectsDescription) }}</p>
		</div>
		<Combobox
			id="background-effects"
			v-model="settings.background_effect"
			:name="formatMessage(messages.backgroundEffectsTitle)"
			class="w-40"
			:options="backgroundEffectOptions"
			:display-value="
				backgroundEffectOptions.find((o) => o.value === settings.background_effect)?.label ??
				formatMessage(messages.backgroundEffectsOff)
			"
			@update:model-value="
				(v) => themeStore.setBackgroundEffect(v as 'off' | 'snow' | 'stars' | 'rain')
			"
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
				{ value: 'home', label: formatMessage(messages.defaultLandingHome) },
				{ value: 'library', label: formatMessage(messages.defaultLandingLibrary) },
			]"
			:display-value="
				settings.default_page
					? formatMessage(
							settings.default_page === 'library'
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

:global(.light-mode),
:global(.rose-gold-mode),
:global(.cherry-blossom-mode) {
	.settings-row {
		border-color: color-mix(in srgb, var(--glass-border) 90%, rgba(29, 43, 48, 0.16));
		background: linear-gradient(
			180deg,
			color-mix(in oklch, var(--color-glass-bg-strong) 96%, white 4%),
			color-mix(in oklch, var(--color-glass-bg) 97%, white 3%)
		);
		box-shadow:
			0 18px 40px -30px rgba(39, 49, 45, 0.18),
			0 1px 0 rgba(255, 255, 255, 0.55) inset;
	}

	.settings-row p {
		color: var(--color-secondary);
	}
}
</style>
