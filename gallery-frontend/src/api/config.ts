import axios from 'axios'

export interface AppConfig {
  // Rocket settings
  address: string
  port: number
  limits: Record<string, string>

  // App settings
  readOnlyMode: boolean
  disableImg: boolean
  password: string
  authKey?: string | null
  discordHookUrl?: string | null
  syncPaths: string[]
}

export const getConfig = async (): Promise<AppConfig> => {
  const response = await axios.get<AppConfig>('/get/config')
  return response.data
}

export const updateConfig = async (config: AppConfig & { oldPassword?: string }): Promise<void> => {
  await axios.put('/put/config', config)
}

export const exportConfig = async (): Promise<AppConfig> => {
  const response = await axios.get<AppConfig>('/get/config/export')
  return response.data
}

export const importConfig = async (config: AppConfig): Promise<void> => {
  // Refactor: path to /post/config/import
  await axios.post('/post/config/import', config)
}
