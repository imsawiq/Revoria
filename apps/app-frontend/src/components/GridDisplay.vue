<script setup>
import {
	ClipboardCopyIcon,
	EyeIcon,
	FolderOpenIcon,
	PlayIcon,
	PlusIcon,
	SearchIcon,
	StopCircleIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, DropdownSelect, injectNotificationManager } from '@modrinth/ui'
import { formatCategoryHeader } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import { computed, ref } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import Instance from '@/components/ui/Instance.vue'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { duplicate, remove } from '@/helpers/profile.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	libraryFallback: { id: 'grid-display.library-fallback', defaultMessage: 'Library' },
	instanceCountOne: { id: 'grid-display.instance-count.one', defaultMessage: '{count} instance' },
	instanceCountFew: { id: 'grid-display.instance-count.few', defaultMessage: '{count} instances' },
	instanceCountMany: {
		id: 'grid-display.instance-count.many',
		defaultMessage: '{count} instances',
	},
	instanceCountOther: {
		id: 'grid-display.instance-count.other',
		defaultMessage: '{count} instances',
	},
	searchPlaceholder: { id: 'grid-display.search', defaultMessage: 'Search' },
	sortDropdownName: { id: 'grid-display.sort-dropdown-name', defaultMessage: 'Sort Dropdown' },
	groupDropdownName: { id: 'grid-display.group-dropdown-name', defaultMessage: 'Group Dropdown' },
	placeholderSelect: { id: 'grid-display.select-placeholder', defaultMessage: 'Select...' },
	labelSort: { id: 'grid-display.label.sort', defaultMessage: 'Sort' },
	labelGroup: { id: 'grid-display.label.group', defaultMessage: 'Group' },
	optionName: { id: 'grid-display.option.name', defaultMessage: 'Name' },
	optionLastPlayed: { id: 'grid-display.option.last-played', defaultMessage: 'Last played' },
	optionDateCreated: { id: 'grid-display.option.date-created', defaultMessage: 'Date created' },
	optionDateModified: { id: 'grid-display.option.date-modified', defaultMessage: 'Date modified' },
	optionGameVersion: { id: 'grid-display.option.game-version', defaultMessage: 'Game version' },
	optionGroup: { id: 'grid-display.option.group', defaultMessage: 'Group' },
	optionLoader: { id: 'grid-display.option.loader', defaultMessage: 'Loader' },
	optionNone: { id: 'grid-display.option.none', defaultMessage: 'None' },
	deleteInstanceTitle: {
		id: 'grid-display.delete-instance.title',
		defaultMessage: 'Are you sure you want to delete this instance?',
	},
	deleteInstanceDescription: {
		id: 'grid-display.delete-instance.description',
		defaultMessage:
			'If you proceed, all data for your instance will be removed. You will not be able to recover it.',
	},
	delete: { id: 'delete', defaultMessage: 'Delete' },
	play: { id: 'instance.index.play', defaultMessage: 'Play' },
	stop: { id: 'instance.index.stop', defaultMessage: 'Stop' },
	addContent: { id: 'instance.index.add-content', defaultMessage: 'Add content' },
	viewInstance: { id: 'row-display.view-instance', defaultMessage: 'View instance' },
	duplicateInstance: { id: 'row-display.duplicate-instance', defaultMessage: 'Duplicate instance' },
	openFolder: { id: 'action.open-folder', defaultMessage: 'Open folder' },
	copyPath: { id: 'instance.index.copy-path', defaultMessage: 'Copy path' },
})

const sortOptions = computed(() => [
	formatMessage(messages.optionName),
	formatMessage(messages.optionLastPlayed),
	formatMessage(messages.optionDateCreated),
	formatMessage(messages.optionDateModified),
	formatMessage(messages.optionGameVersion),
])

const groupOptions = computed(() => [
	formatMessage(messages.optionGroup),
	formatMessage(messages.optionLoader),
	formatMessage(messages.optionGameVersion),
	formatMessage(messages.optionNone),
])

const toolbarCount = computed(() => {
	const count = props.instances.length
	const category = new Intl.PluralRules(navigator.language || 'en').select(count)

	if (category === 'one') return formatMessage(messages.instanceCountOne, { count })
	if (category === 'few') return formatMessage(messages.instanceCountFew, { count })
	if (category === 'many') return formatMessage(messages.instanceCountMany, { count })
	return formatMessage(messages.instanceCountOther, { count })
})

const props = defineProps({
	instances: {
		type: Array,
		default() {
			return []
		},
	},
	label: {
		type: String,
		default: '',
	},
})
const instanceOptions = ref(null)
const instanceComponents = ref([])

const currentDeleteInstance = ref(null)
const confirmModal = ref(null)

async function deleteProfile() {
	if (currentDeleteInstance.value) {
		await remove(currentDeleteInstance.value).catch(handleError)
	}
}

async function duplicateProfile(p) {
	await duplicate(p).catch(handleError)
}

const handleRightClick = (event, profilePathId) => {
	const item = instanceComponents.value?.find?.((x) => x?.instance?.path === profilePathId)
	if (!item || !instanceOptions.value) return
	const baseOptions = [
		{ name: 'add_content' },
		{ type: 'divider' },
		{ name: 'edit' },
		{ name: 'duplicate' },
		{ name: 'open' },
		{ name: 'copy' },
		{ type: 'divider' },
		{
			name: 'delete',
			color: 'danger',
		},
	]

	instanceOptions.value.showMenu(
		event,
		item,
		item.playing
			? [
					{
						name: 'stop',
						color: 'danger',
					},
					...baseOptions,
				]
			: [
					{
						name: 'play',
						color: 'primary',
					},
					...baseOptions,
				],
	)
}

const handleOptionsClick = async (args) => {
	switch (args.option) {
		case 'play':
			args.item.play(null, 'InstanceGridContextMenu')
			break
		case 'stop':
			args.item.stop(null, 'InstanceGridContextMenu')
			break
		case 'add_content':
			await args.item.addContent()
			break
		case 'edit':
			await args.item.seeInstance()
			break
		case 'duplicate':
			if (args.item.instance.install_stage == 'installed')
				await duplicateProfile(args.item.instance.path)
			break
		case 'open':
			await args.item.openFolder()
			break
		case 'copy':
			await navigator.clipboard.writeText(args.item.instance.path)
			break
		case 'delete':
			currentDeleteInstance.value = args.item.instance.path
			confirmModal.value.show()
			break
	}
}

const state = useStorage(
	`${props.label}-grid-display-state`,
	{
		group: formatMessage(messages.optionGroup),
		sortBy: formatMessage(messages.optionName),
	},
	localStorage,
	{ mergeDefaults: true },
)

const search = ref('')

const filteredResults = computed(() => {
	const {
		group = formatMessage(messages.optionGroup),
		sortBy = formatMessage(messages.optionName),
	} = state.value

	const instances = props.instances.filter((instance) => {
		return instance.name.toLowerCase().includes(search.value.toLowerCase())
	})

	if (sortBy === formatMessage(messages.optionName)) {
		instances.sort((a, b) => {
			return a.name.localeCompare(b.name)
		})
	}

	if (sortBy === formatMessage(messages.optionGameVersion)) {
		instances.sort((a, b) => {
			return a.game_version.localeCompare(b.game_version, undefined, { numeric: true })
		})
	}

	if (sortBy === formatMessage(messages.optionLastPlayed)) {
		instances.sort((a, b) => {
			return dayjs(b.last_played ?? 0).diff(dayjs(a.last_played ?? 0))
		})
	}

	if (sortBy === formatMessage(messages.optionDateCreated)) {
		instances.sort((a, b) => {
			return dayjs(b.date_created).diff(dayjs(a.date_created))
		})
	}

	if (sortBy === formatMessage(messages.optionDateModified)) {
		instances.sort((a, b) => {
			return dayjs(b.date_modified).diff(dayjs(a.date_modified))
		})
	}

	const instanceMap = new Map()

	if (group === formatMessage(messages.optionLoader)) {
		instances.forEach((instance) => {
			const loader = formatCategoryHeader(instance.loader)
			if (!instanceMap.has(loader)) {
				instanceMap.set(loader, [])
			}

			instanceMap.get(loader).push(instance)
		})
	} else if (group === formatMessage(messages.optionGameVersion)) {
		instances.forEach((instance) => {
			if (!instanceMap.has(instance.game_version)) {
				instanceMap.set(instance.game_version, [])
			}

			instanceMap.get(instance.game_version).push(instance)
		})
	} else if (group === formatMessage(messages.optionGroup)) {
		instances.forEach((instance) => {
			if (instance.groups.length === 0) {
				instance.groups.push(formatMessage(messages.optionNone))
			}

			for (const category of instance.groups) {
				if (!instanceMap.has(category)) {
					instanceMap.set(category, [])
				}

				instanceMap.get(category).push(instance)
			}
		})
	} else {
		return instanceMap.set(formatMessage(messages.optionNone), instances)
	}

	// For 'name', we intuitively expect the sorting to apply to the name of the group first, not just the name of the instance
	// ie: Category A should come before B, even if the first instance in B comes before the first instance in A
	if (sortBy === formatMessage(messages.optionName)) {
		const sortedEntries = [...instanceMap.entries()].sort((a, b) => {
			// None should always be first
			if (
				a[0] === formatMessage(messages.optionNone) &&
				b[0] !== formatMessage(messages.optionNone)
			) {
				return -1
			}
			if (
				a[0] !== formatMessage(messages.optionNone) &&
				b[0] === formatMessage(messages.optionNone)
			) {
				return 1
			}
			return a[0].localeCompare(b[0])
		})
		instanceMap.clear()
		sortedEntries.forEach((entry) => {
			instanceMap.set(entry[0], entry[1])
		})
	}
	// default sorting would do 1.20.4 < 1.8.9 because 2 < 8
	// localeCompare with numeric=true puts 1.8.9 < 1.20.4 because 8 < 20
	if (group === formatMessage(messages.optionGameVersion)) {
		const sortedEntries = [...instanceMap.entries()].sort((a, b) => {
			return a[0].localeCompare(b[0], undefined, { numeric: true })
		})
		instanceMap.clear()
		sortedEntries.forEach((entry) => {
			instanceMap.set(entry[0], entry[1])
		})
	}

	return instanceMap
})
</script>

<template>
	<div class="library-grid-root">
		<div class="library-toolbar">
			<div class="toolbar-inner">
				<div class="toolbar-top">
					<div class="toolbar-title">
						<div class="toolbar-kicker">{{ label || formatMessage(messages.libraryFallback) }}</div>
						<div class="toolbar-count">{{ toolbarCount }}</div>
					</div>
					<div class="toolbar-search iconified-input flex-1 min-w-[14rem]">
						<SearchIcon />
						<input
							v-model="search"
							type="text"
							:placeholder="formatMessage(messages.searchPlaceholder)"
						/>
						<Button class="r-btn" @click="() => (search = '')">
							<XIcon />
						</Button>
					</div>
				</div>
				<div class="toolbar-controls">
					<DropdownSelect
						v-slot="{ selected }"
						v-model="state.sortBy"
						:name="formatMessage(messages.sortDropdownName)"
						class="toolbar-select"
						:options="sortOptions"
						:placeholder="formatMessage(messages.placeholderSelect)"
					>
						<span class="toolbar-select-content">
							<span class="toolbar-label">{{ formatMessage(messages.labelSort) }}</span>
							<span class="toolbar-value">{{ selected }}</span>
						</span>
					</DropdownSelect>
					<DropdownSelect
						v-slot="{ selected }"
						v-model="state.group"
						:name="formatMessage(messages.groupDropdownName)"
						class="toolbar-select"
						:options="groupOptions"
						:placeholder="formatMessage(messages.placeholderSelect)"
					>
						<span class="toolbar-select-content">
							<span class="toolbar-label">{{ formatMessage(messages.labelGroup) }}</span>
							<span class="toolbar-value">{{ selected }}</span>
						</span>
					</DropdownSelect>
				</div>
			</div>
		</div>

		<div
			v-for="instanceSection in Array.from(filteredResults, ([key, value]) => ({
				key,
				value,
			}))"
			:key="instanceSection.key"
			class="group-section"
			:class="{ 'is-none': instanceSection.key === formatMessage(messages.optionNone) }"
		>
			<div v-if="instanceSection.key !== formatMessage(messages.optionNone)" class="group-header">
				<div class="group-title">
					<div class="group-name">{{ instanceSection.key }}</div>
					<div class="group-meta">{{ instanceSection.value.length }}</div>
				</div>
				<div class="group-spacer"></div>
			</div>
			<TransitionGroup name="library-grid" tag="section" class="instances">
				<Instance
					v-for="instance in instanceSection.value"
					ref="instanceComponents"
					:key="instance.path + instance.install_stage"
					:instance="instance"
					@contextmenu.prevent.stop="(event) => handleRightClick(event, instance.path)"
				/>
			</TransitionGroup>
		</div>
	</div>

	<ConfirmModalWrapper
		ref="confirmModal"
		:title="formatMessage(messages.deleteInstanceTitle)"
		:description="formatMessage(messages.deleteInstanceDescription)"
		:has-to-type="false"
		:proceed-label="formatMessage(messages.delete)"
		@proceed="deleteProfile"
	/>
	<ContextMenu ref="instanceOptions" @option-clicked="handleOptionsClick">
		<template #play> <PlayIcon /> {{ formatMessage(messages.play) }} </template>
		<template #stop> <StopCircleIcon /> {{ formatMessage(messages.stop) }} </template>
		<template #add_content> <PlusIcon /> {{ formatMessage(messages.addContent) }} </template>
		<template #edit> <EyeIcon /> {{ formatMessage(messages.viewInstance) }} </template>
		<template #duplicate>
			<ClipboardCopyIcon /> {{ formatMessage(messages.duplicateInstance) }}</template
		>
		<template #delete> <TrashIcon /> {{ formatMessage(messages.delete) }} </template>
		<template #open> <FolderOpenIcon /> {{ formatMessage(messages.openFolder) }} </template>
		<template #copy> <ClipboardCopyIcon /> {{ formatMessage(messages.copyPath) }} </template>
	</ContextMenu>
</template>
<style lang="scss" scoped>
.library-grid-move,
.library-grid-enter-active,
.library-grid-leave-active {
	transition: all 0.32s cubic-bezier(0.2, 0.9, 0.2, 1);
}

.library-grid-enter-from,
.library-grid-leave-to {
	opacity: 0;
	transform: translateY(12px) scale(0.985);
}

.library-grid-leave-active {
	position: absolute;
}

.library-grid-root {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.library-toolbar {
	position: sticky;
	top: 0;
	z-index: 5;
	padding: 0.25rem;
	border-radius: 1.25rem;
	background: var(--color-glass-bg);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
}

.toolbar-inner {
	display: flex;
	gap: 0.75rem;
	align-items: center;
	flex-wrap: wrap;
	padding: 0.5rem;
}

.toolbar-top {
	display: flex;
	gap: 0.75rem;
	align-items: center;
	flex: 1;
	min-width: 18rem;
}

.toolbar-title {
	display: flex;
	flex-direction: column;
	gap: 0.1rem;
	min-width: 8.5rem;
}

.toolbar-kicker {
	font-size: 0.85rem;
	font-weight: 900;
	letter-spacing: 0.02em;
	color: var(--color-contrast);
}

.toolbar-count {
	font-size: 0.75rem;
	font-weight: 800;
	color: var(--color-secondary);
}

.toolbar-search {
	min-height: 2.5rem;
}

.toolbar-controls {
	display: flex;
	gap: 0.5rem;
	align-items: center;
	flex-wrap: wrap;
}

.toolbar-select {
	max-width: 16rem;
	border-radius: 999px;
	background: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
}

/* DropdownSelect uses v-popper; teleported nodes still carry the scope attribute so this applies */
:deep(.v-popper__inner) {
	background: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	color: var(--color-contrast);
}

:deep(.v-popper__arrow-inner) {
	border-color: var(--glass-border);
}

.toolbar-label {
	margin-right: 0.35rem;
	font-weight: 700;
	color: var(--color-secondary);
}

.toolbar-label::after {
	content: ' ';
}

.toolbar-value {
	font-weight: 800;
	color: var(--color-contrast);
}

.toolbar-select-content {
	display: inline-flex;
	align-items: baseline;
	gap: 0.35rem;
}

.group-section {
	border-radius: 1.25rem;
	background: var(--color-glass-bg);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	padding: 0.75rem;
	transition:
		transform 200ms cubic-bezier(0.2, 0.9, 0.2, 1),
		box-shadow 200ms cubic-bezier(0.2, 0.9, 0.2, 1),
		background-color 160ms ease;
}

.group-section:hover {
	transform: translateY(-1px);
}

.group-section:active {
	transform: translateY(0px) scale(0.995);
}

.group-section.is-none {
	background: transparent;
	border-color: transparent;
	box-shadow: none;
	backdrop-filter: none;
	-webkit-backdrop-filter: none;
	padding: 0;
}

.group-header {
	display: flex;
	align-items: center;
	gap: 0.75rem;
	padding: 0.25rem 0.25rem 0.75rem;
}

.group-title {
	display: inline-flex;
	align-items: baseline;
	gap: 0.5rem;
}

.group-name {
	font-size: 0.95rem;
	font-weight: 900;
	letter-spacing: 0.02em;
	color: var(--color-contrast);
}

.group-meta {
	font-size: 0.75rem;
	font-weight: 800;
	color: var(--color-secondary);
	background: var(--color-button-bg);
	border: 1px solid var(--glass-border);
	border-radius: 999px;
	padding: 0.1rem 0.5rem;
}

.group-spacer {
	flex: 1;
	height: 1px;
	background: var(--glass-border);
	opacity: 0.8;
}

.instances {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(17.5rem, 1fr));
	width: 100%;
	gap: 0.85rem;
	margin-right: auto;
	scroll-behavior: smooth;
	overflow: visible;
	position: relative;
	padding: 0.25rem;
}
</style>
