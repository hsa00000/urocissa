import { AxiosInstance, AxiosError } from 'axios'
import { errorDisplay } from '@/script/utils/errorDisplay'

export function setupWorkerAxiosInterceptor(
  axiosInstance: AxiosInstance,
  notify: (payload: { text: string; color: 'error' }) => void
) {
  axiosInstance.interceptors.response.use(
    (response) => response,
    (error: AxiosError) => {
      // Explicitly handling 500 for workers, passing others through
      if (error.response?.status === 500) {
        const errorMessage = errorDisplay(error)
        notify({ text: errorMessage, color: 'error' })
      }
      return Promise.reject(error)
    }
  )
}
