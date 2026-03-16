import {
	defineMessage as vintlDefineMessage,
	defineMessages as vintlDefineMessages,
	useVIntl,
} from '@vintl/vintl'

export interface MessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

export type MessageDescriptorMap<K extends string> = Record<K, MessageDescriptor>
export type CrowdinMessages = Record<string, { message?: string; defaultMessage?: string } | string>

export function defineMessage<T extends MessageDescriptor>(descriptor: T): T {
	return vintlDefineMessage(descriptor)
}

export function defineMessages<K extends string, T extends MessageDescriptorMap<K>>(
	descriptors: T,
): T {
	return vintlDefineMessages(descriptors)
}

export interface LocaleDefinition {
	code: string
	name: string
	translatedName: MessageDescriptor
	numeric?: Intl.RelativeTimeFormatNumeric
}

export const LOCALES: LocaleDefinition[] = [
	{
		code: 'en-US',
		name: 'English (United States)',
		translatedName: defineMessage({
			id: 'locale.en-US',
			defaultMessage: 'English (United States)',
		}),
	},
	{
		code: 'ru-RU',
		name: 'Русский',
		translatedName: defineMessage({
			id: 'locale.ru-RU',
			defaultMessage: 'Russian',
		}),
	},
]

export { useVIntl }
