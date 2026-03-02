import { getScrollUpperBound } from '@utils/getter'
import { IsolationId } from '@type/types'
import { usePrefetchStore } from '@/store/prefetchStore'
import { useScrollTopStore } from '@/store/scrollTopStore'
import { throttle } from 'lodash'
import { ComputedRef, Ref } from 'vue'
import { useConfigStore } from '@/store/configStore'
import { compensationThreshold, nativeThreshold } from '@/type/constants'

/**
 * Throttled scroll handler for an image container with three-mode scrolling.
 *
 * nativeTop mode (scrollTop < compensationThreshold): DOM scrollTop = virtualScrollTop.
 * Browser's native top boundary prevents overshooting 0.
 *
 * compensation mode: DOM scrollTop pinned at bufferHeight/3,
 * deltas absorbed into virtualScrollTop.
 *
 * nativeBottom mode (near bottom): DOM scrollTop = bottomOffset + virtualScrollTop.
 * Browser's native bottom boundary prevents overshooting the end.
 *
 * @param imageContainerRef - Reference to the scrolling container element.
 * @param lastScrollTop - Reference to the last recorded scroll position.
 * @param stopScroll - Flag to temporarily stop scrolling for mobile adjustments.
 * @param windowHeight - Reference to the window height for scroll limit calculations.
 * @param bufferHeight - Computed buffer height for compensation mode.
 * @param isolationId - Isolation ID for store access.
 *
 * @returns Throttled scroll event handler.
 */
export function handleScroll(
  imageContainerRef: Ref<HTMLElement | null>,
  lastScrollTop: Ref<number>,
  stopScroll: Ref<boolean>,
  windowHeight: Ref<number>,
  bufferHeight: ComputedRef<number>,
  isolationId: IsolationId
) {
  const throttledHandleScroll = throttle(
    () => {
      if (imageContainerRef.value !== null) {
        const configStore = useConfigStore('mainId')
        const mobile = configStore.isMobile
        const scrollTopStore = useScrollTopStore(isolationId)
        const prefetchStore = usePrefetchStore(isolationId)
        const upperBound = getScrollUpperBound(prefetchStore.totalHeight, windowHeight.value)

        if (prefetchStore.totalHeight - windowHeight.value < 0) {
          const difference = imageContainerRef.value.scrollTop - lastScrollTop.value
          if (mobile) {
            stopScroll.value = true
            scrollTopStore.scrollTop = 0
            setTimeout(() => {
              stopScroll.value = false
            }, 100)
          } else {
            scrollTopStore.scrollTop = 0
          }
          imageContainerRef.value.scrollTop -= difference
          lastScrollTop.value = imageContainerRef.value.scrollTop
          return
        }

        if (scrollTopStore.scrollMode === 'nativeTop') {
          // === Native Top mode ===
          const domScrollTop = imageContainerRef.value.scrollTop
          scrollTopStore.scrollTop = Math.max(0, Math.min(domScrollTop, upperBound))

          // Check: transition to compensation or nativeBottom
          if (scrollTopStore.scrollTop >= compensationThreshold) {
            if (upperBound - scrollTopStore.scrollTop >= compensationThreshold) {
              scrollTopStore.scrollMode = 'compensation'
              imageContainerRef.value.scrollTop = bufferHeight.value / 3
              lastScrollTop.value = bufferHeight.value / 3
            } else {
              scrollTopStore.scrollMode = 'nativeBottom'
              const bottomOffset = Math.max(bufferHeight.value, prefetchStore.totalHeight) - prefetchStore.totalHeight
              imageContainerRef.value.scrollTop = bottomOffset + scrollTopStore.scrollTop
              lastScrollTop.value = bottomOffset + scrollTopStore.scrollTop
            }
          }
        } else if (scrollTopStore.scrollMode === 'nativeBottom') {
          // === Native Bottom mode ===
          const bottomOffset = Math.max(bufferHeight.value, prefetchStore.totalHeight) - prefetchStore.totalHeight
          const domScrollTop = imageContainerRef.value.scrollTop
          scrollTopStore.scrollTop = Math.max(0, Math.min(domScrollTop - bottomOffset, upperBound))

          // Check: transition away from nativeBottom
          if (upperBound - scrollTopStore.scrollTop >= compensationThreshold && scrollTopStore.scrollTop >= compensationThreshold) {
            scrollTopStore.scrollMode = 'compensation'
            imageContainerRef.value.scrollTop = bufferHeight.value / 3
            lastScrollTop.value = bufferHeight.value / 3
          } else if (scrollTopStore.scrollTop < nativeThreshold) {
            scrollTopStore.scrollMode = 'nativeTop'
            imageContainerRef.value.scrollTop = scrollTopStore.scrollTop
            lastScrollTop.value = scrollTopStore.scrollTop
          }
        } else {
          // === Compensation mode ===
          const difference = imageContainerRef.value.scrollTop - lastScrollTop.value
          const result = scrollTopStore.scrollTop + difference

          if (result < 0) {
            if (mobile) {
              stopScroll.value = true
              scrollTopStore.scrollTop = 0
              setTimeout(() => {
                stopScroll.value = false
              }, 100)
            } else {
              scrollTopStore.scrollTop = 0
            }
          } else if (result >= upperBound) {
            if (mobile) {
              stopScroll.value = true
              scrollTopStore.scrollTop = upperBound
              setTimeout(() => {
                stopScroll.value = false
              }, 100)
            } else {
              scrollTopStore.scrollTop = upperBound
            }
          } else {
            scrollTopStore.scrollTop += difference
          }

          // Compensate for the change in scrollTop caused by the user's scroll action.
          imageContainerRef.value.scrollTop -= difference
          lastScrollTop.value = imageContainerRef.value.scrollTop

          // Check: transition to nativeTop or nativeBottom
          if (scrollTopStore.scrollTop < nativeThreshold) {
            scrollTopStore.scrollMode = 'nativeTop'
            imageContainerRef.value.scrollTop = scrollTopStore.scrollTop
            lastScrollTop.value = scrollTopStore.scrollTop
          } else if (upperBound - scrollTopStore.scrollTop < nativeThreshold) {
            scrollTopStore.scrollMode = 'nativeBottom'
            const bottomOffset = Math.max(bufferHeight.value, prefetchStore.totalHeight) - prefetchStore.totalHeight
            imageContainerRef.value.scrollTop = bottomOffset + scrollTopStore.scrollTop
            lastScrollTop.value = bottomOffset + scrollTopStore.scrollTop
          }
        }
      }
    },
    100,
    {
      leading: true
    }
  )
  return throttledHandleScroll
}
