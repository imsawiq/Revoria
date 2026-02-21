<script setup lang="ts">
import {
	EditIcon,
	ExcitedRinthbot,
	LogInIcon,
	PlusIcon,
	SpinnerIcon,
	TrashIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import {
	Button,
	ButtonStyled,
	ConfirmModal,
	injectNotificationManager,
	SkinButton,
	SkinLikeTextButton,
	SkinPreviewRenderer,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computedAsync } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, inject, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'

import type AccountsCard from '@/components/ui/AccountsCard.vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import SelectCapeModal from '@/components/ui/skin/SelectCapeModal.vue'
import UploadSkinModal from '@/components/ui/skin/UploadSkinModal.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_default_user, login as login_flow, users } from '@/helpers/auth'
import type { RenderResult } from '@/helpers/rendering/batch-skin-renderer.ts'
import { generateSkinPreviews, skinBlobUrlMap } from '@/helpers/rendering/batch-skin-renderer.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import type { Cape, Skin, SkinTextureUrl } from '@/helpers/skins.ts'
import {
	equip_skin,
	filterDefaultSkins,
	filterSavedSkins,
	get_available_capes,
	get_available_skins,
	get_normalized_skin_texture,
	normalize_skin_texture,
	remove_custom_skin,
	set_default_cape,
} from '@/helpers/skins.ts'
import { handleSevereError } from '@/store/error'
const editSkinModal = useTemplateRef('editSkinModal')
const selectCapeModal = useTemplateRef('selectCapeModal')
const uploadSkinModal = useTemplateRef('uploadSkinModal')

const notifications = injectNotificationManager()
const { handleError } = notifications
const { formatMessage } = useVIntl()

const messages = defineMessages({
	rateLimitTitle: {
		id: 'skins.rate-limit.title',
		defaultMessage: 'Slow down!',
	},
	rateLimitSkinText: {
		id: 'skins.rate-limit.skin',
		defaultMessage:
			"You're changing your skin too frequently. Mojang's servers have temporarily blocked further requests. Please wait a moment before trying again.",
	},
	rateLimitCapeText: {
		id: 'skins.rate-limit.cape',
		defaultMessage:
			"You're changing your cape too frequently. Mojang's servers have temporarily blocked further requests. Please wait a moment before trying again.",
	},
	deleteSkinTitle: {
		id: 'skins.delete.confirm.title',
		defaultMessage: 'Are you sure you want to delete this skin?',
	},
	deleteSkinDescription: {
		id: 'skins.delete.confirm.description',
		defaultMessage: 'This will permanently delete the selected skin. This action cannot be undone.',
	},
	deleteAction: {
		id: 'skins.action.delete',
		defaultMessage: 'Delete',
	},
	skinsTitle: {
		id: 'skins.title',
		defaultMessage: 'Skins',
	},
	beta: {
		id: 'skins.beta',
		defaultMessage: 'Beta',
	},
	overridingDefaultCapeTooltip: {
		id: 'skins.tooltip.overriding-default-cape',
		defaultMessage: 'The equipped skin is overriding the default cape.',
	},
	changeCape: {
		id: 'skins.action.change-cape',
		defaultMessage: 'Change cape',
	},
	savedSkins: {
		id: 'skins.saved.heading',
		defaultMessage: 'Saved skins',
	},
	addSkin: {
		id: 'skins.action.add-skin',
		defaultMessage: 'Add a skin',
	},
	editSkinAria: {
		id: 'skins.action.edit-skin.aria',
		defaultMessage: 'Edit skin',
	},
	edit: {
		id: 'skins.action.edit',
		defaultMessage: 'Edit',
	},
	deleteSkinAria: {
		id: 'skins.action.delete-skin.aria',
		defaultMessage: 'Delete skin',
	},
	deleteSkinTooltip: {
		id: 'skins.action.delete-skin.tooltip',
		defaultMessage: 'Delete skin',
	},
	defaultSkins: {
		id: 'skins.default.heading',
		defaultMessage: 'Default skins',
	},
	pleaseSignIn: {
		id: 'skins.sign-in.title',
		defaultMessage: 'Please sign-in',
	},
	signInDescription: {
		id: 'skins.sign-in.description',
		defaultMessage:
			'Please sign into your Minecraft account to use the skin management features of the Modrinth app.',
	},
	signIn: {
		id: 'skins.action.sign-in',
		defaultMessage: 'Sign In',
	},
})

const settings = ref(await getSettings())
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])

type Account = {
	profile?: {
		id: string
		name: string
	}
}

const accountsCard = inject('accountsCard') as Ref<typeof AccountsCard | null> | undefined
const currentUser = ref<Account | undefined>(undefined)
const currentUserId = ref<string | undefined>(undefined)

const isAccountsLoginDisabled = computed(() => {
	const rawState = (accountsCard?.value as { microsoftLoginDisabled?: boolean | Ref<boolean> } | null)
		?.microsoftLoginDisabled

	if (typeof rawState === 'object' && rawState !== null && 'value' in rawState) {
		return Boolean(rawState.value)
	}

	return Boolean(rawState)
})

const username = computed(() => currentUser.value?.profile?.name ?? undefined)
const selectedSkin = ref<Skin | null>(null)
const defaultCape = ref<Cape>()

const originalSelectedSkin = ref<Skin | null>(null)
const originalDefaultCape = ref<Cape>()

const savedSkins = computed(() => {
	try {
		return filterSavedSkins(skins.value)
	} catch (error) {
		handleError(error as Error)
		return []
	}
})
const defaultSkins = computed(() => filterDefaultSkins(skins.value))

const currentCape = computed(() => {
	if (selectedSkin.value?.cape_id) {
		const overrideCape = capes.value.find((c) => c.id === selectedSkin.value?.cape_id)
		if (overrideCape) {
			return overrideCape
		}
	}
	return defaultCape.value
})

const skinTexture = computedAsync(async () => {
	if (selectedSkin.value?.texture) {
		return await get_normalized_skin_texture(selectedSkin.value)
	} else {
		return ''
	}
})
const capeTexture = computed(() => currentCape.value?.texture)
const skinVariant = computed(() => selectedSkin.value?.variant)
const skinNametag = computed(() =>
	settings.value.hide_nametag_skins_page ? undefined : username.value,
)

let userCheckInterval: number | null = null

const deleteSkinModal = ref()
const skinToDelete = ref<Skin | null>(null)

function confirmDeleteSkin(skin: Skin) {
	skinToDelete.value = skin
	deleteSkinModal.value?.show()
}

async function deleteSkin() {
	if (!skinToDelete.value) return
	await remove_custom_skin(skinToDelete.value).catch(handleError)
	await loadSkins()
	skinToDelete.value = null
}

async function loadCapes() {
	try {
		capes.value = (await get_available_capes()) ?? []
		defaultCape.value = capes.value.find((c) => c.is_equipped)
		originalDefaultCape.value = defaultCape.value
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

async function loadSkins() {
	try {
		skins.value = (await get_available_skins()) ?? []
		generateSkinPreviews(skins.value, capes.value)
		selectedSkin.value = skins.value.find((s) => s.is_equipped) ?? null
		originalSelectedSkin.value = selectedSkin.value
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

async function changeSkin(newSkin: Skin) {
	const previousSkin = selectedSkin.value
	const previousSkinsList = [...skins.value]

	skins.value = skins.value.map((skin) => {
		return {
			...skin,
			is_equipped: skin.texture_key === newSkin.texture_key,
		}
	})

	selectedSkin.value = skins.value.find((s) => s.texture_key === newSkin.texture_key) || null

	try {
		await equip_skin(newSkin)
		if (accountsCard.value) {
			await accountsCard.value.refreshValues()
		}
	} catch (error) {
		selectedSkin.value = previousSkin
		skins.value = previousSkinsList

		if ((error as { message?: string })?.message?.includes('429 Too Many Requests')) {
			notifications.addNotification({
				type: 'error',
				title: formatMessage(messages.rateLimitTitle),
				text: formatMessage(messages.rateLimitSkinText),
			})
		} else {
			handleError(error as Error)
		}
	}
}

async function handleCapeSelected(cape: Cape | undefined) {
	const previousDefaultCape = defaultCape.value
	const previousCapesList = [...capes.value]

	capes.value = capes.value.map((c) => ({
		...c,
		is_equipped: cape ? c.id === cape.id : false,
	}))

	defaultCape.value = cape ? capes.value.find((c) => c.id === cape.id) : undefined

	try {
		await set_default_cape(cape)
	} catch (error) {
		defaultCape.value = previousDefaultCape
		capes.value = previousCapesList

		if ((error as { message?: string })?.message?.includes('429 Too Many Requests')) {
			notifications.addNotification({
				type: 'error',
				title: formatMessage(messages.rateLimitTitle),
				text: formatMessage(messages.rateLimitCapeText),
			})
		} else {
			handleError(error as Error)
		}
	}
}

async function onSkinSaved() {
	await Promise.all([loadCapes(), loadSkins()])
}

async function loadCurrentUser() {
	try {
		const defaultId = await get_default_user()
		currentUserId.value = defaultId

		const allAccounts = await users()
		currentUser.value = allAccounts.find((acc) => acc.profile?.id === defaultId)
	} catch (e) {
		handleError(e as Error)
		currentUser.value = undefined
		currentUserId.value = undefined
	}
}

function getBakedSkinTextures(skin: Skin): RenderResult | undefined {
	const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`
	return skinBlobUrlMap.get(key)
}

async function login() {
	if (!accountsCard?.value) return

	accountsCard.value.setLoginDisabled(true)
	try {
		const loggedIn = await login_flow().catch(handleSevereError)

		if (loggedIn) {
			await accountsCard.value.refreshValues()
		}

		trackEvent('AccountLogIn')
	} finally {
		accountsCard.value?.setLoginDisabled(false)
	}
}

function openUploadSkinModal() {
	uploadSkinModal.value?.show()
}

function onSkinFileUploaded(buffer: ArrayBuffer) {
	const fakeEvent = new MouseEvent('click')
	const originalSkinTexUrl = `data:image/png;base64,` + arrayBufferToBase64(buffer)
	normalize_skin_texture(originalSkinTexUrl).then((skinTextureNormalized: Uint8Array) => {
		const skinTexUrl: SkinTextureUrl = {
			original: originalSkinTexUrl,
			normalized: `data:image/png;base64,` + arrayBufferToBase64(skinTextureNormalized),
		}
		if (editSkinModal.value && editSkinModal.value.shouldRestoreModal) {
			editSkinModal.value.restoreWithNewTexture(skinTexUrl)
		} else {
			editSkinModal.value?.showNew(fakeEvent, skinTexUrl)
		}
	})
}

function onUploadCanceled() {
	editSkinModal.value?.restoreModal()
}

watch(
	() => selectedSkin.value?.cape_id,
	() => {},
)

onMounted(() => {
	userCheckInterval = window.setInterval(checkUserChanges, 250)
})

onUnmounted(() => {
	if (userCheckInterval !== null) {
		window.clearInterval(userCheckInterval)
	}
})

async function checkUserChanges() {
	try {
		const defaultId = await get_default_user()
		if (defaultId !== currentUserId.value) {
			await loadCurrentUser()
			await loadCapes()
			await loadSkins()
		}
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

onMounted(async () => {
	await Promise.all([loadCapes(), loadSkins(), loadCurrentUser()])
})
</script>

<template>
	<EditSkinModal
		ref="editSkinModal"
		:capes="capes"
		:default-cape="defaultCape"
		@saved="onSkinSaved"
		@deleted="() => loadSkins()"
		@open-upload-modal="openUploadSkinModal"
	/>
	<SelectCapeModal ref="selectCapeModal" :capes="capes" @select="handleCapeSelected" />
	<UploadSkinModal
		ref="uploadSkinModal"
		@uploaded="onSkinFileUploaded"
		@canceled="onUploadCanceled"
	/>
	<ConfirmModal
		ref="deleteSkinModal"
		:title="formatMessage(messages.deleteSkinTitle)"
		:description="formatMessage(messages.deleteSkinDescription)"
		:proceed-label="formatMessage(messages.deleteAction)"
		@proceed="deleteSkin"
	/>

	<div v-if="currentUser" class="p-6 skin-layout">
		<div class="preview-panel skin-panel">
			<h1 class="m-0 text-2xl font-bold flex items-center gap-2">
				{{ formatMessage(messages.skinsTitle) }}
				<span class="text-sm font-bold px-2 bg-brand-highlight text-brand rounded-full">{{ formatMessage(messages.beta) }}</span>
			</h1>
			<div class="preview-container skin-preview-surface">
				<SkinPreviewRenderer
					:cape-src="capeTexture"
					:texture-src="skinTexture || ''"
					:variant="skinVariant"
					:nametag="skinNametag"
					:initial-rotation="Math.PI / 8"
				>
					<template #subtitle>
						<ButtonStyled :disabled="!!selectedSkin?.cape_id">
							<button
								v-tooltip="
									selectedSkin?.cape_id
										? formatMessage(messages.overridingDefaultCapeTooltip)
										: undefined
								"
								:disabled="!!selectedSkin?.cape_id"
								@click="
									(e: MouseEvent) =>
										selectCapeModal?.show(
											e,
											selectedSkin?.texture_key,
											currentCape,
											skinTexture,
											skinVariant,
										)
								"
							>
								<UpdatedIcon />
								{{ formatMessage(messages.changeCape) }}
							</button>
						</ButtonStyled>
					</template>
				</SkinPreviewRenderer>
			</div>
		</div>

		<div class="skins-container skin-panel">
			<section class="flex flex-col gap-2 mt-1">
				<h2 class="text-lg font-bold m-0 text-primary">{{ formatMessage(messages.savedSkins) }}</h2>
				<div class="skin-card-grid">
					<div class="skin-card">
						<SkinLikeTextButton class="skin-card-inner" @click="openUploadSkinModal">
							<template #icon>
								<PlusIcon class="size-8" />
							</template>
							<span>{{ formatMessage(messages.addSkin) }}</span>
						</SkinLikeTextButton>
					</div>

					<div v-for="skin in savedSkins" :key="`saved-skin-${skin.texture_key}`" class="skin-card">
						<SkinButton
							class="skin-card-inner"
							:forward-image-src="getBakedSkinTextures(skin)?.forwards"
							:backward-image-src="getBakedSkinTextures(skin)?.backwards"
							:selected="selectedSkin === skin"
							@select="changeSkin(skin)"
						>
							<template #overlay-buttons>
								<Button
									color="green"
									:aria-label="formatMessage(messages.editSkinAria)"
									class="pointer-events-auto"
									@click.stop="(e: MouseEvent) => editSkinModal?.show(e, skin)"
								>
									<EditIcon /> {{ formatMessage(messages.edit) }}
								</Button>
								<Button
									v-show="!skin.is_equipped"
									:v-tooltip="formatMessage(messages.deleteSkinTooltip)"
									:aria-label="formatMessage(messages.deleteSkinAria)"
									color="red"
									class="!rounded-[100%] pointer-events-auto"
									icon-only
									@click.stop="() => confirmDeleteSkin(skin)"
								>
									<TrashIcon />
								</Button>
							</template>
						</SkinButton>
					</div>
				</div>
			</section>

			<section class="flex flex-col gap-2 mt-6">
				<h2 class="text-lg font-bold m-0 text-primary">{{ formatMessage(messages.defaultSkins) }}</h2>
				<div class="skin-card-grid">
					<div
						v-for="skin in defaultSkins"
						:key="`default-skin-${skin.texture_key}`"
						class="skin-card"
					>
						<SkinButton
							class="skin-card-inner"
							:forward-image-src="getBakedSkinTextures(skin)?.forwards"
							:backward-image-src="getBakedSkinTextures(skin)?.backwards"
							:selected="selectedSkin === skin"
							:tooltip="skin.name"
							@select="changeSkin(skin)"
						/>
					</div>
				</div>
			</section>
		</div>
	</div>

	<div v-else class="flex items-center justify-center min-h-[50vh] pt-[25%]">
		<div
			class="bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] rounded-lg p-7 flex flex-col gap-5 relative max-w-xl w-full mx-auto"
		>
			<img
				:src="ExcitedRinthbot"
				alt="Excited Modrinth Bot"
				class="absolute -top-28 right-8 md:right-20 h-28 w-auto"
			/>
			<div
				class="absolute top-0 left-0 w-full h-[1px] opacity-40 bg-gradient-to-r from-transparent via-green-500 to-transparent"
				style="
					background: linear-gradient(
						to right,
						transparent 2rem,
						var(--color-green) calc(100% - 13rem),
						var(--color-green) calc(100% - 5rem),
						transparent calc(100% - 2rem)
					);
				"
			></div>

			<div class="flex flex-col gap-5">
				<h1 class="text-3xl font-extrabold m-0">{{ formatMessage(messages.pleaseSignIn) }}</h1>
				<p class="text-lg m-0">
					{{ formatMessage(messages.signInDescription) }}
				</p>
				<ButtonStyled v-show="!!accountsCard" color="brand" :disabled="isAccountsLoginDisabled">
					<button :disabled="isAccountsLoginDisabled" @click="login">
						<LogInIcon v-if="!isAccountsLoginDisabled" />
						<SpinnerIcon v-else class="animate-spin" />
						{{ formatMessage(messages.signIn) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<style lang="scss" scoped>
$skin-card-width: 155px;
$skin-card-gap: 4px;

.skin-layout {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(0, 2.5fr);
	gap: 2.5rem;

	@media (max-width: 700px) {
		grid-template-columns: 1fr;
	}
}

.preview-panel {
	top: 1.5rem;
	position: sticky;
	align-self: start;
	padding: 1rem;
}

.preview-container {
	height: 80vh;
	display: flex;
	align-items: center;
	justify-content: center;
	margin-top: 0.75rem;
	border-radius: 1rem;
	overflow: hidden;
	background: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);

	@media (max-width: 700px) {
		height: 50vh;
	}
}

.skin-preview-surface {
	position: relative;
}

.skin-preview-surface::before {
	content: '';
	position: absolute;
	inset: 0;
	background:
		radial-gradient(700px 220px at 10% 0%, rgba(255, 255, 255, 0.12), transparent 60%),
		radial-gradient(520px 240px at 90% 0%, rgba(27, 217, 106, 0.12), transparent 62%);
	opacity: 0.9;
	pointer-events: none;
}

.skins-container {
	padding-top: 0.5rem;
}

.skin-card-grid {
	display: grid;
	grid-template-columns: repeat(3, 1fr);
	gap: $skin-card-gap;
	width: 100%;

	@media (min-width: 1300px) {
		grid-template-columns: repeat(4, 1fr);
	}

	@media (min-width: 1750px) {
		grid-template-columns: repeat(5, 1fr);
	}

	@media (min-width: 2050px) {
		grid-template-columns: repeat(6, 1fr);
	}
}

.skin-card {
	aspect-ratio: 0.95;
	border-radius: 10px;
	box-sizing: border-box;
	width: 100%;
	min-width: 0;
	overflow: hidden;
	background: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	transform: translateZ(0);
	transition:
		transform 220ms cubic-bezier(0.2, 0.9, 0.2, 1),
		box-shadow 220ms cubic-bezier(0.2, 0.9, 0.2, 1),
		border-color 180ms ease;
}

.skin-card-inner {
	width: 100%;
	height: 100%;
	background: transparent !important;
	background-color: transparent !important;
	border: 0 !important;
	box-shadow: none !important;
}

:deep(.skin-card > *) {
	width: 100%;
	height: 100%;
}

:deep(.skin-card button),
:deep(.skin-card a) {
	background: transparent !important;
	background-color: transparent !important;
	border: 0 !important;
	box-shadow: none !important;
}

.skin-card:hover {
	transform: translateY(-2px);
	box-shadow: var(--shadow-floating);
	border-color: rgba(27, 217, 106, 0.22);
}

.skin-card:active {
	transform: translateY(-1px) scale(0.985);
}

@media (prefers-reduced-motion: reduce) {
	.skin-card {
		transition: none;
	}
}
</style>
