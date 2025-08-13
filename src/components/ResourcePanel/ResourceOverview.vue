<template>
  <div class="h-full overflow-y-auto p-6 space-y-6">
    <!-- Resource Information -->
    <div class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">{{ resourceKind }} Information</h3>
      <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div>
          <dt class="text-xs font-medium text-text-secondary">Name</dt>
          <dd class="text-sm text-text-primary font-mono">{{ resourceData?.metadata?.name }}</dd>
        </div>
        <div v-if="resourceData?.metadata?.namespace">
          <dt class="text-xs font-medium text-text-secondary">Namespace</dt>
          <dd class="text-sm text-text-primary">{{ resourceData.metadata.namespace }}</dd>
        </div>
        <div>
          <dt class="text-xs font-medium text-text-secondary">Kind</dt>
          <dd class="text-sm text-text-primary">{{ resourceKind }}</dd>
        </div>
        <div v-if="resourceData?.metadata?.uid">
          <dt class="text-xs font-medium text-text-secondary">UID</dt>
          <dd class="text-sm text-text-primary font-mono text-xs">{{ resourceData.metadata.uid }}</dd>
        </div>
        <!-- Resource-specific fields -->
        <template v-for="field in getResourceSpecificFields()" :key="field.key">
          <div>
            <dt class="text-xs font-medium text-text-secondary">{{ field.label }}</dt>
            <dd class="text-sm text-text-primary" :class="field.mono ? 'font-mono' : ''">{{ field.value }}</dd>
          </div>
        </template>
      </dl>
    </div>

    <!-- Ingress Hosts -->
    <div v-if="resourceKind === 'Ingress' && getGenericSpec(resourceData)?.rules?.length" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Hosts
        <span class="text-xs font-normal text-text-secondary ml-2">({{ getGenericSpec(resourceData).rules.length }})</span>
      </h3>
      <div class="space-y-3">
        <div v-for="(rule, index) in getGenericSpec(resourceData).rules" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <!-- Host header -->
              <div class="flex items-center gap-2 mb-2">
                <span class="status-badge status-badge-info">
                  {{ rule.host || '*' }}
                </span>
                <span v-if="rule.http?.paths?.length" class="text-xs text-text-secondary">
                  {{ rule.http.paths.length }} path{{ rule.http.paths.length !== 1 ? 's' : '' }}
                </span>
              </div>
              
              <!-- Paths -->
              <div v-if="rule.http?.paths?.length" class="space-y-2">
                <div v-for="(path, pathIndex) in rule.http.paths" :key="pathIndex"
                     class="bg-surface-tertiary rounded p-2">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="status-badge status-badge-success font-mono">
                      {{ path.path || '/' }}
                    </span>
                    <span class="status-badge status-badge-secondary">
                      {{ path.pathType || 'Prefix' }}
                    </span>
                  </div>
                  
                  <div class="text-xs text-text-secondary">
                    <div class="flex items-center gap-1">
                      <span class="font-medium">→</span>
                      <span class="font-mono">{{ path.backend?.service?.name || path.backend?.resource?.name || 'Unknown' }}</span>
                      <span v-if="path.backend?.service?.port" class="text-text-muted">
                        :{{ path.backend.service.port.number || path.backend.service.port.name }}
                      </span>
                    </div>
                  </div>
                  
                  <!-- Full URL display -->
                  <div class="mt-1 text-xs text-text-secondary">
                    <span class="font-medium">URL:</span>
                    <a 
                      :href="getIngressUrl(rule.host, path.path)" 
                      target="_blank" 
                      rel="noopener noreferrer"
                      class="font-mono ml-1 text-accent-primary hover:text-accent-hover hover:underline transition-colors"
                      :title="`Open ${getIngressUrl(rule.host, path.path)} in new tab`"
                    >
                      {{ getIngressUrl(rule.host, path.path) }}
                    </a>
                  </div>
                </div>
              </div>
              
              <!-- No paths case -->
              <div v-else class="text-xs text-text-secondary italic">
                No HTTP paths configured
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyIngressRule(rule)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy rule for ${rule.host || 'default host'}`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 002 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Labels -->
    <div v-if="resourceData?.metadata?.labels" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Labels
        <span class="text-xs font-normal text-text-secondary ml-2">({{ Object.keys(resourceData.metadata.labels).length }})</span>
      </h3>
      <div class="flex flex-wrap gap-2">
        <div v-for="(value, key) in resourceData.metadata.labels" :key="key"
             class="inline-flex items-center group">
          <span class="status-badge status-badge-info rounded-l-full border-r-0">
            {{ key }}
          </span>
          <span class="status-badge status-badge-info rounded-r-full border-l-0 opacity-80">
            {{ value }}
          </span>
          <button
            @click="copyLabel(String(key), value)"
            class="ml-1 p-1 opacity-0 group-hover:opacity-100 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-all"
            :title="`Copy ${String(key)}=${value}`"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Annotations -->
    <div v-if="resourceData?.metadata?.annotations && Object.keys(resourceData.metadata.annotations).length > 0" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Annotations
        <span class="text-xs font-normal text-text-secondary ml-2">({{ Object.keys(resourceData.metadata.annotations).length }})</span>
      </h3>
      <div class="space-y-2">
        <div v-for="(value, key) in resourceData.metadata.annotations" :key="key"
             class="bg-surface-secondary rounded border border-purple-200 dark:border-purple-700 p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="status-badge status-badge-secondary">
                  {{ key }}
                </span>
                <button
                  v-if="isLargeAnnotation(value)"
                  @click="toggleAnnotation(String(key))"
                  class="text-xs text-text-secondary hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
                >
                  {{ expandedAnnotations.has(String(key)) ? 'Collapse' : 'Expand' }}
                </button>
              </div>
              <div class="text-xs text-text-primary font-mono">
                <div v-if="!isLargeAnnotation(value)" class="break-all">
                  {{ value }}
                </div>
                <div v-else>
                  <div v-if="expandedAnnotations.has(String(key))" class="break-all whitespace-pre-wrap bg-surface-tertiary p-2 rounded border max-h-60 overflow-y-auto">
                    {{ formatAnnotationValue(value) }}
                  </div>
                  <div v-else class="text-text-secondary">
                    {{ getTruncatedValue(value) }}
                    <button
                      @click="toggleAnnotation(String(key))"
                      class="ml-1 text-accent-primary hover:underline"
                    >
                      Show more
                    </button>
                  </div>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-1 flex-shrink-0">
              <button
                @click="copyAnnotation(String(key), value)"
                class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                :title="`Copy ${String(key)}`"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>


    <!-- Pod Volumes -->
    <PodVolumes
      v-if="resourceKind === 'Pod' && getGenericSpec(resourceData)?.volumes"
      :volumes="getGenericSpec(resourceData).volumes"
      :containers="getGenericSpec(resourceData).containers"
      :initContainers="getGenericSpec(resourceData).initContainers"
    />

    <!-- Pod Tolerations -->
    <div v-if="resourceKind === 'Pod' && getGenericSpec(resourceData)?.tolerations?.length" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Tolerations
        <span class="text-xs font-normal text-text-secondary ml-2">({{ getGenericSpec(resourceData).tolerations.length }})</span>
      </h3>
      <div class="space-y-2">
        <div v-for="(toleration, index) in getGenericSpec(resourceData).tolerations" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-2">
                <span v-if="toleration.key" class="status-badge status-badge-yellow">
                  {{ toleration.key }}
                </span>
                <span v-else class="status-badge status-badge-secondary">
                  No Key
                </span>
                <span class="status-badge status-badge-info">
                  {{ toleration.effect || 'NoSchedule' }}
                </span>
              </div>
              
              <div class="text-xs space-y-1">
                <div v-if="toleration.operator" class="text-text-secondary">
                  <span class="font-medium">Operator:</span> {{ toleration.operator }}
                </div>
                <div v-if="toleration.value" class="text-text-secondary">
                  <span class="font-medium">Value:</span> {{ toleration.value }}
                </div>
                <div v-if="toleration.tolerationSeconds !== undefined" class="text-text-secondary">
                  <span class="font-medium">Toleration Seconds:</span> {{ toleration.tolerationSeconds }}s
                </div>
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyToleration(toleration)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy toleration ${toleration.key || 'configuration'}`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <PodConditions
      v-if="resourceKind === 'Pod' && getGenericStatus(resourceData)?.conditions"
      :conditions="getGenericStatus(resourceData).conditions"
    />

    <!-- Service-specific configuration -->
    <ServiceConfiguration
      v-if="resourceKind === 'Service' && getGenericSpec(resourceData)"
      :spec="getGenericSpec(resourceData)"
    />

    <!-- Ingress-specific configuration -->
    <IngressConfiguration
      v-if="resourceKind === 'Ingress' && getGenericSpec(resourceData)"
      :spec="getGenericSpec(resourceData)"
      :status="getGenericStatus(resourceData)"
    />

    <!-- Deployment/ReplicaSet-specific configuration -->
    <WorkloadConfiguration
      v-if="['Deployment', 'ReplicaSet', 'StatefulSet', 'DaemonSet'].includes(resourceKind) && getGenericSpec(resourceData)"
      :resourceKind="resourceKind"
      :resourceName="resourceData?.metadata?.name || ''"
      :namespace="resourceData?.metadata?.namespace"
      :spec="getGenericSpec(resourceData)"
      :status="getGenericStatus(resourceData)"
      @scaled="handleResourceScaled"
    />

    <!-- CronJob-specific configuration -->
    <div v-if="resourceKind === 'CronJob' && resourceData?.spec" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">CronJob Configuration</h3>
      <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <!-- Schedule -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Schedule</dt>
          <dd class="mt-1 text-sm text-text-primary">
            <span class="font-mono">{{ resourceData.spec.schedule || '-' }}</span>
            <span v-if="getGenericSpec(resourceData).schedule" class="text-xs text-text-secondary ml-2">
              ({{ getCronDescription(resourceData.spec.schedule) }})
            </span>
          </dd>
        </div>
        
        <!-- Timezone -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Timezone</dt>
          <dd class="mt-1 text-sm text-text-primary">
            {{ resourceData.spec.timeZone || 'UTC' }}
          </dd>
        </div>
        
        <!-- Concurrency Policy -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Concurrency Policy</dt>
          <dd class="mt-1">
            <span :class="[
              'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
              getConcurrencyPolicyClass(resourceData.spec.concurrencyPolicy)
            ]">
              {{ resourceData.spec.concurrencyPolicy || 'Allow' }}
            </span>
          </dd>
        </div>
        
        <!-- Suspend -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Suspended</dt>
          <dd class="mt-1">
            <span :class="[
              'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
              resourceData.spec.suspend 
                ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300'
                : 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
            ]">
              {{ resourceData.spec.suspend ? 'Yes' : 'No' }}
            </span>
          </dd>
        </div>
        
        <!-- Successful Jobs History Limit -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Successful Jobs History</dt>
          <dd class="mt-1 text-sm text-text-primary">
            {{ resourceData.spec.successfulJobsHistoryLimit ?? 3 }} jobs
          </dd>
        </div>
        
        <!-- Failed Jobs History Limit -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Failed Jobs History</dt>
          <dd class="mt-1 text-sm text-text-primary">
            {{ resourceData.spec.failedJobsHistoryLimit ?? 1 }} jobs
          </dd>
        </div>
      </dl>
    </div>

    <!-- Job Configuration -->
    <div v-if="resourceKind === 'Job' && resourceData?.spec" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">Job Configuration</h3>
      <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <!-- Completions -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Completions</dt>
          <dd class="mt-1 text-sm text-text-primary">
            {{ resourceData.spec.completions || 1 }}
          </dd>
        </div>
        
        <!-- Parallelism -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Parallelism</dt>
          <dd class="mt-1 text-sm text-text-primary">
            {{ resourceData.spec.parallelism || 1 }}
          </dd>
        </div>
        
        <!-- Backoff Limit -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Backoff Limit</dt>
          <dd class="mt-1 text-sm text-text-primary">
            {{ resourceData.spec.backoffLimit ?? 6 }} retries
          </dd>
        </div>
        
        <!-- Completion Mode -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Completion Mode</dt>
          <dd class="mt-1">
            <span :class="[
              'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
              resourceData.spec.completionMode === 'Indexed' 
                ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
                : 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
            ]">
              {{ resourceData.spec.completionMode || 'NonIndexed' }}
            </span>
          </dd>
        </div>
        
        <!-- Manual Selector -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Manual Selector</dt>
          <dd class="mt-1">
            <span :class="[
              'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
              resourceData.spec.manualSelector 
                ? 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300'
                : 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
            ]">
              {{ resourceData.spec.manualSelector ? 'Yes' : 'No' }}
            </span>
          </dd>
        </div>
        
        <!-- Pod Replacement Policy -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Pod Replacement Policy</dt>
          <dd class="mt-1">
            <span :class="[
              'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
              getPodReplacementPolicyClass(resourceData.spec.podReplacementPolicy)
            ]">
              {{ resourceData.spec.podReplacementPolicy || 'TerminatingOrFailed' }}
            </span>
          </dd>
        </div>
      </dl>
    </div>

    <!-- Job Conditions -->
    <div v-if="resourceKind === 'Job' && getGenericStatus(resourceData)?.conditions?.length" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Job Conditions
        <span class="text-xs font-normal text-text-secondary ml-2">({{ getGenericStatus(resourceData).conditions.length }})</span>
      </h3>
      <div class="space-y-2">
        <div v-for="(condition, index) in getGenericStatus(resourceData).conditions" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span :class="[
                  'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
                  condition.status === 'True' 
                    ? 'status-badge-success'
                    : condition.status === 'False' 
                      ? 'status-badge-error'
                      : 'status-badge-yellow'
                ]">
                  {{ condition.type }}
                </span>
                <span :class="[
                  'text-xs px-1.5 py-0.5 rounded',
                  condition.status === 'True' 
                    ? 'status-badge-success opacity-75'
                    : condition.status === 'False' 
                      ? 'status-badge-error opacity-75'
                      : 'bg-yellow-50 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-300'
                ]">
                  {{ condition.status }}
                </span>
                <span v-if="condition.lastTransitionTime" class="text-xs text-text-secondary">
                  {{ getRelativeTime(new Date().getTime() - new Date(condition.lastTransitionTime).getTime()) }} ago
                </span>
              </div>
              <div class="text-xs space-y-1">
                <div v-if="condition.reason" class="text-text-secondary">
                  <span class="font-medium">Reason:</span> {{ condition.reason }}
                </div>
                <div v-if="condition.message" class="text-text-secondary">
                  <span class="font-medium">Message:</span> {{ condition.message }}
                </div>
                <div v-if="condition.lastProbeTime" class="text-text-muted">
                  <span class="font-medium">Last Probe:</span> {{ new Date(condition.lastProbeTime).toLocaleString() }}
                </div>
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyJobCondition(condition)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy condition ${condition.type}`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- PodDisruptionBudget Conditions -->
    <div v-if="resourceKind === 'PodDisruptionBudget' && getGenericStatus(resourceData)?.conditions?.length" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Conditions
        <span class="text-xs font-normal text-text-secondary ml-2">({{ getGenericStatus(resourceData).conditions.length }})</span>
      </h3>
      <div class="space-y-2">
        <div v-for="(condition, index) in getGenericStatus(resourceData).conditions" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span :class="[
                  'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
                  condition.status === 'True' 
                    ? 'status-badge-success'
                    : condition.status === 'False' 
                      ? 'status-badge-error'
                      : 'status-badge-yellow'
                ]">
                  {{ condition.type }}
                </span>
                <span :class="[
                  'text-xs px-1.5 py-0.5 rounded',
                  condition.status === 'True' 
                    ? 'status-badge-success opacity-75'
                    : condition.status === 'False' 
                      ? 'status-badge-error opacity-75'
                      : 'bg-yellow-50 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-300'
                ]">
                  {{ condition.status }}
                </span>
                <span v-if="condition.lastTransitionTime" class="text-xs text-text-secondary">
                  {{ getRelativeTime(new Date().getTime() - new Date(condition.lastTransitionTime).getTime()) }} ago
                </span>
              </div>
              <div class="text-xs space-y-1">
                <div v-if="condition.reason" class="text-text-secondary">
                  <span class="font-medium">Reason:</span> {{ condition.reason }}
                </div>
                <div v-if="condition.message" class="text-text-secondary">
                  <span class="font-medium">Message:</span> {{ condition.message }}
                </div>
                <div v-if="condition.lastProbeTime" class="text-text-muted">
                  <span class="font-medium">Last Probe:</span> {{ new Date(condition.lastProbeTime).toLocaleString() }}
                </div>
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyPDBCondition(condition)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy condition ${condition.type}`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Node Addresses -->
    <div v-if="resourceKind === 'Node' && getGenericStatus(resourceData)?.addresses?.length" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Addresses
        <span class="text-xs font-normal text-text-secondary ml-2">({{ getGenericStatus(resourceData).addresses.length }})</span>
      </h3>
      <div class="space-y-2">
        <div v-for="(address, index) in getGenericStatus(resourceData).addresses" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="status-badge status-badge-info">
                  {{ address.type }}
                </span>
              </div>
              <div class="text-sm text-text-primary font-mono">
                {{ address.address }}
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyLabel('address', address.address)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy ${address.type} address`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Node Capacity -->
    <div v-if="resourceKind === 'Node' && getGenericStatus(resourceData)?.capacity" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Capacity
        <span class="text-xs font-normal text-text-secondary ml-2">({{ Object.keys(getGenericStatus(resourceData).capacity).length }} resources)</span>
        <Tooltip content="Total node resources available, including system reserved resources" side="top">
          <span class="ml-1 inline-flex items-center">
            <svg class="w-3 h-3 text-gray-400 cursor-help" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"></path>
            </svg>
          </span>
        </Tooltip>
      </h3>
      <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div v-for="(value, key) in getGenericStatus(resourceData).capacity" :key="key">
          <dt class="text-xs font-medium text-text-secondary">
            {{ key }}
            <Tooltip v-if="String(key).startsWith('hugepages-')" :content="`Large memory pages (${String(key).split('-')[1]}) that reduce TLB misses and improve memory performance for memory-intensive applications`" side="top">
              <span class="ml-1 inline-flex items-center">
                <svg class="w-3 h-3 text-gray-400 cursor-help" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"></path>
                </svg>
              </span>
            </Tooltip>
          </dt>
          <dd class="text-sm text-text-primary font-mono">
            <span v-if="String(key) === 'memory'">{{ formatMemory(String(value)) }}</span>
            <span v-else-if="String(key) === 'ephemeral-storage'">{{ formatMemory(String(value)) }}</span>
            <span v-else>{{ value }}</span>
          </dd>
        </div>
      </dl>
    </div>

    <!-- Node Allocatable -->
    <div v-if="resourceKind === 'Node' && getGenericStatus(resourceData)?.allocatable" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Allocatable
        <span class="text-xs font-normal text-text-secondary ml-2">({{ Object.keys(getGenericStatus(resourceData).allocatable).length }} resources)</span>
        <Tooltip content="Resources available for scheduling pods after system reservations (kubelet, kube-proxy, OS, etc.)" side="top">
          <span class="ml-1 inline-flex items-center">
            <svg class="w-3 h-3 text-gray-400 cursor-help" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"></path>
            </svg>
          </span>
        </Tooltip>
      </h3>
      <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div v-for="(value, key) in getGenericStatus(resourceData).allocatable" :key="key">
          <dt class="text-xs font-medium text-text-secondary">
            {{ key }}
            <Tooltip v-if="String(key).startsWith('hugepages-')" :content="`Large memory pages (${String(key).split('-')[1]}) that reduce TLB misses and improve memory performance for memory-intensive applications`" side="top">
              <span class="ml-1 inline-flex items-center">
                <svg class="w-3 h-3 text-gray-400 cursor-help" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"></path>
                </svg>
              </span>
            </Tooltip>
          </dt>
          <dd class="text-sm text-text-primary font-mono">
            <span v-if="String(key) === 'memory'">{{ formatMemory(String(value)) }}</span>
            <span v-else-if="String(key) === 'ephemeral-storage'">{{ formatMemory(String(value)) }}</span>
            <span v-else>{{ value }}</span>
          </dd>
        </div>
      </dl>
    </div>

    <!-- Node Conditions -->
    <div v-if="resourceKind === 'Node' && getGenericStatus(resourceData)?.conditions?.length" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Conditions
        <span class="text-xs font-normal text-text-secondary ml-2">({{ getGenericStatus(resourceData).conditions.length }})</span>
      </h3>
      <div class="space-y-2">
        <div v-for="(condition, index) in getGenericStatus(resourceData).conditions" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span :class="[
                  'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
                  condition.status === 'True' 
                    ? 'status-badge-success'
                    : condition.status === 'False' 
                      ? 'status-badge-error'
                      : 'status-badge-yellow'
                ]">
                  {{ condition.type }}
                </span>
                <span :class="[
                  'text-xs px-1.5 py-0.5 rounded',
                  condition.status === 'True' 
                    ? 'status-badge-success opacity-75'
                    : condition.status === 'False' 
                      ? 'status-badge-error opacity-75'
                      : 'bg-yellow-50 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-300'
                ]">
                  {{ condition.status }}
                </span>
                <span v-if="condition.lastTransitionTime" class="text-xs text-text-secondary">
                  {{ getRelativeTime(new Date().getTime() - new Date(condition.lastTransitionTime).getTime()) }} ago
                </span>
              </div>
              <div class="text-xs space-y-1">
                <div v-if="condition.reason" class="text-text-secondary">
                  <span class="font-medium">Reason:</span> {{ condition.reason }}
                </div>
                <div v-if="condition.message" class="text-text-secondary">
                  <span class="font-medium">Message:</span> {{ condition.message }}
                </div>
                <div v-if="condition.lastHeartbeatTime" class="text-text-muted">
                  <span class="font-medium">Last Heartbeat:</span> {{ new Date(condition.lastHeartbeatTime).toLocaleString() }}
                </div>
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyNodeCondition(condition)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy condition ${condition.type}`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Role Rules -->
    <div v-if="resourceKind === 'Role' && (resourceData?.roleSpec?.rules || resourceData?.completeObject?.rules)" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Rules
        <span class="text-xs font-normal text-text-secondary ml-2">({{ (resourceData?.roleSpec?.rules || resourceData?.completeObject?.rules || []).length }})</span>
      </h3>
      <div class="space-y-3">
        <div v-for="(rule, index) in (resourceData?.roleSpec?.rules || resourceData?.completeObject?.rules || [])" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <!-- API Groups -->
              <div class="flex items-center gap-2 mb-2">
                <span class="text-xs font-medium text-text-secondary">API Groups:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(apiGroup, apiIndex) in (rule.apiGroups || [''])" :key="apiIndex"
                        class="status-badge status-badge-info">
                    {{ apiGroup === '' ? 'core' : apiGroup }}
                  </span>
                </div>
              </div>
              
              <!-- Resources -->
              <div class="flex items-center gap-2 mb-2">
                <span class="text-xs font-medium text-text-secondary">Resources:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(resource, resIndex) in (rule.resources || [])" :key="resIndex"
                        class="status-badge status-badge-success">
                    {{ resource }}
                  </span>
                </div>
              </div>
              
              <!-- Resource Names (if specified) -->
              <div v-if="rule.resourceNames?.length" class="flex items-center gap-2 mb-2">
                <span class="text-xs font-medium text-text-secondary">Resource Names:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(resourceName, nameIndex) in rule.resourceNames" :key="nameIndex"
                        class="status-badge status-badge-yellow">
                    {{ resourceName }}
                  </span>
                </div>
              </div>
              
              <!-- Verbs -->
              <div class="flex items-center gap-2">
                <span class="text-xs font-medium text-text-secondary">Verbs:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(verb, verbIndex) in (rule.verbs || [])" :key="verbIndex"
                        class="status-badge status-badge-secondary">
                    {{ verb }}
                  </span>
                </div>
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyRoleRule(rule)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy rule ${index + 1}`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 002 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- ClusterRole Rules -->
    <div v-if="resourceKind === 'ClusterRole' && (resourceData?.clusterRoleSpec?.rules || resourceData?.completeObject?.rules)" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Rules
        <span class="text-xs font-normal text-text-secondary ml-2">({{ (resourceData?.clusterRoleSpec?.rules || resourceData?.completeObject?.rules || []).length }})</span>
      </h3>
      <div class="space-y-3">
        <div v-for="(rule, index) in (resourceData?.clusterRoleSpec?.rules || resourceData?.completeObject?.rules || [])" :key="index"
             class="bg-surface-secondary rounded border border-border-primary p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <!-- API Groups -->
              <div class="flex items-center gap-2 mb-2">
                <span class="text-xs font-medium text-text-secondary">API Groups:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(apiGroup, apiIndex) in (rule.apiGroups || [''])" :key="apiIndex"
                        class="status-badge status-badge-info">
                    {{ apiGroup === '' ? 'core' : apiGroup }}
                  </span>
                </div>
              </div>
              
              <!-- Resources -->
              <div class="flex items-center gap-2 mb-2">
                <span class="text-xs font-medium text-text-secondary">Resources:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(resource, resIndex) in (rule.resources || [])" :key="resIndex"
                        class="status-badge status-badge-success">
                    {{ resource }}
                  </span>
                </div>
              </div>
              
              <!-- Resource Names (if specified) -->
              <div v-if="rule.resourceNames?.length" class="flex items-center gap-2 mb-2">
                <span class="text-xs font-medium text-text-secondary">Resource Names:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(resourceName, nameIndex) in rule.resourceNames" :key="nameIndex"
                        class="status-badge status-badge-yellow">
                    {{ resourceName }}
                  </span>
                </div>
              </div>
              
              <!-- Verbs -->
              <div class="flex items-center gap-2">
                <span class="text-xs font-medium text-text-secondary">Verbs:</span>
                <div class="flex flex-wrap gap-1">
                  <span v-for="(verb, verbIndex) in (rule.verbs || [])" :key="verbIndex"
                        class="status-badge status-badge-secondary">
                    {{ verb }}
                  </span>
                </div>
              </div>
            </div>
            
            <!-- Copy button -->
            <button
              @click="copyClusterRoleRule(rule)"
              class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors flex-shrink-0"
              :title="`Copy rule ${index + 1}`"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 002 2v8a2 2 0 002 2z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- RoleBinding Subjects -->
    <div v-if="resourceKind === 'RoleBinding' && (resourceData?.roleBindingSpec?.subjects || resourceData?.completeObject?.subjects)" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Subjects
        <span class="text-xs font-normal text-text-secondary ml-2">({{ (resourceData?.roleBindingSpec?.subjects || resourceData?.completeObject?.subjects || []).length }})</span>
      </h3>
      <div class="overflow-x-auto">
        <table class="min-w-full table-background">
          <thead class="table-header-background border-b border-border-primary">
            <tr>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">Kind</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">Name</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">Namespace</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">API Group</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider w-16">Actions</th>
            </tr>
          </thead>
          <tbody class="table-background divide-y divide-border-primary">
            <tr v-for="(subject, index) in (resourceData?.roleBindingSpec?.subjects || resourceData?.completeObject?.subjects || [])" 
                :key="index"
                class="table-row-background">
              <!-- Kind -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span class="status-badge status-badge-info">
                  {{ subject.kind }}
                </span>
              </td>
              
              <!-- Name -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span class="text-sm table-cell-text font-mono">
                  {{ subject.name }}
                </span>
              </td>
              
              <!-- Namespace -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span v-if="subject.namespace" class="status-badge status-badge-yellow">
                  {{ subject.namespace }}
                </span>
                <span v-else class="text-text-muted text-xs">-</span>
              </td>
              
              <!-- API Group -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span v-if="subject.apiGroup" class="status-badge status-badge-secondary">
                  {{ subject.apiGroup }}
                </span>
                <span v-else class="text-text-muted text-xs">-</span>
              </td>
              
              <!-- Actions -->
              <td class="px-3 py-2 whitespace-nowrap">
                <button
                  @click="copyRoleBindingSubject(subject)"
                  class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                  :title="`Copy subject ${index + 1}`"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 002 2v8a2 2 0 002 2z"/>
                  </svg>
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- ClusterRoleBinding Subjects -->
    <div v-if="resourceKind === 'ClusterRoleBinding' && (resourceData?.clusterRoleBindingSpec?.subjects || resourceData?.completeObject?.subjects)" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">
        Subjects
        <span class="text-xs font-normal text-text-secondary ml-2">({{ (resourceData?.clusterRoleBindingSpec?.subjects || resourceData?.completeObject?.subjects || []).length }})</span>
      </h3>
      <div class="overflow-x-auto">
        <table class="min-w-full table-background">
          <thead class="table-header-background border-b border-border-primary">
            <tr>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">Kind</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">Name</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">Namespace</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider">API Group</th>
              <th class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider w-16">Actions</th>
            </tr>
          </thead>
          <tbody class="table-background divide-y divide-border-primary">
            <tr v-for="(subject, index) in (resourceData?.clusterRoleBindingSpec?.subjects || resourceData?.completeObject?.subjects || [])" 
                :key="index"
                class="table-row-background">
              <!-- Kind -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span class="status-badge status-badge-info">
                  {{ subject.kind }}
                </span>
              </td>
              
              <!-- Name -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span class="text-sm table-cell-text font-mono">
                  {{ subject.name }}
                </span>
              </td>
              
              <!-- Namespace -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span v-if="subject.namespace" class="status-badge status-badge-yellow">
                  {{ subject.namespace }}
                </span>
                <span v-else class="text-text-muted text-xs">-</span>
              </td>
              
              <!-- API Group -->
              <td class="px-3 py-2 whitespace-nowrap">
                <span v-if="subject.apiGroup" class="status-badge status-badge-secondary">
                  {{ subject.apiGroup }}
                </span>
                <span v-else class="text-text-muted text-xs">-</span>
              </td>
              
              <!-- Actions -->
              <td class="px-3 py-2 whitespace-nowrap">
                <button
                  @click="copyClusterRoleBindingSubject(subject)"
                  class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                  :title="`Copy subject ${index + 1}`"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 002 2v8a2 2 0 002 2z"/>
                  </svg>
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- CronJob Status -->
    <div v-if="resourceKind === 'CronJob' && resourceData?.status" class="elevated-surface rounded-lg p-4">
      <h3 class="text-sm font-semibold text-status-warning mb-3">CronJob Status</h3>
      <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <!-- Last Schedule Time -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Last Schedule</dt>
          <dd class="mt-1 text-sm text-text-primary">
            <div v-if="getGenericStatus(resourceData).lastScheduleTime">
              {{ getRelativeTime(new Date().getTime() - new Date(getGenericStatus(resourceData).lastScheduleTime).getTime()) }} ago
              <div class="text-xs text-text-secondary mt-0.5">
                {{ new Date(getGenericStatus(resourceData).lastScheduleTime).toLocaleString() }}
              </div>
            </div>
            <span v-else class="text-text-secondary">Never</span>
          </dd>
        </div>
        
        <!-- Next Schedule Time -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Next Schedule</dt>
          <dd class="mt-1 text-sm text-text-primary">
            <div v-if="getGenericStatus(resourceData).lastSuccessfulTime">
              <span class="text-text-secondary">Calculated from schedule</span>
            </div>
            <span v-else class="text-text-secondary">-</span>
          </dd>
        </div>
        
        <!-- Active Jobs -->
        <div>
          <dt class="text-xs font-medium text-text-secondary">Active Jobs</dt>
          <dd class="mt-1">
            <span :class="[
              'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
              (getGenericStatus(resourceData).active?.length || 0) > 0
                ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
                : 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
            ]">
              {{ getGenericStatus(resourceData).active?.length || 0 }}
            </span>
            <div v-if="getGenericStatus(resourceData).active?.length" class="mt-1 space-y-1">
              <div v-for="activeJob in getGenericStatus(resourceData).active" :key="activeJob.name" class="text-xs text-text-secondary">
                → {{ activeJob.name }}
              </div>
            </div>
          </dd>
        </div>
        
        <!-- Last Successful Time -->
        <div v-if="getGenericStatus(resourceData).lastSuccessfulTime">
          <dt class="text-xs font-medium text-text-secondary">Last Successful</dt>
          <dd class="mt-1 text-sm text-text-primary">
            {{ getRelativeTime(new Date().getTime() - new Date(getGenericStatus(resourceData).lastSuccessfulTime).getTime()) }} ago
            <div class="text-xs text-text-secondary mt-0.5">
              {{ new Date(getGenericStatus(resourceData).lastSuccessfulTime).toLocaleString() }}
            </div>
          </dd>
        </div>
      </dl>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useResourceStatus } from '@/composables/useResourceStatus'
import PodConditions from './PodConditions.vue'
import PodVolumes from './PodVolumes.vue'
import ServiceConfiguration from './ServiceConfiguration.vue'
import IngressConfiguration from './IngressConfiguration.vue'
import WorkloadConfiguration from './WorkloadConfiguration.vue'
import Tooltip from '@/components/ui/Tooltip.vue'

interface Props {
  resourceData: any | null
  resourceKind: string
}

const props = defineProps<Props>()

// Helper functions for accessing resource-specific fields
const { getGenericStatus, getGenericSpec } = useResourceStatus()

// State for managing expanded annotations
const expandedAnnotations = ref(new Set<string>())

function getRelativeTime(diffMs: number): string {
  const seconds = Math.floor(diffMs / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  const months = Math.floor(days / 30)
  const years = Math.floor(days / 365)

  if (years > 0) return `${years} year${years > 1 ? 's' : ''} ago`
  if (months > 0) return `${months} month${months > 1 ? 's' : ''} ago`
  if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`
  if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`
  if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`
  if (seconds > 30) return `${seconds} second${seconds > 1 ? 's' : ''} ago`
  return 'Just now'
}

function getResourceSpecificFields(): Array<{key: string, label: string, value: string, mono?: boolean}> {
  const fields: Array<{key: string, label: string, value: string, mono?: boolean}> = []
  
  if (props.resourceKind === 'Pod' && props.resourceData) {
    // Created date
    if (props.resourceData.metadata?.creationTimestamp) {
      const createdDate = new Date(props.resourceData.metadata.creationTimestamp)
      const now = new Date()
      const diffMs = now.getTime() - createdDate.getTime()
      const relativeTime = getRelativeTime(diffMs)
      fields.push({ 
        key: 'created', 
        label: 'Created', 
        value: `${createdDate.toLocaleString()} (${relativeTime})` 
      })
    }
    
    // Controlled By - extract from ownerReferences
    if (props.resourceData.metadata?.ownerReferences?.length > 0) {
      const controller = props.resourceData.metadata.ownerReferences.find((ref: any) => ref.controller === true)
      if (controller) {
        fields.push({ 
          key: 'controlledBy', 
          label: 'Controlled By', 
          value: `${controller.kind}/${controller.name}` 
        })
      }
    }
    
    if (getGenericSpec(props.resourceData)?.serviceAccountName) {
      fields.push({ key: 'serviceAccount', label: 'Service Account', value: getGenericSpec(props.resourceData).serviceAccountName })
    }
    if (getGenericSpec(props.resourceData)?.nodeName) {
      fields.push({ key: 'node', label: 'Node', value: getGenericSpec(props.resourceData).nodeName })
    }
    if (getGenericSpec(props.resourceData)?.priorityClassName) {
      fields.push({ key: 'priorityClass', label: 'Priority Class', value: getGenericSpec(props.resourceData).priorityClassName })
    }
    if (getGenericSpec(props.resourceData)?.terminationGracePeriodSeconds !== undefined) {
      fields.push({ 
        key: 'terminationGracePeriod', 
        label: 'Termination Grace Period', 
        value: `${getGenericSpec(props.resourceData).terminationGracePeriodSeconds}s` 
      })
    }
    if (getGenericSpec(props.resourceData)?.restartPolicy) {
      fields.push({ key: 'restartPolicy', label: 'Restart Policy', value: getGenericSpec(props.resourceData).restartPolicy })
    }
    if (getGenericStatus(props.resourceData)?.qosClass) {
      fields.push({ key: 'qosClass', label: 'QoS Class', value: getGenericStatus(props.resourceData)?.qosClass })
    }
    if (getGenericStatus(props.resourceData)?.podIP) {
      fields.push({ key: 'podIP', label: 'Pod IP', value: getGenericStatus(props.resourceData)?.podIP, mono: true })
    }
    
  }
  
  if (props.resourceKind === 'Service' && props.resourceData?.spec) {
    if (getGenericSpec(props.resourceData).clusterIP) {
      fields.push({ key: 'clusterIP', label: 'Cluster IP', value: getGenericSpec(props.resourceData).clusterIP, mono: true })
    }
    if (getGenericSpec(props.resourceData).type) {
      fields.push({ key: 'type', label: 'Type', value: getGenericSpec(props.resourceData).type })
    }
    if (getGenericSpec(props.resourceData).sessionAffinity) {
      fields.push({ key: 'sessionAffinity', label: 'Session Affinity', value: getGenericSpec(props.resourceData).sessionAffinity })
    }
    
    // LoadBalancer specific fields
    if (getGenericSpec(props.resourceData).type === 'LoadBalancer') {
      // External IP/Hostname from status
      if (getGenericStatus(props.resourceData)?.loadBalancer?.ingress?.length) {
        const ingress = getGenericStatus(props.resourceData)?.loadBalancer?.ingress
        const endpoints = ingress.map((ing: any) => ing.ip || ing.hostname).filter(Boolean)
        if (endpoints.length > 0) {
          fields.push({ 
            key: 'externalEndpoints', 
            label: 'External Endpoints', 
            value: endpoints.length > 1 ? `${endpoints[0]} +${endpoints.length - 1}` : endpoints[0], 
            mono: true 
          })
        }
      } else {
        fields.push({ key: 'externalStatus', label: 'External Status', value: 'Pending' })
      }
      
      // LoadBalancer Class
      if (getGenericSpec(props.resourceData).loadBalancerClass) {
        fields.push({ key: 'loadBalancerClass', label: 'LoadBalancer Class', value: getGenericSpec(props.resourceData).loadBalancerClass })
      }
      
      // Source Ranges
      if (getGenericSpec(props.resourceData).loadBalancerSourceRanges?.length) {
        const ranges = getGenericSpec(props.resourceData).loadBalancerSourceRanges
        fields.push({ 
          key: 'sourceRanges', 
          label: 'Source Ranges', 
          value: ranges.length > 1 ? `${ranges[0]} +${ranges.length - 1}` : ranges[0], 
          mono: true 
        })
      }
    }
  }
  
  if (['Deployment', 'ReplicaSet', 'StatefulSet'].includes(props.resourceKind) && props.resourceData) {
    if (getGenericSpec(props.resourceData)?.replicas !== undefined) {
      fields.push({ key: 'replicas', label: 'Replicas', value: getGenericSpec(props.resourceData).replicas.toString() })
    }
  }
  
  if (props.resourceKind === 'PersistentVolumeClaim' && props.resourceData) {
    // Capacity from status
    if (getGenericStatus(props.resourceData)?.capacity?.storage) {
      fields.push({ 
        key: 'capacity', 
        label: 'Capacity', 
        value: getGenericStatus(props.resourceData).capacity.storage,
        mono: true
      })
    }
    
    // Storage Class from spec
    if (getGenericSpec(props.resourceData)?.storageClassName) {
      fields.push({ 
        key: 'storageClass', 
        label: 'Storage Class', 
        value: getGenericSpec(props.resourceData).storageClassName
      })
    } else {
      // Show default storage class if none specified
      fields.push({ 
        key: 'storageClass', 
        label: 'Storage Class', 
        value: 'default'
      })
    }
  }
  
  return fields
}

function handleResourceScaled(newReplicas: number): void {
  // The resource will automatically update through the watch mechanism
  // No additional action needed here as the backend will emit updated resource data
}

// Annotation management functions
function isLargeAnnotation(value: string | number): boolean {
  const strValue = String(value)
  return strValue.length > 100 || strValue.includes('\n') || strValue.startsWith('{')
}

function getTruncatedValue(value: string | number): string {
  const strValue = String(value)
  if (strValue.length <= 100) return strValue
  return strValue.substring(0, 100) + '...'
}

function formatAnnotationValue(value: string | number): string {
  const strValue = String(value)
  // Try to format JSON if it looks like JSON
  if (strValue.startsWith('{') || strValue.startsWith('[')) {
    try {
      const parsed = JSON.parse(strValue)
      return JSON.stringify(parsed, null, 2)
    } catch {
      // If not valid JSON, return as-is
      return strValue
    }
  }
  return strValue
}

function toggleAnnotation(key: string): void {
  if (expandedAnnotations.value.has(key)) {
    expandedAnnotations.value.delete(key)
  } else {
    expandedAnnotations.value.add(key)
  }
}

async function copyAnnotation(key: string, value: string | number): Promise<void> {
  try {
    await navigator.clipboard.writeText(`${key}: ${value}`)
    // Could add a toast notification here
  } catch (error) {
    console.error('Failed to copy annotation:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `${key}: ${value}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyLabel(key: string, value: string | number): Promise<void> {
  try {
    await navigator.clipboard.writeText(`${key}=${value}`)
  } catch (error) {
    console.error('Failed to copy label:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `${key}=${value}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyToleration(toleration: any): Promise<void> {
  try {
    const tolerationText = `Toleration: ${JSON.stringify(toleration, null, 2)}`
    await navigator.clipboard.writeText(tolerationText)
  } catch (error) {
    console.error('Failed to copy toleration:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `Key: ${toleration.key || 'N/A'}, Effect: ${toleration.effect || 'NoSchedule'}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

function getConcurrencyPolicyClass(policy: string): string {
  switch (policy) {
    case 'Allow':
      return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
    case 'Forbid':
      return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
    case 'Replace':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
  }
}

function getCronDescription(cronExpression: string): string {
  if (!cronExpression) return 'Invalid'
  
  const parts = cronExpression.trim().split(/\s+/)
  if (parts.length !== 5) return 'Invalid format'
  
  const [minute, hour, dayOfMonth, month, dayOfWeek] = parts
  
  // Handle common patterns
  if (cronExpression === '* * * * *') return 'Every minute'
  if (cronExpression === '0 * * * *') return 'Every hour'
  if (cronExpression === '0 0 * * *') return 'Daily at midnight'
  if (cronExpression === '0 0 * * 0') return 'Weekly on Sunday at midnight'
  if (cronExpression === '0 0 1 * *') return 'Monthly on the 1st at midnight'
  if (cronExpression === '0 0 1 1 *') return 'Yearly on January 1st at midnight'
  
  // More specific patterns
  if (minute === '0' && hour === '0' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    return 'Daily at midnight'
  }
  if (minute === '0' && hour !== '*' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    return `Daily at ${hour}:00`
  }
  if (minute !== '*' && hour !== '*' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
    return `Daily at ${hour.padStart(2, '0')}:${minute.padStart(2, '0')}`
  }
  if (minute === '0' && hour === '0' && dayOfMonth === '*' && month === '*' && dayOfWeek !== '*') {
    const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
    const dayNum = parseInt(dayOfWeek)
    if (dayNum >= 0 && dayNum <= 6) {
      return `Weekly on ${days[dayNum]} at midnight`
    }
  }
  
  // Handle */n patterns
  if (minute.startsWith('*/')) {
    const interval = minute.substring(2)
    if (hour === '*' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
      return `Every ${interval} minute${interval !== '1' ? 's' : ''}`
    }
  }
  if (hour.startsWith('*/')) {
    const interval = hour.substring(2)
    if (minute === '0' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
      return `Every ${interval} hour${interval !== '1' ? 's' : ''}`
    }
  }
  
  // Fallback to generic description
  let description = 'At '
  if (minute === '*') description += 'every minute'
  else if (minute.startsWith('*/')) description += `every ${minute.substring(2)} minutes`
  else description += `minute ${minute}`
  
  if (hour !== '*') {
    if (hour.startsWith('*/')) description += ` of every ${hour.substring(2)} hours`
    else description += ` of hour ${hour}`
  }
  
  return description.charAt(0).toUpperCase() + description.slice(1)
}

function getPodReplacementPolicyClass(policy: string): string {
  switch (policy) {
    case 'TerminatingOrFailed':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
    case 'Failed':
      return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300'
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
  }
}

// Helper function to convert memory to appropriate binary units (Mi, Gi, Ti)
function formatMemory(memory: string): string {
  const match = memory.match(/^(\d+)(.*)$/)
  if (!match) return memory
  
  const value = parseInt(match[1])
  const unit = match[2]
  
  // Convert to bytes first
  let bytes = value
  switch (unit) {
    case 'Ki': bytes = value * 1024; break
    case 'Mi': bytes = value * 1024 * 1024; break
    case 'Gi': bytes = value * 1024 * 1024 * 1024; break
    case 'Ti': bytes = value * 1024 * 1024 * 1024 * 1024; break
    default: bytes = value; break
  }
  
  // Convert to appropriate unit based on 1024 boundaries
  if (bytes >= 1024 * 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024 * 1024)).toFixed(1)}Ti`
  } else if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}Gi`
  } else if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)}Mi`
  } else if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)}Ki`
  } else {
    return `${bytes}B`
  }
}

async function copyJobCondition(condition: any): Promise<void> {
  try {
    const conditionText = `Job Condition: ${JSON.stringify(condition, null, 2)}`
    await navigator.clipboard.writeText(conditionText)
  } catch (error) {
    console.error('Failed to copy job condition:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `Type: ${condition.type}, Status: ${condition.status}, Reason: ${condition.reason || 'N/A'}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

function getIngressUrl(host: string | undefined, path: string | undefined): string {
  const hostname = host && host !== '*' ? host : 'example.com'
  const pathname = path || '/'
  return `https://${hostname}${pathname}`
}

async function copyIngressRule(rule: any): Promise<void> {
  try {
    const ruleText = `Ingress Rule: ${JSON.stringify(rule, null, 2)}`
    await navigator.clipboard.writeText(ruleText)
  } catch (error) {
    console.error('Failed to copy ingress rule:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `Host: ${rule.host || '*'}, Paths: ${rule.http?.paths?.length || 0}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyPDBCondition(condition: any): Promise<void> {
  try {
    const conditionText = `PodDisruptionBudget Condition: ${JSON.stringify(condition, null, 2)}`
    await navigator.clipboard.writeText(conditionText)
  } catch (error) {
    console.error('Failed to copy PDB condition:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `Type: ${condition.type}, Status: ${condition.status}, Reason: ${condition.reason || 'N/A'}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyNodeCondition(condition: any): Promise<void> {
  try {
    const conditionText = `Node Condition: ${JSON.stringify(condition, null, 2)}`
    await navigator.clipboard.writeText(conditionText)
  } catch (error) {
    console.error('Failed to copy Node condition:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `Type: ${condition.type}, Status: ${condition.status}, Reason: ${condition.reason || 'N/A'}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyRoleRule(rule: any): Promise<void> {
  try {
    const ruleText = `Role Rule: ${JSON.stringify(rule, null, 2)}`
    await navigator.clipboard.writeText(ruleText)
  } catch (error) {
    console.error('Failed to copy Role rule:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    const apiGroups = rule.apiGroups?.join(', ') || 'core'
    const resources = rule.resources?.join(', ') || 'N/A'
    const verbs = rule.verbs?.join(', ') || 'N/A'
    textArea.value = `API Groups: ${apiGroups}, Resources: ${resources}, Verbs: ${verbs}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyClusterRoleRule(rule: any): Promise<void> {
  try {
    const ruleText = `ClusterRole Rule: ${JSON.stringify(rule, null, 2)}`
    await navigator.clipboard.writeText(ruleText)
  } catch (error) {
    console.error('Failed to copy ClusterRole rule:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    const apiGroups = rule.apiGroups?.join(', ') || 'core'
    const resources = rule.resources?.join(', ') || 'N/A'
    const verbs = rule.verbs?.join(', ') || 'N/A'
    textArea.value = `API Groups: ${apiGroups}, Resources: ${resources}, Verbs: ${verbs}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyRoleBindingSubject(subject: any): Promise<void> {
  try {
    const subjectText = `RoleBinding Subject: ${JSON.stringify(subject, null, 2)}`
    await navigator.clipboard.writeText(subjectText)
  } catch (error) {
    console.error('Failed to copy RoleBinding subject:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    const kind = subject.kind || 'N/A'
    const name = subject.name || 'N/A'
    const namespace = subject.namespace ? `, Namespace: ${subject.namespace}` : ''
    const apiGroup = subject.apiGroup ? `, API Group: ${subject.apiGroup}` : ''
    textArea.value = `Kind: ${kind}, Name: ${name}${namespace}${apiGroup}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}

async function copyClusterRoleBindingSubject(subject: any): Promise<void> {
  try {
    const subjectText = `ClusterRoleBinding Subject: ${JSON.stringify(subject, null, 2)}`
    await navigator.clipboard.writeText(subjectText)
  } catch (error) {
    console.error('Failed to copy ClusterRoleBinding subject:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    const kind = subject.kind || 'N/A'
    const name = subject.name || 'N/A'
    const namespace = subject.namespace ? `, Namespace: ${subject.namespace}` : ''
    const apiGroup = subject.apiGroup ? `, API Group: ${subject.apiGroup}` : ''
    textArea.value = `Kind: ${kind}, Name: ${name}${namespace}${apiGroup}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}
</script>