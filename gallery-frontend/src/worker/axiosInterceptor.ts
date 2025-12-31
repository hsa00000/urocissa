import { AxiosInstance, AxiosError } from 'axios'
import { errorDisplay } from '@/script/utils/errorDisplay'

export function setupAxiosInterceptor(
  axiosInstance: AxiosInstance,
  notify: (payload: { text: string; color: 'error' }) => void
) {
  axiosInstance.interceptors.response.use(
    (response) => response,
    (error: AxiosError) => {
      // Handle server errors (500) or other unexpected errors
      // Use the unified errorDisplay function to parse the error
      if (error.response?.status === 500) {
        const errorMessage = errorDisplay(error)
        notify({ text: errorMessage, color: 'error' })
      }
      return Promise.reject(error)
    }
  )
}
