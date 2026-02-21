import 'floating-vue/dist/style.css'

import * as Sentry from '@sentry/vue'
import { VueScanPlugin } from '@taijased/vue-render-tracker'
import { VueQueryPlugin } from '@tanstack/vue-query'
import { createPlugin } from '@vintl/vintl/plugin'
import FloatingVue from 'floating-vue'
import { createPinia } from 'pinia'
import { createApp, watch } from 'vue'
import { useStorage } from '@vueuse/core'

import App from '@/App.vue'
import router from '@/routes'
import appEnMessages from '@/locales/en-US/index.json'
import appRuMessages from '@/locales/ru-RU/index.json'
import uiEnMessages from '@modrinth/ui/src/locales/en-US/index.json'
import uiRuMessages from '@modrinth/ui/src/locales/ru-RU/index.json'

const launcherLanguage = useStorage('launcher-language', 'en')
const localeMap = {
	en: 'en-US',
	ru: 'ru-RU',
}
const initialLocale = localeMap[launcherLanguage.value] ?? 'en-US'
const normalizeMessages = (messages) =>
	Object.fromEntries(
		Object.entries(messages ?? {}).map(([key, value]) => [
			key,
			value?.message ?? value?.defaultMessage ?? value,
		]),
	)

const VIntlPlugin = createPlugin({
	controllerOpts: {
		defaultLocale: 'en-US',
		locale: initialLocale,
		locales: [
			{
				tag: 'en-US',
				meta: {
					displayName: 'American English',
				},
			},
			{
				tag: 'ru-RU',
				meta: {
					displayName: 'Русский',
				},
			},
		],
	},
	globalMixin: true,
	injectInto: [],
})

const vintlController = VIntlPlugin.getOrCreateController()
vintlController.addEventListener('localeload', (event) => {
	const tag = event.locale.tag
	if (tag === 'ru-RU') {
		event.addMessages({
			...normalizeMessages(appRuMessages),
			...normalizeMessages(uiRuMessages),
		})
		return
	}
	event.addMessages({
		...normalizeMessages(appEnMessages),
		...normalizeMessages(uiEnMessages),
	})
})

const vueScan = new VueScanPlugin({
	enabled: false, // Enable or disable the tracker
	showOverlay: true, // Show overlay to visualize renders
	log: false, // Log render events to the console
	playSound: false, // Play sound on each render
})

const pinia = createPinia()

let app = createApp(App)

Sentry.init({
	app,
	dsn: '', // Disabled — original Modrinth Sentry DSN returns 403
	integrations: [Sentry.browserTracingIntegration({ router })],
	tracesSampleRate: 0.1,
	enabled: false,
})

app.use(VueQueryPlugin)
app.use(vueScan)
app.use(router)
app.use(pinia)
app.use(FloatingVue, {
	themes: {
		'ribbit-popout': {
			$extend: 'dropdown',
			placement: 'bottom-end',
			instantMove: true,
			distance: 8,
		},
	},
})
app.use(VIntlPlugin)

watch(
	launcherLanguage,
	async (language) => {
		const nextLocale = localeMap[language] ?? 'en-US'
		await vintlController.changeLocale(nextLocale)
	},
	{ immediate: true },
)

app.mount('#app')
