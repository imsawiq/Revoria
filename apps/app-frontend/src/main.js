import 'floating-vue/dist/style.css'

import * as Sentry from '@sentry/vue'
import { VueScanPlugin } from '@taijased/vue-render-tracker'
import { VueQueryPlugin } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'
import { createPlugin } from '@vintl/vintl/plugin'
import { useStorage } from '@vueuse/core'
import FloatingVue from 'floating-vue'
import { createPinia } from 'pinia'
import { createApp, watch } from 'vue'

import App from '@/App.vue'
import allDeDeMessages from '@/locales/combined/de-DE.json'
import allEnMessages from '@/locales/combined/en-US.json'
import allRoMessages from '@/locales/combined/ro-RO.json'
import allRuMessages from '@/locales/combined/ru-RU.json'
import allUkMessages from '@/locales/combined/uk-UA.json'
import router from '@/routes'

const launcherLanguage = useStorage('launcher-language', 'en')
const localeMap = {
	en: 'en-US',
	ru: 'ru-RU',
	uk: 'uk-UA',
	de: 'de-DE',
	ro: 'ro-RO',
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
			{
				tag: 'uk-UA',
				meta: {
					displayName: 'Українська',
				},
			},
			{
				tag: 'de-DE',
				meta: {
					displayName: 'Deutsch (Deutschland)',
				},
			},
			{
				tag: 'ro-RO',
				meta: {
					displayName: 'Română',
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
	const messagesByLocale = {
		'en-US': allEnMessages,
		'ru-RU': allRuMessages,
		'uk-UA': allUkMessages,
		'de-DE': allDeDeMessages,
		'ro-RO': allRoMessages,
	}
	const messages = messagesByLocale[tag] ?? allEnMessages
	event.addMessages(normalizeMessages(messages))
})

const vueScan = new VueScanPlugin({
	enabled: false, // Enable or disable the tracker
	showOverlay: true, // Show overlay to visualize renders
	log: false, // Log render events to the console
	playSound: false, // Play sound on each render
})

const pinia = createPinia()

let app = createApp(App)

function logFrontendError(message, source, stack) {
	void invoke('log_frontend_error', {
		message: String(message ?? 'Unknown frontend error'),
		source: source ? String(source) : null,
		stack: stack ? String(stack) : null,
	}).catch(() => {})
}

window.addEventListener('error', (event) => {
	logFrontendError(event.message, event.filename, event.error?.stack)
})

window.addEventListener('unhandledrejection', (event) => {
	const reason = event.reason
	logFrontendError(reason?.message ?? reason, 'unhandledrejection', reason?.stack)
})

app.config.errorHandler = (error, instance, info) => {
	logFrontendError(error?.message ?? error, info, error?.stack)
	console.error(error, instance, info)
}

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

let lastLocale = initialLocale
watch(
	launcherLanguage,
	async (language) => {
		const nextLocale = localeMap[language] ?? 'en-US'
		if (nextLocale === lastLocale) return
		lastLocale = nextLocale
		await vintlController.changeLocale(nextLocale)
	},
	{ immediate: false },
)

app.mount('#app')
