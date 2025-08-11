import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { ref, nextTick, watch } from 'vue'

describe('Watch Event Handling', () => {
  let resourceItems
  let mockEvent
  let consoleSpy

  beforeEach(() => {
    resourceItems = ref([])
    consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
    
    // Mock event data
    mockEvent = {
      Added: {
        metadata: {
          name: 'test-node-1',
          uid: 'uid-123',
          namespace: null,
          creationTimestamp: '2025-08-04T10:00:00Z',
          labels: {
            'kubernetes.io/hostname': 'test-node-1'
          }
        },
        kind: 'Node',
        status: {
          conditions: [
            { type: 'Ready', status: 'True' }
          ]
        }
      }
    }
  })

  afterEach(() => {
    consoleSpy.mockRestore()
  })

  // Function under test - extracted from App.vue
  function handleWatchEvent(event) {
    console.log('🎯 Received watch event:', event)
    console.log('🎯 Current resourceItems.value.length BEFORE:', resourceItems.value.length)
    
    if (event.Added) {
      console.log('➕ Adding resource:', event.Added.metadata?.name || 'Unknown')
      console.log('➕ Event.Added data:', event.Added)
      
      const existingIndex = resourceItems.value.findIndex(item => 
        item.metadata?.uid === event.Added.metadata?.uid
      )
      console.log('🔍 Existing index:', existingIndex)
      
      if (existingIndex >= 0 && event.Added.metadata?.uid) {
        // Update existing item by creating new array to trigger reactivity
        resourceItems.value = [
          ...resourceItems.value.slice(0, existingIndex),
          event.Added,
          ...resourceItems.value.slice(existingIndex + 1)
        ]
        console.log('🔄 Updated existing resource at index', existingIndex)
      } else {
        // Add new item by creating new array to trigger reactivity
        console.log('✨ About to add new resource...')
        resourceItems.value = [...resourceItems.value, event.Added]
        console.log('✨ Added new resource, new length:', resourceItems.value.length)
      }
      
      console.log('📊 Total resource items AFTER:', resourceItems.value.length)
      console.log('📊 Resource names:', resourceItems.value.map(item => item.metadata?.name || 'Unknown'))
      
    } else if (event.Modified) {
      const index = resourceItems.value.findIndex(item => 
        item.metadata?.uid === event.Modified.metadata?.uid
      )
      if (index >= 0 && event.Modified.metadata?.uid) {
        // Update by creating new array to trigger reactivity
        resourceItems.value = [
          ...resourceItems.value.slice(0, index),
          event.Modified,
          ...resourceItems.value.slice(index + 1)
        ]
        console.log('📝 Modified resource:', event.Modified.metadata?.name || 'Unknown')
      }
    } else if (event.Deleted) {
      resourceItems.value = resourceItems.value.filter(item => 
        item.metadata?.uid !== event.Deleted.metadata?.uid
      )
      console.log('🗑️ Deleted resource, remaining:', resourceItems.value.length)
    }
  }

  describe('Adding new resources', () => {
    it('should add a new resource to empty array', async () => {
      expect(resourceItems.value).toHaveLength(0)

      handleWatchEvent(mockEvent)
      await nextTick()

      expect(resourceItems.value).toHaveLength(1)
      expect(resourceItems.value[0]).toEqual(mockEvent.Added)
      expect(resourceItems.value[0].metadata.name).toBe('test-node-1')
      expect(consoleSpy).toHaveBeenCalledWith('✨ About to add new resource...')
      expect(consoleSpy).toHaveBeenCalledWith('✨ Added new resource, new length:', 1)
    })

    it('should add multiple different resources', async () => {
      const secondEvent = {
        Added: {
          metadata: {
            name: 'test-node-2',
            uid: 'uid-456',
            namespace: null,
            creationTimestamp: '2025-08-04T10:01:00Z'
          },
          kind: 'Node'
        }
      }

      handleWatchEvent(mockEvent)
      await nextTick()
      handleWatchEvent(secondEvent)
      await nextTick()

      expect(resourceItems.value).toHaveLength(2)
      expect(resourceItems.value[0].metadata.name).toBe('test-node-1')
      expect(resourceItems.value[1].metadata.name).toBe('test-node-2')
    })

    it('should update existing resource instead of adding duplicate', async () => {
      // Add initial resource
      handleWatchEvent(mockEvent)
      await nextTick()
      expect(resourceItems.value).toHaveLength(1)

      // Update same resource (same UID)
      const updatedEvent = {
        Added: {
          ...mockEvent.Added,
          metadata: {
            ...mockEvent.Added.metadata,
            name: 'test-node-1-updated'
          }
        }
      }

      handleWatchEvent(updatedEvent)
      await nextTick()

      expect(resourceItems.value).toHaveLength(1)
      expect(resourceItems.value[0].metadata.name).toBe('test-node-1-updated')
      expect(consoleSpy).toHaveBeenCalledWith('🔄 Updated existing resource at index', 0)
    })
  })

  describe('Modifying resources', () => {
    beforeEach(async () => {
      // Add initial resource
      handleWatchEvent(mockEvent)
      await nextTick()
    })

    it('should modify existing resource', async () => {
      const modifyEvent = {
        Modified: {
          ...mockEvent.Added,
          metadata: {
            ...mockEvent.Added.metadata,
            name: 'test-node-1-modified'
          },
          status: {
            conditions: [
              { type: 'Ready', status: 'False' }
            ]
          }
        }
      }

      handleWatchEvent(modifyEvent)
      await nextTick()

      expect(resourceItems.value).toHaveLength(1)
      expect(resourceItems.value[0].metadata.name).toBe('test-node-1-modified')
      expect(resourceItems.value[0].status.conditions[0].status).toBe('False')
      expect(consoleSpy).toHaveBeenCalledWith('📝 Modified resource:', 'test-node-1-modified')
    })

    it('should ignore modification of non-existent resource', async () => {
      const modifyEvent = {
        Modified: {
          metadata: {
            name: 'non-existent-node',
            uid: 'uid-999'
          }
        }
      }

      const initialLength = resourceItems.value.length
      handleWatchEvent(modifyEvent)
      await nextTick()

      expect(resourceItems.value).toHaveLength(initialLength)
      expect(resourceItems.value[0].metadata.name).toBe('test-node-1') // unchanged
    })
  })

  describe('Deleting resources', () => {
    beforeEach(async () => {
      // Add multiple resources
      handleWatchEvent(mockEvent)
      
      const secondEvent = {
        Added: {
          metadata: {
            name: 'test-node-2',
            uid: 'uid-456'
          }
        }
      }
      handleWatchEvent(secondEvent)
      await nextTick()
    })

    it('should delete existing resource', async () => {
      expect(resourceItems.value).toHaveLength(2)

      const deleteEvent = {
        Deleted: {
          metadata: {
            uid: 'uid-123'
          }
        }
      }

      handleWatchEvent(deleteEvent)
      await nextTick()

      expect(resourceItems.value).toHaveLength(1)
      expect(resourceItems.value[0].metadata.uid).toBe('uid-456')
      expect(consoleSpy).toHaveBeenCalledWith('🗑️ Deleted resource, remaining:', 1)
    })

    it('should ignore deletion of non-existent resource', async () => {
      const initialLength = resourceItems.value.length
      
      const deleteEvent = {
        Deleted: {
          metadata: {
            uid: 'uid-999'
          }
        }
      }

      handleWatchEvent(deleteEvent)
      await nextTick()

      expect(resourceItems.value).toHaveLength(initialLength)
    })
  })

  describe('Vue reactivity', () => {
    it('should trigger Vue reactivity when adding resources', async () => {
      const watcherSpy = vi.fn()
      
      // Set up a watcher to track reactivity
      const stopWatcher = watch(resourceItems, watcherSpy, { deep: true })

      handleWatchEvent(mockEvent)
      await nextTick()

      expect(watcherSpy).toHaveBeenCalled()
      stopWatcher()
    })

    it('should create new array reference for reactivity', async () => {
      const originalArray = resourceItems.value
      
      handleWatchEvent(mockEvent)
      await nextTick()

      // Verify that a new array reference was created
      expect(resourceItems.value).not.toBe(originalArray)
      expect(resourceItems.value).toHaveLength(1)
    })

    it('should maintain array reference equality for modifications', async () => {
      handleWatchEvent(mockEvent)
      await nextTick()
      
      const arrayAfterAdd = resourceItems.value
      
      const modifyEvent = {
        Modified: {
          ...mockEvent.Added,
          metadata: {
            ...mockEvent.Added.metadata,
            name: 'modified-name'
          }
        }
      }

      handleWatchEvent(modifyEvent)
      await nextTick()

      // Should create new array reference for modification too
      expect(resourceItems.value).not.toBe(arrayAfterAdd)
    })
  })

  describe('Error handling', () => {
    it('should handle events with missing metadata', () => {
      const malformedEvent = {
        Added: {
          kind: 'Node'
          // missing metadata
        }
      }

      expect(() => handleWatchEvent(malformedEvent)).not.toThrow()
    })

    it('should handle empty events', () => {
      const emptyEvent = {}
      
      expect(() => handleWatchEvent(emptyEvent)).not.toThrow()
      expect(resourceItems.value).toHaveLength(0)
    })
  })

  describe('Logging', () => {
    it('should log all expected debug messages for adding resources', async () => {
      handleWatchEvent(mockEvent)

      expect(consoleSpy).toHaveBeenCalledWith('🎯 Received watch event:', mockEvent)
      expect(consoleSpy).toHaveBeenCalledWith('🎯 Current resourceItems.value.length BEFORE:', 0)
      expect(consoleSpy).toHaveBeenCalledWith('➕ Adding resource:', 'test-node-1')
      expect(consoleSpy).toHaveBeenCalledWith('🔍 Existing index:', -1)
      expect(consoleSpy).toHaveBeenCalledWith('✨ About to add new resource...')
      expect(consoleSpy).toHaveBeenCalledWith('✨ Added new resource, new length:', 1)
      expect(consoleSpy).toHaveBeenCalledWith('📊 Total resource items AFTER:', 1)
      expect(consoleSpy).toHaveBeenCalledWith('📊 Resource names:', ['test-node-1'])
    })
  })
})