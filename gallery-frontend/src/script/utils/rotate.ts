import axios from 'axios'
import { useEditStore } from '@/store/editStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'
import { IsolationId } from '@/type/types'

/**
 * Common handler for rotating an image.
 * This logic is shared between the UI menu item and keyboard shortcuts.
 */
export const handleRotateImage = async (
  hash: string,
  isolationId: IsolationId
): Promise<void> => {
  const editStore = useEditStore('mainId')

  // 1. Optimistic Update (Immediate visual feedback)
  editStore.incrementRotation(hash)

  // 2. Queue Backend Request (Serialized execution)
  // We use the queue to ensure requests are processed in order (1 -> 2 -> 3)
  // even if the user clicks rapidly. This prevents race conditions on the backend.
  await editStore.queueRotate(hash, async () => {
    await tryWithMessageStore(isolationId, async () => {
      // messageStore.info('Rotating image...') // Optional: Commented out to reduce spam on rapid clicks

      await axios.put('/put/rotate-image', { hash })

      // messageStore.success('Image rotated successfully') // Optional
    })
  })
}
