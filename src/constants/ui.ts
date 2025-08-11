// UI Constants and shared styles

export const TAB_STYLES = {
  active: 'bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100',
  inactive: 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
} as const

export const CONNECTION_STATUS = {
  connected: 'bg-green-500 animate-pulse',
  connecting: 'bg-yellow-500 animate-pulse', 
  disconnected: 'bg-gray-400'
} as const

export const DROPDOWN_STYLES = {
  menu: 'position: fixed; z-index: 999999; background: white; border: 2px solid #374151; border-radius: 6px; box-shadow: 0 10px 25px rgba(0,0,0,0.3); min-width: 160px; max-height: 200px; overflow-y: auto;',
  item: 'padding: 8px 12px; cursor: pointer; font-size: 14px; transition: background-color 0.15s;',
  selectedItem: {
    backgroundColor: '#dbeafe',
    color: '#1d4ed8',
    fontWeight: 'bold'
  },
  defaultItem: {
    backgroundColor: 'transparent',
    color: '#374151',
    fontWeight: 'normal'
  }
} as const

export const KEYBOARD_SHORTCUTS = {
  CLOSE_TAB: ['Ctrl+W', 'Cmd+W'],
  CYCLE_TABS: ['Ctrl+Tab', 'Cmd+Tab'],
  TAB_NUMBERS: ['Ctrl+1-9', 'Cmd+1-9']
} as const