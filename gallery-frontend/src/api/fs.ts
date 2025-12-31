import axios from 'axios'

export interface FsCompletion {
  roots: string[]
  children: string[]
  is_default: boolean
}

export const fetchFsCompletion = async (path: string): Promise<FsCompletion> => {
  const response = await axios.get<FsCompletion>('/get/path-completion', {
    params: { path }
  })
  return response.data
}
