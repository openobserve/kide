import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import type { K8sListItem } from '@/types'

export function useResourceSelection() {
  const selectedItems: Ref<Set<string>> = ref(new Set())
  const selectedItem: Ref<K8sListItem | null> = ref(null)

  const selectedCount = computed(() => selectedItems.value.size)

  function toggleSelectAll(items: K8sListItem[]): void {
    if (selectedItems.value.size === items.length) {
      selectedItems.value.clear()
    } else {
      selectedItems.value = new Set(items.map(item => item.metadata?.uid || '').filter(uid => uid))
    }
  }

  function toggleItemSelection(item: K8sListItem): void {
    const uid = item.metadata?.uid || ''
    if (selectedItems.value.has(uid)) {
      selectedItems.value.delete(uid)
    } else {
      selectedItems.value.add(uid)
    }
  }

  function selectItem(item: K8sListItem): void {
    selectedItem.value = item
  }

  function clearSelection(): void {
    selectedItems.value.clear()
    selectedItem.value = null
  }

  function getSelectedResourceItems(allItems: K8sListItem[]): K8sListItem[] {
    return allItems.filter(item => selectedItems.value.has(item.metadata?.uid || ''))
  }

  function isItemSelected(item: K8sListItem): boolean {
    return selectedItems.value.has(item.metadata?.uid || '')
  }

  return {
    selectedItems,
    selectedItem,
    selectedCount,
    toggleSelectAll,
    toggleItemSelection,
    selectItem,
    clearSelection,
    getSelectedResourceItems,
    isItemSelected
  }
}