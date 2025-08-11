import { vi } from 'vitest'

// Mock Tauri core API
export const invoke = vi.fn().mockImplementation((command: string, args?: any) => {
  switch (command) {
    case 'get_pod_logs':
      return Promise.resolve(`2025-08-05T04:51:15.136893673Z Starting application
2025-08-05T04:51:16.245781234Z Loading configuration
2025-08-05T04:51:17.358924567Z Database connected
2025-08-05T04:51:18.471035890Z Server listening on port 8080`)

    case 'start_pod_logs_stream':
      return Promise.resolve('stream-id-123')

    case 'stop_pod_logs_stream':
      return Promise.resolve()

    case 'connect_k8s':
      return Promise.resolve()

    case 'get_resources':
      return Promise.resolve([
        {
          name: 'Workloads',
          resources: [
            {
              name: 'Pods',
              apiVersion: 'v1',
              kind: 'Pod',
              namespaced: true,
              description: 'Smallest deployable units'
            }
          ]
        }
      ])

    case 'get_namespaces':
      return Promise.resolve(['default', 'kube-system', 'test-namespace'])

    case 'start_resource_watch':
      return Promise.resolve()

    case 'stop_resource_watch':
      return Promise.resolve()

    case 'get_resource_events':
      return Promise.resolve([
        {
          uid: 'event-1',
          type: 'Normal',
          reason: 'Created',
          message: 'Pod created successfully',
          source: { component: 'kubelet' },
          firstTimestamp: '2025-08-05T04:51:15.136893673Z',
          lastTimestamp: '2025-08-05T04:51:15.136893673Z',
          count: 1
        },
        {
          uid: 'event-2',
          type: 'Normal',
          reason: 'Started',
          message: 'Container started',
          source: { component: 'kubelet' },
          firstTimestamp: '2025-08-05T04:51:16.245781234Z',
          lastTimestamp: '2025-08-05T04:51:16.245781234Z',
          count: 1
        },
        {
          uid: 'event-3',
          type: 'Warning',
          reason: 'BackOff',
          message: 'Back-off restarting failed container',
          source: { component: 'kubelet' },
          firstTimestamp: '2025-08-05T04:51:20.345678901Z',
          lastTimestamp: '2025-08-05T04:51:25.456789012Z',
          count: 3
        }
      ])

    case 'scale_resource':
      // Mock successful scaling
      return Promise.resolve()

    case 'delete_resource':
      return Promise.resolve()

    case 'update_resource':
      return Promise.resolve()

    default:
      console.warn(`Unhandled Tauri command: ${command}`)
      return Promise.resolve()
  }
})