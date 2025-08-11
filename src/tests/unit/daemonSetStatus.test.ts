import { describe, it, expect } from 'vitest'
import { useResourceStatus } from '../../composables/useResourceStatus'

describe('DaemonSet Status Logic', () => {
  const { getStatusText, getStatusClass } = useResourceStatus()

  it('should show Ready when all pods are ready', () => {
    const daemonSet = {
      kind: 'DaemonSet',
      daemonSetStatus: {
        desiredNumberScheduled: 3,
        currentNumberScheduled: 3,
        numberReady: 3,
        numberMisscheduled: 0
      }
    }

    expect(getStatusText(daemonSet)).toBe('Ready')
    expect(getStatusClass(daemonSet)).toContain('bg-green-100')
  })

  it('should show NotReady when some pods are not ready', () => {
    const daemonSet = {
      kind: 'DaemonSet',
      daemonSetStatus: {
        desiredNumberScheduled: 3,
        currentNumberScheduled: 3,
        numberReady: 2,
        numberMisscheduled: 0
      }
    }

    expect(getStatusText(daemonSet)).toBe('NotReady')
    expect(getStatusClass(daemonSet)).toContain('bg-red-100')
  })

  it('should show NotReady when pods are misscheduled', () => {
    const daemonSet = {
      kind: 'DaemonSet',
      daemonSetStatus: {
        desiredNumberScheduled: 3,
        currentNumberScheduled: 3,
        numberReady: 3,
        numberMisscheduled: 1
      }
    }

    expect(getStatusText(daemonSet)).toBe('NotReady')
    expect(getStatusClass(daemonSet)).toContain('bg-red-100')
  })

  it('should show Updating during rolling update', () => {
    const daemonSet = {
      kind: 'DaemonSet',
      daemonSetStatus: {
        desiredNumberScheduled: 3,
        currentNumberScheduled: 3,
        numberReady: 3,
        numberMisscheduled: 0,
        updatedNumberScheduled: 2
      }
    }

    expect(getStatusText(daemonSet)).toBe('Updating')
    expect(getStatusClass(daemonSet)).toContain('bg-yellow-100')
  })

  it('should show Failed when conditions indicate failure', () => {
    const daemonSet = {
      kind: 'DaemonSet',
      daemonSetStatus: {
        desiredNumberScheduled: 3,
        currentNumberScheduled: 2,
        numberReady: 1,
        numberMisscheduled: 0,
        conditions: [
          {
            type: 'Ready',
            status: 'False',
            reason: 'PodNotReady',
            message: 'Some pods are not ready'
          }
        ]
      }
    }

    expect(getStatusText(daemonSet)).toBe('Failed')
    expect(getStatusClass(daemonSet)).toContain('bg-red-100')
  })

  it('should show Unknown when no status is available', () => {
    const daemonSet = {
      kind: 'DaemonSet'
    }

    expect(getStatusText(daemonSet)).toBe('Unknown')
    expect(getStatusClass(daemonSet)).toContain('bg-gray-100')
  })

  it('should show Unknown when desiredNumberScheduled is 0', () => {
    const daemonSet = {
      kind: 'DaemonSet',
      daemonSetStatus: {
        desiredNumberScheduled: 0,
        currentNumberScheduled: 0,
        numberReady: 0,
        numberMisscheduled: 0
      }
    }

    expect(getStatusText(daemonSet)).toBe('Unknown')
    expect(getStatusClass(daemonSet)).toContain('bg-gray-100')
  })
})