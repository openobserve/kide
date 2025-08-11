// Simple utility functions for terminal tabs

export function generateTabId(type: 'shell' | 'logs'): string {
  return `${type}-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`
}

export function getTabTooltip(tab: { type: string; podName: string }): string {
  return `${tab.type.charAt(0).toUpperCase() + tab.type.slice(1)}: ${tab.podName}`
}

export interface TabData {
  podName: string
  namespace: string
  containerName: string
  containers: any[]
  initContainers?: any[]
}