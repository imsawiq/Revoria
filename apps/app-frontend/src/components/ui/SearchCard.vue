<template>
	<div
		class="p-4 bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] rounded-xl flex gap-3 group cursor-pointer transition-colors hover:bg-[--color-button-bg-hover]"
		@click="
			() => {
				emit('open')
				$router.push({
					path: `/project/${project.project_id ?? project.id}`,
					query: {
						source: project.source === 'curseforge' ? 'curseforge' : undefined,
						t: project.source === 'curseforge' ? project.project_type : undefined,
						i: props.instance ? props.instance.path : undefined,
					},
				})
			}
		"
	>
		<div class="icon w-[96px] h-[96px] relative">
			<Avatar :src="project.icon_url" size="96px" class="search-icon origin-top transition-all" />
		</div>
		<div class="flex flex-col gap-2 overflow-hidden">
			<div class="gap-2 overflow-hidden no-wrap text-ellipsis">
				<span class="text-lg font-extrabold text-contrast m-0 leading-none">
					{{ project.title }}
				</span>
				<span v-if="project.author" class="text-secondary"> by {{ project.author }}</span>
			</div>
			<div class="m-0 line-clamp-2">
				{{ project.description }}
			</div>
			<div v-if="categories.length > 0" class="mt-auto flex items-center gap-1 no-wrap">
				<TagsIcon class="h-4 w-4 shrink-0" />
				<div
					v-if="project.project_type === 'mod' || project.project_type === 'modpack'"
					class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
				>
					<template v-if="project.client_side === 'optional' && project.server_side === 'optional'">
						{{ formatMessage(messages.clientOrServer) }}
					</template>
					<template
						v-else-if="
							(project.client_side === 'optional' || project.client_side === 'required') &&
							(project.server_side === 'optional' || project.server_side === 'unsupported')
						"
					>
						{{ formatMessage(messages.client) }}
					</template>
					<template
						v-else-if="
							(project.server_side === 'optional' || project.server_side === 'required') &&
							(project.client_side === 'optional' || project.client_side === 'unsupported')
						"
					>
						{{ formatMessage(messages.server) }}
					</template>
					<template
						v-else-if="
							project.client_side === 'unsupported' && project.server_side === 'unsupported'
						"
					>
						{{ formatMessage(messages.unsupported) }}
					</template>
					<template
						v-else-if="project.client_side === 'required' && project.server_side === 'required'"
					>
						{{ formatMessage(messages.clientAndServer) }}
					</template>
				</div>
				<div
					v-for="tag in categories"
					:key="tag"
					class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
				>
					{{ formatCategory(tag.name) }}
				</div>
			</div>
		</div>
		<div class="flex flex-col gap-2 items-end shrink-0 ml-auto">
			<div class="flex items-center gap-2">
				<DownloadIcon class="shrink-0" />
				<span>
					{{ formatNumber(project.downloads) }}
					<span class="text-secondary">{{ formatMessage(messages.downloads) }}</span>
				</span>
			</div>
			<div class="flex items-center gap-2">
				<HeartIcon class="shrink-0" />
				<span>
					{{ formatNumber(project.follows ?? project.followers) }}
					<span class="text-secondary">{{ formatMessage(messages.followers) }}</span>
				</span>
			</div>
			<div class="mt-auto relative">
				<div class="absolute bottom-0 right-0 w-fit">
					<ButtonStyled color="brand" type="outlined">
						<button
							:disabled="installed || installing"
							class="shrink-0 no-wrap"
							@click.stop="install()"
						>
							<template v-if="!installed">
								<DownloadIcon v-if="modpack || instance" />
								<PlusIcon v-else />
							</template>
							<CheckIcon v-else />
							{{
								installing
									? formatMessage(messages.installing)
									: installed
										? formatMessage(messages.installed)
										: modpack || instance
											? formatMessage(messages.install)
											: formatMessage(messages.addToInstance)
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import { CheckIcon, DownloadIcon, HeartIcon, PlusIcon, TagsIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import { formatCategory, formatNumber } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { install as installVersion } from '@/store/install.js'
dayjs.extend(relativeTime)

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	clientOrServer: { id: 'search-card.compat.client-or-server', defaultMessage: 'Client or server' },
	client: { id: 'search-card.compat.client', defaultMessage: 'Client' },
	server: { id: 'search-card.compat.server', defaultMessage: 'Server' },
	unsupported: { id: 'search-card.compat.unsupported', defaultMessage: 'Unsupported' },
	clientAndServer: { id: 'search-card.compat.client-and-server', defaultMessage: 'Client and server' },
	downloads: { id: 'search-card.downloads', defaultMessage: 'downloads' },
	followers: { id: 'search-card.followers', defaultMessage: 'followers' },
	installing: { id: 'search-card.installing', defaultMessage: 'Installing' },
	installed: { id: 'search-card.installed', defaultMessage: 'Installed' },
	install: { id: 'search-card.install', defaultMessage: 'Install' },
	addToInstance: { id: 'search-card.add-to-instance', defaultMessage: 'Add to an instance' },
})

const router = useRouter()

const props = defineProps({
	backgroundImage: {
		type: String,
		default: null,
	},
	project: {
		type: Object,
		required: true,
	},
	categories: {
		type: Array,
		required: true,
	},
	instance: {
		type: Object,
		default: null,
	},
	featured: {
		type: Boolean,
		default: false,
	},
	installed: {
		type: Boolean,
		default: false,
	},
})

const emit = defineEmits(['open', 'install'])

const installing = ref(false)

async function install() {
	if (props.project?.source === 'curseforge') {
		emit('install', props.project.project_id ?? props.project.id)
		return
	}
	installing.value = true
	await installVersion(
		props.project.project_id ?? props.project.id,
		null,
		props.instance ? props.instance.path : null,
		'SearchCard',
		() => {
			installing.value = false
			emit('install', props.project.project_id ?? props.project.id)
		},
		(profile) => {
			router.push(`/instance/${profile}`)
		},
	).catch(handleError)
}

const modpack = computed(() => props.project.project_type === 'modpack')
</script>
