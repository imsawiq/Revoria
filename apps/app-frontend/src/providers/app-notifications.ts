import {
	AbstractWebNotificationManager,
	type NotificationPanelLocation,
	type WebNotification,
} from '@modrinth/ui'
import { type Ref, ref } from 'vue'

import allDeMessages from '@/locales/combined/de-DE.json'
import allEnMessages from '@/locales/combined/en-US.json'
import allRoMessages from '@/locales/combined/ro-RO.json'
import allRuMessages from '@/locales/combined/ru-RU.json'
import allUkMessages from '@/locales/combined/uk-UA.json'

const knownErrorMessages = {
	invalidProjectType: 'Unable to infer project type for input file',
}

const localeMessages = {
	en: allEnMessages,
	ru: allRuMessages,
	uk: allUkMessages,
	de: allDeMessages,
	ro: allRoMessages,
} as const

const readMessage = (
	messages: Record<string, { message?: string; defaultMessage?: string }>,
	id: string,
	fallback?: string,
) => messages?.[id]?.message ?? messages?.[id]?.defaultMessage ?? fallback ?? id

const getCurrentMessages = () => {
	const language = (localStorage.getItem('launcher-language') ?? 'en') as keyof typeof localeMessages
	return localeMessages[language] ?? localeMessages.en
}

export class AppNotificationManager extends AbstractWebNotificationManager {
	private readonly state: Ref<WebNotification[]>
	private readonly locationState: Ref<NotificationPanelLocation>

	public constructor() {
		super()
		this.state = ref<WebNotification[]>([])
		this.locationState = ref<NotificationPanelLocation>('right')
	}

	public getNotificationLocation(): NotificationPanelLocation {
		return this.locationState.value
	}

	public setNotificationLocation(location: NotificationPanelLocation): void {
		this.locationState.value = location
	}

	public getNotifications(): WebNotification[] {
		return this.state.value
	}

	public handleError = (error: Error): void => {
		const messages = getCurrentMessages()
		const errorText =
			error.message?.includes(knownErrorMessages.invalidProjectType)
				? readMessage(
						messages,
						'notification.error.invalid-project-type',
						'Invalid input: Unable to infer project type for input file',
					)
				: error.message ?? String(error)

		this.addNotification({
			title: readMessage(messages, 'notification.error.title', 'An error occurred'),
			text: errorText,
			type: 'error',
		})
	}

	protected addNotificationToStorage(notification: WebNotification): void {
		this.state.value.push(notification)
	}

	protected removeNotificationFromStorage(id: string | number): void {
		const index = this.state.value.findIndex((n) => n.id === id)
		if (index > -1) {
			this.state.value.splice(index, 1)
		}
	}

	protected removeNotificationFromStorageByIndex(index: number): void {
		this.state.value.splice(index, 1)
	}

	protected clearAllNotificationsFromStorage(): void {
		this.state.value.splice(0)
	}
}
