<template>
  <div class="elevated-surface rounded-lg p-4">
    <h3 class="text-sm font-semibold text-text-primary mb-3">Ingress Configuration</h3>
    <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <!-- Ingress Class -->
      <div v-if="spec.ingressClassName">
        <dt class="text-xs font-medium text-text-secondary">Ingress Class</dt>
        <dd class="text-sm text-text-primary">
          <span class="status-badge status-badge-info">
            {{ spec.ingressClassName }}
          </span>
        </dd>
      </div>

      <!-- Default Backend -->
      <div v-if="spec.defaultBackend">
        <dt class="text-xs font-medium text-text-secondary">Default Backend</dt>
        <dd class="text-sm text-text-primary">
          <div class="bg-surface-secondary px-2 py-1 rounded border border-border-primary">
            <span class="font-mono text-sm">
              {{ spec.defaultBackend.service?.name || 'Unknown' }}:{{ spec.defaultBackend.service?.port?.number || spec.defaultBackend.service?.port?.name || '?' }}
            </span>
          </div>
        </dd>
      </div>

      <!-- Hosts and Rules -->
      <div v-if="spec.rules && spec.rules.length > 0" class="col-span-2">
        <dt class="text-xs font-medium text-text-secondary mb-2">Hosts & Rules</dt>
        <dd class="text-sm text-text-primary">
          <div class="space-y-3">
            <div v-for="(rule, ruleIndex) in spec.rules" :key="ruleIndex" class="border border-border-primary rounded-lg p-3 bg-surface-secondary">
              <!-- Host -->
              <div class="flex items-center justify-between mb-2">
                <div class="flex items-center space-x-2">
                  <span class="status-badge status-badge-info">
                    {{ rule.host || '*' }}
                  </span>
                  <span v-if="!rule.host" class="text-xs text-text-secondary">(catch-all)</span>
                </div>
                <div class="text-xs text-text-secondary">
                  {{ rule.http?.paths?.length || 0 }} path{{ (rule.http?.paths?.length || 0) !== 1 ? 's' : '' }}
                </div>
              </div>
              
              <!-- Paths -->
              <div v-if="rule.http?.paths && rule.http.paths.length > 0" class="space-y-2">
                <div v-for="(path, pathIndex) in rule.http.paths" :key="pathIndex" class="flex items-center justify-between bg-surface-primary px-2 py-1.5 rounded border border-border-primary">
                  <div class="flex items-center space-x-3">
                    <!-- Path -->
                    <div class="flex items-center space-x-1">
                      <span class="font-mono text-xs bg-accent-subtle px-1.5 py-0.5 rounded">
                        {{ path.path || '/' }}
                      </span>
                      <span v-if="path.pathType" class="text-xs text-text-secondary">
                        ({{ path.pathType }})
                      </span>
                    </div>
                    
                    <!-- Backend Service -->
                    <div class="flex items-center space-x-2">
                      <span class="text-xs text-text-secondary">→</span>
                      <span class="font-mono text-xs">
                        {{ path.backend?.service?.name || 'Unknown Service' }}
                      </span>
                      <span class="text-xs text-text-secondary">:</span>
                      <span class="font-mono text-xs text-text-primary">
                        {{ path.backend?.service?.port?.number || path.backend?.service?.port?.name || '?' }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </dd>
      </div>

      <!-- TLS Configuration -->
      <div v-if="spec.tls && spec.tls.length > 0" class="col-span-2">
        <dt class="text-xs font-medium text-text-secondary mb-2">TLS Configuration</dt>
        <dd class="text-sm text-text-primary">
          <div class="space-y-2">
            <div v-for="(tls, tlsIndex) in spec.tls" :key="tlsIndex" class="flex items-center justify-between bg-surface-secondary px-3 py-2 rounded border border-border-primary">
              <div class="flex items-center space-x-3">
                <!-- Secret Name -->
                <span class="font-mono text-sm">
                  {{ tls.secretName || 'Default Certificate' }}
                </span>
                <!-- Hosts -->
                <div v-if="tls.hosts && tls.hosts.length > 0" class="flex flex-wrap gap-1">
                  <span v-for="(host, hostIndex) in tls.hosts" :key="hostIndex" class="status-badge status-badge-success text-xs">
                    🔒 {{ host }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </dd>
      </div>

      <!-- Load Balancer Status -->
      <div v-if="status?.loadBalancer?.ingress && status.loadBalancer.ingress.length > 0" class="col-span-2">
        <dt class="text-xs font-medium text-text-secondary mb-2">Load Balancer</dt>
        <dd class="text-sm text-text-primary">
          <div class="space-y-1">
            <div v-for="(lb, lbIndex) in status.loadBalancer.ingress" :key="lbIndex" class="flex items-center space-x-2">
              <span v-if="lb.ip" class="font-mono text-sm bg-surface-secondary px-2 py-1 rounded border border-border-primary">
                {{ lb.ip }}
              </span>
              <span v-if="lb.hostname" class="font-mono text-sm bg-surface-secondary px-2 py-1 rounded border border-border-primary">
                {{ lb.hostname }}
              </span>
              <div v-if="lb.ports && lb.ports.length > 0" class="flex space-x-1">
                <span v-for="(port, portIndex) in lb.ports" :key="portIndex" class="text-xs text-text-secondary">
                  :{{ port.port }}{{ port.protocol ? `/${port.protocol}` : '' }}
                </span>
              </div>
            </div>
          </div>
        </dd>
      </div>
    </dl>
  </div>
</template>

<script setup lang="ts">
interface Props {
  spec: {
    ingressClassName?: string
    defaultBackend?: {
      service?: {
        name?: string
        port?: {
          number?: number
          name?: string
        }
      }
    }
    rules?: Array<{
      host?: string
      http?: {
        paths?: Array<{
          path?: string
          pathType?: string
          backend?: {
            service?: {
              name?: string
              port?: {
                number?: number
                name?: string
              }
            }
          }
        }>
      }
    }>
    tls?: Array<{
      secretName?: string
      hosts?: string[]
    }>
  }
  status?: {
    loadBalancer?: {
      ingress?: Array<{
        ip?: string
        hostname?: string
        ports?: Array<{
          port?: number
          protocol?: string
        }>
      }>
    }
  }
}

defineProps<Props>()
</script>