import axios from 'axios'
import { useEditStore } from '@/store/editStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'
import { IsolationId } from '@/type/types'

export async function handleRotateImage(hash: string, isolationId: IsolationId): Promise<void> {
  const editStore = useEditStore('mainId')

  editStore.incrementRotation(hash)

  await editStore.queueRotate(hash, async () => {
    await tryWithMessageStore(isolationId, async () => {
      await axios.put('/put/rotate-image', { hash })
    })
  })
}
