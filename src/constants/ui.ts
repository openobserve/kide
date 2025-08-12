// UI Constants and shared styles

export const TAB_STYLES = {
  active: 'bg-surface-primary text-text-primary',
  inactive: 'bg-surface-secondary text-text-secondary interactive-hover'
} as const

export const CONNECTION_STATUS = {
  connected: 'bg-status-success animate-pulse',
  connecting: 'bg-status-warning animate-pulse', 
  disconnected: 'bg-text-muted'
} as const

export const DROPDOWN_STYLES = {
  menu: 'dropdown-menu',
  item: 'dropdown-item',
  selectedItem: 'dropdown-item dropdown-item-selected',
  defaultItem: 'dropdown-item'
} as const

export const KEYBOARD_SHORTCUTS = {
  CLOSE_TAB: ['Ctrl+W', 'Cmd+W'],
  CYCLE_TABS: ['Ctrl+Tab', 'Cmd+Tab'],
  TAB_NUMBERS: ['Ctrl+1-9', 'Cmd+1-9']
} as const