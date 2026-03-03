import { editTags } from '@/api/editTags'
import { IsolationId } from '@type/types'

export async function quickAddTags(tag: string, indexList: number[], isolationId: IsolationId) {
  await editTags(indexList, [tag], [], isolationId)
}

export async function quickRemoveTags(tag: string, indexList: number[], isolationId: IsolationId) {
  await editTags(indexList, [], [tag], isolationId)
}

export async function quickEditTags(
  tag: string,
  indexListAdd: number[],
  indexListRemove: number[],
  isolationId: IsolationId
) {
  await quickAddTags(tag, indexListAdd, isolationId)
  await quickRemoveTags(tag, indexListRemove, isolationId)
}
