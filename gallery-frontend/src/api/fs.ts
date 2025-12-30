import axios from 'axios'

export const fetchFsCompletion = async (path: string): Promise<string[]> => {
  const response = await axios.get<string[]>('/get/path-completion', {
    params: { path }
  })
  return response.data
}
