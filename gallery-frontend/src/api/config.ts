import axios from 'axios'

export interface AppConfig {
  // Rocket settings
  address: string
  port: number
  limits: Record<string, string>

  // App settings
  readOnlyMode: boolean
  disableImg: boolean
  // password is handled separately now
  authKey?: string | null
  hasPassword: boolean
  hasDiscordHook: boolean
  discordHookUrl?: string | null
  syncPaths: string[]
}

export const getConfig = async (): Promise<AppConfig> => {
  const response = await axios.get<AppConfig>('/get/config')
  return response.data
}

export const updateConfig = async (config: AppConfig): Promise<void> => {
  await axios.put('/put/config', config)
}

export const updatePassword = async (oldPassword: string, newPassword?: string): Promise<void> => {
  await axios.put('/put/config/password', {
    oldPassword: oldPassword || null,
    password: newPassword
  })
}

export const exportConfig = async (): Promise<AppConfig> => {
  const response = await axios.get<AppConfig>('/get/config/export')
  return response.data
}

export const importConfig = async (config: AppConfig): Promise<void> => {
  // Refactor: path to /post/config/import
  await axios.post('/post/config/import', config)
}
