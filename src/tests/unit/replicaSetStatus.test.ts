import { describe, it, expect } from 'vitest'
import { useResourceStatus } from '../../composables/useResourceStatus'

describe('ReplicaSet Status Logic', () => {
  const { getStatusText, getStatusClass } = useResourceStatus()

  it('should show Ready when all replicas are ready and available', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 3 },
      replicaSetStatus: {
        replicas: 3,
        readyReplicas: 3,
        availableReplicas: 3
      }
    }

    expect(getStatusText(replicaSet)).toBe('Ready')
    expect(getStatusClass(replicaSet)).toContain('status-badge-success')
  })

  it('should show NotReady when some replicas are not ready', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 3 },
      replicaSetStatus: {
        replicas: 3,
        readyReplicas: 2,
        availableReplicas: 3
      }
    }

    expect(getStatusText(replicaSet)).toBe('NotReady')
    expect(getStatusClass(replicaSet)).toContain('status-badge-error')
  })

  it('should show NotReady when some replicas are not available', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 3 },
      replicaSetStatus: {
        replicas: 3,
        readyReplicas: 3,
        availableReplicas: 2
      }
    }

    expect(getStatusText(replicaSet)).toBe('NotReady')
    expect(getStatusClass(replicaSet)).toContain('status-badge-error')
  })

  it('should show Scaling when replica count does not match desired', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 5 },
      replicaSetStatus: {
        replicas: 3,
        readyReplicas: 3,
        availableReplicas: 3
      }
    }

    expect(getStatusText(replicaSet)).toBe('Scaling')
    expect(getStatusClass(replicaSet)).toContain('status-badge-yellow')
  })

  it('should show Scaling when scaling down', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 1 },
      replicaSetStatus: {
        replicas: 3,
        readyReplicas: 3,
        availableReplicas: 3
      }
    }

    expect(getStatusText(replicaSet)).toBe('Scaling')
    expect(getStatusClass(replicaSet)).toContain('status-badge-yellow')
  })

  it('should show Failed when ReplicaFailure condition is present', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 3 },
      replicaSetStatus: {
        replicas: 1,
        readyReplicas: 1,
        availableReplicas: 1,
        conditions: [
          {
            type: 'ReplicaFailure',
            status: 'True',
            reason: 'FailedCreate',
            message: 'pods "test-pod-" is forbidden'
          }
        ]
      }
    }

    expect(getStatusText(replicaSet)).toBe('Failed')
    expect(getStatusClass(replicaSet)).toContain('status-badge-error')
  })

  it('should show Ready when desired replicas is 0 and actual is 0', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 0 },
      replicaSetStatus: {
        replicas: 0,
        readyReplicas: 0,
        availableReplicas: 0
      }
    }

    expect(getStatusText(replicaSet)).toBe('Ready')
    expect(getStatusClass(replicaSet)).toContain('status-badge-success')
  })

  it('should show Unknown when no status is available', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 3 }
    }

    expect(getStatusText(replicaSet)).toBe('Unknown')
    expect(getStatusClass(replicaSet)).toContain('status-badge-secondary')
  })

  it('should show Scaling when spec is missing (defaults to 0 desired)', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetStatus: {
        replicas: 3,
        readyReplicas: 3,
        availableReplicas: 3
      }
    }

    expect(getStatusText(replicaSet)).toBe('Scaling') // defaults to 0 desired, so 3 actual != 0 desired
    expect(getStatusClass(replicaSet)).toContain('status-badge-yellow') // Should be Scaling since replicas (3) != desired (0)
  })

  it('should handle ReplicaFailure condition with False status', () => {
    const replicaSet = {
      kind: 'ReplicaSet',
      replicaSetSpec: { replicas: 3 },
      replicaSetStatus: {
        replicas: 3,
        readyReplicas: 3,
        availableReplicas: 3,
        conditions: [
          {
            type: 'ReplicaFailure',
            status: 'False'
          }
        ]
      }
    }

    expect(getStatusText(replicaSet)).toBe('Ready')
    expect(getStatusClass(replicaSet)).toContain('status-badge-success')
  })
})