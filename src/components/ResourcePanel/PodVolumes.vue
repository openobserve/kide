<template>
  <div class="bg-gray-700 rounded-lg p-4">
    <h3 class="text-sm font-semibold text-gray-100 mb-3">
      Volumes
      <span class="text-xs font-normal text-gray-400 ml-2">({{ volumes.length }})</span>
    </h3>
    <div v-if="volumes.length === 0" class="text-sm text-gray-400">
      No volumes configured
    </div>
    <div v-else class="space-y-3">
      <div v-for="volume in volumes" :key="volume.name"
           class="bg-gray-800 rounded border border-gray-600 p-3">
        <div class="flex items-start justify-between gap-2">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-2">
              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-900/30 text-green-300">
                {{ volume.name }}
              </span>
              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-900/30 text-gray-300">
                {{ getVolumeType(volume) }}
              </span>
            </div>
            
            <!-- Volume details based on type -->
            <div class="text-xs space-y-1">
              <!-- ConfigMap Volume -->
              <div v-if="volume.configMap" class="space-y-1">
                <div class="text-gray-400">
                  <span class="font-medium">ConfigMap:</span> {{ volume.configMap.name }}
                </div>
                <div v-if="volume.configMap.defaultMode" class="text-gray-400">
                  <span class="font-medium">Default Mode:</span> {{ formatFileMode(volume.configMap.defaultMode) }}
                </div>
                <div v-if="volume.configMap.items?.length" class="text-gray-400">
                  <span class="font-medium">Items:</span>
                  <div class="ml-2 mt-1 space-y-1">
                    <div v-for="item in volume.configMap.items" :key="item.key" class="font-mono text-xs">
                      {{ item.key }} → {{ item.path }}
                      <span v-if="item.mode" class="text-gray-500">({{ formatFileMode(item.mode) }})</span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Secret Volume -->
              <div v-else-if="volume.secret" class="space-y-1">
                <div class="text-gray-400">
                  <span class="font-medium">Secret:</span> {{ volume.secret.secretName }}
                </div>
                <div v-if="volume.secret.defaultMode" class="text-gray-400">
                  <span class="font-medium">Default Mode:</span> {{ formatFileMode(volume.secret.defaultMode) }}
                </div>
                <div v-if="volume.secret.items?.length" class="text-gray-400">
                  <span class="font-medium">Items:</span>
                  <div class="ml-2 mt-1 space-y-1">
                    <div v-for="item in volume.secret.items" :key="item.key" class="font-mono text-xs">
                      {{ item.key }} → {{ item.path }}
                      <span v-if="item.mode" class="text-gray-500">({{ formatFileMode(item.mode) }})</span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- PersistentVolumeClaim Volume -->
              <div v-else-if="volume.persistentVolumeClaim" class="space-y-1">
                <div class="text-gray-400">
                  <span class="font-medium">PVC:</span> {{ volume.persistentVolumeClaim.claimName }}
                </div>
                <div class="text-gray-400">
                  <span class="font-medium">Read Only:</span> {{ volume.persistentVolumeClaim.readOnly ? 'Yes' : 'No' }}
                </div>
              </div>

              <!-- EmptyDir Volume -->
              <div v-else-if="volume.emptyDir" class="space-y-1">
                <div v-if="volume.emptyDir.medium" class="text-gray-400">
                  <span class="font-medium">Medium:</span> {{ volume.emptyDir.medium }}
                </div>
                <div v-if="volume.emptyDir.sizeLimit" class="text-gray-400">
                  <span class="font-medium">Size Limit:</span> {{ volume.emptyDir.sizeLimit }}
                </div>
              </div>

              <!-- HostPath Volume -->
              <div v-else-if="volume.hostPath" class="space-y-1">
                <div class="text-gray-400">
                  <span class="font-medium">Host Path:</span> 
                  <span class="font-mono">{{ volume.hostPath.path }}</span>
                </div>
                <div v-if="volume.hostPath.type" class="text-gray-400">
                  <span class="font-medium">Type:</span> {{ volume.hostPath.type }}
                </div>
              </div>

              <!-- Downward API Volume -->
              <div v-else-if="volume.downwardAPI" class="space-y-1">
                <div v-if="volume.downwardAPI.defaultMode" class="text-gray-400">
                  <span class="font-medium">Default Mode:</span> {{ formatFileMode(volume.downwardAPI.defaultMode) }}
                </div>
                <div v-if="volume.downwardAPI.items?.length" class="text-gray-400">
                  <span class="font-medium">Items:</span>
                  <div class="ml-2 mt-1 space-y-1">
                    <div v-for="item in volume.downwardAPI.items" :key="item.path" class="font-mono text-xs">
                      {{ item.path }}
                      <span v-if="item.mode" class="text-gray-500">({{ formatFileMode(item.mode) }})</span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Projected Volume -->
              <div v-else-if="volume.projected" class="space-y-1">
                <div v-if="volume.projected.defaultMode" class="text-gray-400">
                  <span class="font-medium">Default Mode:</span> {{ formatFileMode(volume.projected.defaultMode) }}
                </div>
                <div v-if="volume.projected.sources?.length" class="text-gray-400">
                  <span class="font-medium">Sources:</span> {{ volume.projected.sources.length }} source(s)
                </div>
                
                <!-- Sources Subsection -->
                <div v-if="volume.projected.sources?.length" class="mt-2 p-3 bg-blue-900/20 rounded-lg border border-blue-800">
                  <h5 class="text-xs font-semibold text-blue-300 mb-2">Source Details</h5>
                  <div class="space-y-2">
                    <div v-for="(source, index) in volume.projected.sources" :key="index" class="p-2 bg-gray-800 rounded border border-blue-700">
                      <!-- ConfigMap Source -->
                      <div v-if="source.configMap" class="space-y-1">
                        <div class="font-medium text-blue-400 text-xs">ConfigMap</div>
                        <div class="text-xs text-gray-400">
                          <span class="font-medium">Name:</span> {{ source.configMap.name }}
                        </div>
                        <div v-if="source.configMap.optional !== undefined" class="text-xs text-gray-400">
                          <span class="font-medium">Optional:</span> {{ source.configMap.optional ? 'Yes' : 'No' }}
                        </div>
                        <div v-if="source.configMap.items?.length" class="text-xs text-gray-400">
                          <span class="font-medium">Items:</span>
                          <div class="ml-2 mt-1 space-y-1">
                            <div v-for="item in source.configMap.items" :key="item.key" class="font-mono text-xs">
                              {{ item.key }} → {{ item.path }}
                              <span v-if="item.mode" class="text-gray-500">({{ formatFileMode(item.mode) }})</span>
                            </div>
                          </div>
                        </div>
                      </div>
                      
                      <!-- Secret Source -->
                      <div v-else-if="source.secret" class="space-y-1">
                        <div class="font-medium text-purple-400 text-xs">Secret Source</div>
                        <div class="text-xs text-gray-400">
                          <span class="font-medium">Name:</span> {{ source.secret.name }}
                        </div>
                        <div v-if="source.secret.optional !== undefined" class="text-xs text-gray-400">
                          <span class="font-medium">Optional:</span> {{ source.secret.optional ? 'Yes' : 'No' }}
                        </div>
                        <div v-if="source.secret.items?.length" class="text-xs text-gray-400">
                          <span class="font-medium">Items:</span>
                          <div class="ml-2 mt-1 space-y-1">
                            <div v-for="item in source.secret.items" :key="item.key" class="font-mono text-xs">
                              {{ item.key }} → {{ item.path }}
                              <span v-if="item.mode" class="text-gray-500">({{ formatFileMode(item.mode) }})</span>
                            </div>
                          </div>
                        </div>
                      </div>
                      
                      <!-- Downward API Source -->
                      <div v-else-if="source.downwardAPI" class="space-y-1">
                        <div class="font-medium text-green-400 text-xs">Downward API</div>
                        <div v-if="source.downwardAPI.items?.length" class="text-xs text-gray-400">
                          <span class="font-medium">Items:</span>
                          <div class="ml-2 mt-1 space-y-1">
                            <div v-for="item in source.downwardAPI.items" :key="item.path" class="font-mono text-xs">
                              {{ item.path }}
                              <span v-if="item.mode" class="text-gray-500">({{ formatFileMode(item.mode) }})</span>
                              <div v-if="item.fieldRef" class="text-gray-500 text-xs ml-2">
                                Field: {{ item.fieldRef.fieldPath }}
                              </div>
                              <div v-if="item.resourceFieldRef" class="text-gray-500 text-xs ml-2">
                                Resource: {{ item.resourceFieldRef.resource }}
                                <span v-if="item.resourceFieldRef.containerName"> ({{ item.resourceFieldRef.containerName }})</span>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                      
                      <!-- Service Account Token Source -->
                      <div v-else-if="source.serviceAccountToken" class="space-y-1">
                        <div class="font-medium text-orange-400 text-xs">Service Account Token</div>
                        <div class="text-xs text-gray-400">
                          <span class="font-medium">Path:</span> {{ source.serviceAccountToken.path }}
                        </div>
                        <div v-if="source.serviceAccountToken.audience" class="text-xs text-gray-400">
                          <span class="font-medium">Audience:</span> {{ source.serviceAccountToken.audience }}
                        </div>
                        <div v-if="source.serviceAccountToken.expirationSeconds" class="text-xs text-gray-400">
                          <span class="font-medium">Expiration:</span> {{ source.serviceAccountToken.expirationSeconds }}s
                        </div>
                      </div>
                      
                      <!-- Unknown Source Type -->
                      <div v-else class="text-xs text-gray-400">
                        Unknown source type
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Generic volume type -->
              <div v-else class="text-gray-400">
                <span class="font-medium">Type:</span> {{ getVolumeType(volume) }}
              </div>
            </div>

            <!-- Volume mounts in containers -->
            <div v-if="getVolumeMounts(volume.name).length > 0" class="mt-3 pt-2 border-t border-gray-600">
              <div class="text-xs font-medium text-gray-300 mb-1">Mounted in:</div>
              <div class="space-y-1">
                <div v-for="mount in getVolumeMounts(volume.name)" :key="`${mount.containerName}-${mount.mountPath}`"
                     class="flex items-center gap-2 text-xs">
                  <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-blue-900/30 text-blue-300">
                    {{ mount.containerName }}
                  </span>
                  <span class="font-mono text-gray-400">{{ mount.mountPath }}</span>
                  <span v-if="mount.readOnly" class="text-orange-400 text-xs">(RO)</span>
                  <span v-if="mount.subPath" class="text-gray-400 text-xs">subPath: {{ mount.subPath }}</span>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Copy button -->
          <button
            @click="copyVolumeInfo(volume)"
            class="p-1 text-gray-400 hover:text-gray-300 transition-colors flex-shrink-0"
            :title="`Copy volume ${volume.name} configuration`"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface VolumeMount {
  containerName: string
  mountPath: string
  readOnly?: boolean
  subPath?: string
}

interface Props {
  volumes: any[]
  containers?: any[]
  initContainers?: any[]
}

const props = defineProps<Props>()

function getVolumeType(volume: any): string {
  if (volume.configMap) return 'ConfigMap'
  if (volume.secret) return 'Secret'
  if (volume.persistentVolumeClaim) return 'PersistentVolumeClaim'
  if (volume.emptyDir) return 'EmptyDir'
  if (volume.hostPath) return 'HostPath'
  if (volume.downwardAPI) return 'DownwardAPI'
  if (volume.projected) return 'Projected'
  if (volume.nfs) return 'NFS'
  if (volume.awsElasticBlockStore) return 'AWS EBS'
  if (volume.gcePersistentDisk) return 'GCE PD'
  if (volume.azureDisk) return 'Azure Disk'
  if (volume.csi) return 'CSI'
  if (volume.flexVolume) return 'FlexVolume'
  
  // Return the first key that's not 'name'
  const keys = Object.keys(volume).filter(key => key !== 'name')
  return keys.length > 0 ? keys[0] : 'Unknown'
}

function formatFileMode(mode: number): string {
  return `0${mode.toString(8)}`
}

function getVolumeMounts(volumeName: string): VolumeMount[] {
  const mounts: VolumeMount[] = []
  
  // Check regular containers
  if (props.containers) {
    props.containers.forEach(container => {
      if (container.volumeMounts) {
        container.volumeMounts.forEach((mount: any) => {
          if (mount.name === volumeName) {
            mounts.push({
              containerName: container.name,
              mountPath: mount.mountPath,
              readOnly: mount.readOnly,
              subPath: mount.subPath
            })
          }
        })
      }
    })
  }
  
  // Check init containers
  if (props.initContainers) {
    props.initContainers.forEach(container => {
      if (container.volumeMounts) {
        container.volumeMounts.forEach((mount: any) => {
          if (mount.name === volumeName) {
            mounts.push({
              containerName: `${container.name} (init)`,
              mountPath: mount.mountPath,
              readOnly: mount.readOnly,
              subPath: mount.subPath
            })
          }
        })
      }
    })
  }
  
  return mounts
}

async function copyVolumeInfo(volume: any): Promise<void> {
  try {
    const volumeInfo = `Volume: ${volume.name}\nType: ${getVolumeType(volume)}\nConfiguration: ${JSON.stringify(volume, null, 2)}`
    await navigator.clipboard.writeText(volumeInfo)
  } catch (error) {
    console.error('Failed to copy volume info:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = `Volume: ${volume.name}\nType: ${getVolumeType(volume)}`
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}
</script>