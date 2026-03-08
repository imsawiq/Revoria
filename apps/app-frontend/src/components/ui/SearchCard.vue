<template>
	<ProjectCard
		:title="project.title ?? project.name ?? 'Unknown project'"
		:link="projectLink"
		:author="{ name: authorName, link: authorHref }"
		:icon-url="project.icon_url"
		:summary="project.description ?? project.summary ?? ''"
		:tags="displayTags"
		:all-tags="project.categories ?? []"
		:downloads="downloadsCount"
		:followers="followsCount"
		:date-updated="updatedAt"
		:date-published="createdAt"
		:banner="project.featured_gallery ?? undefined"
		:color="project.color ?? undefined"
		:environment="environment"
		layout="list"
		@click="
			emit('open')
		"
	>
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
	</ProjectCard>
</template>

<script setup>
import { CheckIcon, DownloadIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager, ProjectCard } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { install as installVersion } from '@/store/install.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	installing: {
		id: 'search-card.installing',
		defaultMessage: 'Installing',
	},
	installed: {
		id: 'search-card.installed',
		defaultMessage: 'Installed',
	},
	install: {
		id: 'search-card.install',
		defaultMessage: 'Install',
	},
	addToInstance: {
		id: 'search-card.add-to-instance',
		defaultMessage: 'Add to an instance',
	},
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
	projectType: {
		type: String,
		default: undefined,
	},
	tagDefinitions: {
		type: Object,
		default: () => ({}),
	},
})

const emit = defineEmits(['open', 'install'])

const installing = ref(false)
const authorName = computed(() =>
	typeof props.project.author === 'string' ? props.project.author : '',
)
const authorHref = computed(() =>
	authorName.value ? `https://modrinth.com/user/${authorName.value}` : undefined,
)
const loaderIndex = computed(() => {
	const map = new Map()
	for (const loader of props.tagDefinitions?.loaders ?? []) {
		if (loader?.name) map.set(String(loader.name).toLowerCase(), loader)
	}
	return map
})
const categoryIndex = computed(() => {
	const map = new Map()
	for (const category of props.tagDefinitions?.categories ?? []) {
		if (category?.name) map.set(String(category.name).toLowerCase(), category)
	}
	return map
})
const displayTags = computed(() =>
	(props.project.display_categories ?? props.project.categories ?? [])
		.map((rawTag) => String(rawTag).trim())
		.filter((tag) => tag.length > 0)
		.map((tag) => {
			const key = tag.toLowerCase()
			const loader = loaderIndex.value.get(key)
			const category = categoryIndex.value.get(key)
			const matched = loader ?? category
			return {
				name: tag,
				icon: typeof matched?.icon === 'string' ? matched.icon : '',
				isLoader: Boolean(loader),
			}
		}),
)
const toFiniteNumber = (value) => {
	const numeric = Number(value)
	return Number.isFinite(numeric) ? numeric : null
}
const downloadsCount = computed(() => toFiniteNumber(props.project.downloads))
const followsCount = computed(() => toFiniteNumber(props.project.follows))
const createdAt = computed(
	() => props.project.date_created ?? props.project.date_modified ?? '1970-01-01T00:00:00.000Z',
)
const updatedAt = computed(() => props.project.date_modified ?? createdAt.value)
const projectLink = computed(() => {
	const projectId = props.project.project_id ?? props.project.id
	if (props.project?.source === 'curseforge') {
		return () =>
			router.push({
				path: `/project/${projectId}`,
				query: {
					source: 'curseforge',
					t: props.projectType ?? props.project.project_type ?? 'mod',
				},
			})
	}
	return `/project/${projectId}`
})
const environment = computed(() =>
	props.projectType && ['mod', 'modpack'].includes(props.projectType)
		? {
				clientSide: props.project.client_side,
				serverSide: props.project.server_side,
			}
		: undefined,
)

async function install() {
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
