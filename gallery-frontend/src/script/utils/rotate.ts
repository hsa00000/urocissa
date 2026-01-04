import axios from 'axios'
import { useMessageStore } from '@/store/messageStore'
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
  const messageStore = useMessageStore('mainId')
  const editStore = useEditStore('mainId')

  if (editStore.hasRotate(hash)) return

  editStore.addRotate(hash)
  try {
    await tryWithMessageStore(isolationId, async () => {
      messageStore.info('Rotating image...')

      await axios.put('/put/rotate-image', { hash })

      editStore.incrementRotation(hash)

      messageStore.success('Image rotated successfully')
    })
  } finally {
    editStore.removeRotate(hash)
  }
}
