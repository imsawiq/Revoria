<script setup lang="ts">
import {
	GlobeIcon,
	KeyIcon,
	PlugIcon,
	ShieldIcon,
	SparklesIcon,
	UserIcon,
} from '@modrinth/assets'
import { Button, Toggle, injectNotificationManager } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref } from 'vue'

import {
	get,
	set,
	testProxy,
	type AppSettings,
	type ProxyTestResult,
	type ProxyType,
} from '@/helpers/settings.ts'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()

const messages = defineMessages({
	title: {
		id: 'settings.proxy.title',
		defaultMessage: 'Launcher proxy',
	},
	description: {
		id: 'settings.proxy.description',
		defaultMessage:
			'Route launcher network requests through a proxy. Changes apply immediately after saving.',
	},
	enableTitle: {
		id: 'settings.proxy.enable.title',
		defaultMessage: 'Enable global proxy',
	},
	enableDescription: {
		id: 'settings.proxy.enable.description',
		defaultMessage: 'Applies to launcher API calls, downloads, skins, auth, and content requests.',
	},
	typeTitle: {
		id: 'settings.proxy.type.title',
		defaultMessage: 'Proxy type',
	},
	typeDescription: {
		id: 'settings.proxy.type.description',
		defaultMessage: 'Choose the protocol your proxy server expects.',
	},
	hostTitle: {
		id: 'settings.proxy.host.title',
		defaultMessage: 'Host',
	},
	hostPlaceholder: {
		id: 'settings.proxy.host.placeholder',
		defaultMessage: '127.0.0.1 or proxy.example.com',
	},
	portTitle: {
		id: 'settings.proxy.port.title',
		defaultMessage: 'Port',
	},
	portPlaceholder: {
		id: 'settings.proxy.port.placeholder',
		defaultMessage: '8080',
	},
	authTitle: {
		id: 'settings.proxy.auth.title',
		defaultMessage: 'Authentication',
	},
	authDescription: {
		id: 'settings.proxy.auth.description',
		defaultMessage: 'Enable this if your proxy requires a username and password.',
	},
	usernameTitle: {
		id: 'settings.proxy.username.title',
		defaultMessage: 'Username',
	},
	usernamePlaceholder: {
		id: 'settings.proxy.username.placeholder',
		defaultMessage: 'proxy-user',
	},
	passwordTitle: {
		id: 'settings.proxy.password.title',
		defaultMessage: 'Password',
	},
	passwordPlaceholder: {
		id: 'settings.proxy.password.placeholder',
		defaultMessage: 'Enter password',
	},
	previewTitle: {
		id: 'settings.proxy.preview.title',
		defaultMessage: 'Applied route',
	},
	previewDisabled: {
		id: 'settings.proxy.preview.disabled',
		defaultMessage: 'Proxy is currently disabled.',
	},
	typeHttp: {
		id: 'settings.proxy.type.http',
		defaultMessage: 'HTTP',
	},
	typeHttpDescription: {
		id: 'settings.proxy.type.http.description',
		defaultMessage: 'Best for most standard proxy servers.',
	},
	typeHttps: {
		id: 'settings.proxy.type.https',
		defaultMessage: 'HTTPS',
	},
	typeHttpsDescription: {
		id: 'settings.proxy.type.https.description',
		defaultMessage: 'Use when the proxy endpoint itself is served over TLS.',
	},
	typeSocks5: {
		id: 'settings.proxy.type.socks5',
		defaultMessage: 'SOCKS5',
	},
	typeSocks5Description: {
		id: 'settings.proxy.type.socks5.description',
		defaultMessage: 'Good for tunneling all traffic through a single socket.',
	},
	save: {
		id: 'settings.proxy.actions.save',
		defaultMessage: 'Apply proxy',
	},
	reset: {
		id: 'settings.proxy.actions.reset',
		defaultMessage: 'Reset',
	},
	test: {
		id: 'settings.proxy.actions.test',
		defaultMessage: 'Test proxy',
	},
	validationHost: {
		id: 'settings.proxy.validation.host',
		defaultMessage: 'Enter a proxy host before enabling the proxy.',
	},
	validationPort: {
		id: 'settings.proxy.validation.port',
		defaultMessage: 'Proxy port must be between 1 and 65535.',
	},
	statusIdle: {
		id: 'settings.proxy.status.idle',
		defaultMessage: 'Run a test to verify that the launcher can reach Minecraft and Xbox services through this proxy.',
	},
	statusTesting: {
		id: 'settings.proxy.status.testing',
		defaultMessage: 'Testing proxy route...',
	},
	statusSuccess: {
		id: 'settings.proxy.status.success',
		defaultMessage: 'Proxy test passed.',
	},
	statusFailed: {
		id: 'settings.proxy.status.failed',
		defaultMessage: 'Proxy test failed.',
	},
	lastIp: {
		id: 'settings.proxy.status.ip',
		defaultMessage: 'Detected external IP',
	},
	lastMinecraft: {
		id: 'settings.proxy.status.minecraft',
		defaultMessage: 'Minecraft status',
	},
	lastXbox: {
		id: 'settings.proxy.status.xbox',
		defaultMessage: 'Xbox status',
	},
})

const settings = ref<AppSettings>(await get())
const draft = ref<AppSettings>(JSON.parse(JSON.stringify(settings.value)))
const isSaving = ref(false)
const isTesting = ref(false)
const lastTest = ref<ProxyTestResult | null>(null)
const lastTestError = ref<string | null>(null)

const proxyTypeOptions: Array<{
	value: ProxyType
	icon: typeof GlobeIcon
	label: () => string
	description: () => string
}> = [
	{
		value: 'http',
		icon: GlobeIcon,
		label: () => formatMessage(messages.typeHttp),
		description: () => formatMessage(messages.typeHttpDescription),
	},
	{
		value: 'https',
		icon: ShieldIcon,
		label: () => formatMessage(messages.typeHttps),
		description: () => formatMessage(messages.typeHttpsDescription),
	},
	{
		value: 'socks5',
		icon: SparklesIcon,
		label: () => formatMessage(messages.typeSocks5),
		description: () => formatMessage(messages.typeSocks5Description),
	},
]

const previewUrl = computed(() => {
	if (!draft.value.proxy_enabled) {
		return formatMessage(messages.previewDisabled)
	}

	const host = draft.value.proxy_host.trim()
	if (!host) return formatMessage(messages.previewDisabled)

	const auth =
		draft.value.proxy_auth_enabled && draft.value.proxy_username
			? `${draft.value.proxy_username}:${draft.value.proxy_password ? '••••••••' : ''}@`
			: ''

	return `${draft.value.proxy_type}://${auth}${host}:${draft.value.proxy_port}`
})

function resetDraft() {
	draft.value = JSON.parse(JSON.stringify(settings.value))
}

async function applyProxySettings() {
	try {
		if (draft.value.proxy_enabled && !draft.value.proxy_host.trim()) {
			throw new Error(formatMessage(messages.validationHost))
		}

		if (
			!Number.isInteger(Number(draft.value.proxy_port)) ||
			Number(draft.value.proxy_port) < 1 ||
			Number(draft.value.proxy_port) > 65535
		) {
			throw new Error(formatMessage(messages.validationPort))
		}

		isSaving.value = true
		const next = JSON.parse(JSON.stringify(draft.value)) as AppSettings
		next.proxy_host = next.proxy_host.trim()
		next.proxy_port = Number(next.proxy_port)
		await set(next)
		settings.value = next
		resetDraft()
	} catch (err) {
		handleError(err)
	} finally {
		isSaving.value = false
	}
}

async function runProxyTest() {
	try {
		if (draft.value.proxy_enabled && !draft.value.proxy_host.trim()) {
			throw new Error(formatMessage(messages.validationHost))
		}

		if (
			!Number.isInteger(Number(draft.value.proxy_port)) ||
			Number(draft.value.proxy_port) < 1 ||
			Number(draft.value.proxy_port) > 65535
		) {
			throw new Error(formatMessage(messages.validationPort))
		}

		isTesting.value = true
		lastTestError.value = null
		lastTest.value = null

		const next = JSON.parse(JSON.stringify(draft.value)) as AppSettings
		next.proxy_host = next.proxy_host.trim()
		next.proxy_port = Number(next.proxy_port)

		const result = await testProxy(next)
		lastTest.value = result
		addNotification({
			title: formatMessage(messages.statusSuccess),
			text: result.message,
			type: 'success',
		})
	} catch (err) {
		lastTestError.value = err instanceof Error ? err.message : String(err)
		handleError(err)
	} finally {
		isTesting.value = false
	}
}
</script>

<template>
	<div class="proxy-hero">
		<div class="proxy-hero__copy">
			<p class="proxy-hero__eyebrow">
				<PlugIcon />
				{{ formatMessage(messages.title) }}
			</p>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.enableTitle) }}
			</h2>
			<p class="m-0 mt-2 text-secondary">
				{{ formatMessage(messages.description) }}
			</p>
			<p class="m-0 mt-2 text-secondary">
				{{ formatMessage(messages.enableDescription) }}
			</p>
		</div>
		<div class="proxy-hero__toggle">
			<Toggle id="proxy-enabled" v-model="draft.proxy_enabled" />
		</div>
	</div>

	<div class="settings-row mt-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.typeTitle) }}
			</h2>
			<p class="m-0 mt-1 text-secondary">
				{{ formatMessage(messages.typeDescription) }}
			</p>
		</div>

		<div class="proxy-type-grid">
			<button
				v-for="option in proxyTypeOptions"
				:key="option.value"
				type="button"
				class="proxy-type-card"
				:class="{ 'proxy-type-card--selected': draft.proxy_type === option.value }"
				@click="draft.proxy_type = option.value"
			>
				<span class="proxy-type-card__icon">
					<component :is="option.icon" />
				</span>
				<span class="proxy-type-card__label">{{ option.label() }}</span>
				<span class="proxy-type-card__description">{{ option.description() }}</span>
			</button>
		</div>
	</div>

	<div class="proxy-form-grid mt-4">
		<label class="proxy-input-card">
			<span class="proxy-input-card__label">
				<GlobeIcon />
				{{ formatMessage(messages.hostTitle) }}
			</span>
			<input
				v-model="draft.proxy_host"
				class="proxy-input-card__input"
				type="text"
				:placeholder="formatMessage(messages.hostPlaceholder)"
				autocomplete="off"
				spellcheck="false"
			/>
		</label>

		<label class="proxy-input-card proxy-input-card--port">
			<span class="proxy-input-card__label">
				<PlugIcon />
				{{ formatMessage(messages.portTitle) }}
			</span>
			<input
				v-model.number="draft.proxy_port"
				class="proxy-input-card__input"
				type="number"
				min="1"
				max="65535"
				:placeholder="formatMessage(messages.portPlaceholder)"
			/>
		</label>
	</div>

	<div class="settings-row mt-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.authTitle) }}
			</h2>
			<p class="m-0 mt-1 text-secondary">
				{{ formatMessage(messages.authDescription) }}
			</p>
		</div>
		<Toggle id="proxy-auth-enabled" v-model="draft.proxy_auth_enabled" />
	</div>

	<div class="proxy-form-grid mt-4" :class="{ 'proxy-form-grid--disabled': !draft.proxy_auth_enabled }">
		<label class="proxy-input-card">
			<span class="proxy-input-card__label">
				<UserIcon />
				{{ formatMessage(messages.usernameTitle) }}
			</span>
			<input
				v-model="draft.proxy_username"
				class="proxy-input-card__input"
				type="text"
				:disabled="!draft.proxy_auth_enabled"
				:placeholder="formatMessage(messages.usernamePlaceholder)"
				autocomplete="off"
				spellcheck="false"
			/>
		</label>

		<label class="proxy-input-card">
			<span class="proxy-input-card__label">
				<KeyIcon />
				{{ formatMessage(messages.passwordTitle) }}
			</span>
			<input
				v-model="draft.proxy_password"
				class="proxy-input-card__input"
				type="password"
				:disabled="!draft.proxy_auth_enabled"
				:placeholder="formatMessage(messages.passwordPlaceholder)"
				autocomplete="new-password"
			/>
		</label>
	</div>

	<div class="proxy-preview mt-4">
		<p class="proxy-preview__label">{{ formatMessage(messages.previewTitle) }}</p>
		<code class="proxy-preview__value">{{ previewUrl }}</code>
	</div>

	<div
		class="proxy-test-panel mt-4"
		:class="{
			'proxy-test-panel--success': !!lastTest,
			'proxy-test-panel--error': !!lastTestError,
		}"
	>
		<p class="proxy-preview__label">
			{{ isTesting ? formatMessage(messages.statusTesting) : formatMessage(messages.statusIdle) }}
		</p>
		<p v-if="lastTest" class="proxy-test-panel__message">
			{{ lastTest.message }}
		</p>
		<p v-else-if="lastTestError" class="proxy-test-panel__message proxy-test-panel__message--error">
			{{ lastTestError }}
		</p>
		<div v-if="lastTest" class="proxy-test-panel__stats">
			<span v-if="lastTest.ip">
				<strong>{{ formatMessage(messages.lastIp) }}:</strong> {{ lastTest.ip }}
			</span>
			<span v-if="lastTest.minecraft_status">
				<strong>{{ formatMessage(messages.lastMinecraft) }}:</strong>
				{{ lastTest.minecraft_status }}
			</span>
			<span v-if="lastTest.xbox_status">
				<strong>{{ formatMessage(messages.lastXbox) }}:</strong>
				{{ lastTest.xbox_status }}
			</span>
		</div>
	</div>

	<div class="proxy-actions mt-4">
		<Button :disabled="isSaving" @click="applyProxySettings">
			<PlugIcon />
			{{ formatMessage(messages.save) }}
		</Button>
		<Button color="secondary" :disabled="isTesting" @click="runProxyTest">
			<ShieldIcon />
			{{ formatMessage(messages.test) }}
		</Button>
		<Button color="secondary" :disabled="isSaving" @click="resetDraft">
			{{ formatMessage(messages.reset) }}
		</Button>
	</div>
</template>

<style scoped lang="scss">
.settings-row,
.proxy-hero,
.proxy-preview,
.proxy-test-panel,
.proxy-input-card,
.proxy-type-card {
	border: 1px solid var(--glass-border);
	background:
		linear-gradient(
			145deg,
			color-mix(in srgb, var(--color-brand-highlight) 10%, transparent),
			transparent 55%
		),
		var(--color-glass-bg);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
}

.settings-row,
.proxy-hero,
.proxy-preview {
	border-radius: 1.1rem;
	padding: 1rem 1.1rem;
}

.proxy-hero {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 1rem;
}

.proxy-hero__eyebrow {
	display: inline-flex;
	align-items: center;
	gap: 0.45rem;
	margin: 0 0 0.55rem;
	color: var(--color-brand);
	font-size: 0.82rem;
	font-weight: 700;
	letter-spacing: 0.02em;
}

.proxy-hero__toggle {
	flex-shrink: 0;
	padding-top: 0.15rem;
}

.proxy-type-grid,
.proxy-form-grid {
	display: grid;
	gap: 0.85rem;
}

.proxy-type-grid {
	grid-template-columns: repeat(3, minmax(0, 1fr));
	margin-top: 1rem;
}

.proxy-type-card {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	gap: 0.55rem;
	padding: 1rem;
	border-radius: 1rem;
	text-align: left;
	transition:
		transform 160ms ease,
		border-color 160ms ease,
		background 160ms ease,
		box-shadow 160ms ease;
}

.proxy-type-card:hover {
	transform: translateY(-1px);
	border-color: color-mix(in srgb, var(--color-brand) 38%, var(--glass-border));
}

.proxy-type-card--selected {
	border-color: color-mix(in srgb, var(--color-brand) 64%, var(--glass-border));
	background:
		linear-gradient(
			155deg,
			color-mix(in srgb, var(--color-brand-highlight) 20%, transparent),
			color-mix(in srgb, var(--color-brand) 10%, transparent) 70%
		),
		var(--color-glass-bg-strong);
}

.proxy-type-card__icon {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 2.5rem;
	height: 2.5rem;
	border-radius: 0.85rem;
	background: color-mix(in srgb, var(--color-brand-highlight) 18%, var(--color-glass-bg));
	color: var(--color-brand);
}

.proxy-type-card__label {
	color: var(--color-contrast);
	font-weight: 700;
}

.proxy-type-card__description {
	color: var(--color-secondary);
	font-size: 0.9rem;
	line-height: 1.35;
}

.proxy-form-grid {
	grid-template-columns: repeat(2, minmax(0, 1fr));
}

.proxy-form-grid--disabled {
	opacity: 0.65;
}

.proxy-input-card {
	display: flex;
	flex-direction: column;
	gap: 0.65rem;
	padding: 0.95rem 1rem;
	border-radius: 1rem;
}

.proxy-input-card--port {
	max-width: 12rem;
}

.proxy-input-card__label {
	display: inline-flex;
	align-items: center;
	gap: 0.5rem;
	color: var(--color-secondary);
	font-size: 0.92rem;
	font-weight: 700;
}

.proxy-input-card__input {
	width: 100%;
	border: none;
	outline: none;
	background: transparent;
	color: var(--color-contrast);
	font: inherit;
	padding: 0;
}

.proxy-input-card__input::placeholder {
	color: var(--color-secondary);
}

.proxy-preview {
	display: flex;
	flex-direction: column;
	gap: 0.45rem;
}

.proxy-test-panel {
	display: flex;
	flex-direction: column;
	gap: 0.6rem;
	border-radius: 1.1rem;
	padding: 1rem 1.1rem;
}

.proxy-test-panel--success {
	border-color: color-mix(in srgb, var(--color-success, #3fb950) 40%, var(--glass-border));
}

.proxy-test-panel--error {
	border-color: color-mix(in srgb, var(--color-danger, #ff6b6b) 40%, var(--glass-border));
}

.proxy-test-panel__message {
	margin: 0;
	color: var(--color-contrast);
	line-height: 1.45;
}

.proxy-test-panel__message--error {
	color: var(--color-danger, #ff7a7a);
}

.proxy-test-panel__stats {
	display: flex;
	flex-wrap: wrap;
	gap: 0.85rem 1rem;
	color: var(--color-secondary);
	font-size: 0.92rem;
}

.proxy-preview__label {
	margin: 0;
	color: var(--color-secondary);
	font-size: 0.88rem;
	font-weight: 700;
}

.proxy-preview__value {
	overflow-wrap: anywhere;
	color: var(--color-contrast);
	font-size: 0.95rem;
}

.proxy-actions {
	display: flex;
	gap: 0.75rem;
}

@media (max-width: 860px) {
	.proxy-type-grid,
	.proxy-form-grid {
		grid-template-columns: 1fr;
	}

	.proxy-input-card--port {
		max-width: none;
	}
}
</style>
