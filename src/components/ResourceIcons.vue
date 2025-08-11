<template>
  <component :is="iconComponent" v-bind="$attrs" />
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  kind: string
}

const props = defineProps<Props>()

const iconComponent = computed(() => {
  switch (props.kind) {
    case 'Pod':
      return PodIcon
    case 'Deployment':
      return DeploymentIcon
    case 'Service':
      return ServiceIcon
    case 'ConfigMap':
      return ConfigIcon
    case 'Secret':
      return SecretIcon
    case 'Node':
      return NodeIcon
    case 'Namespace':
      return NamespaceIcon
    case 'ReplicaSet':
      return ReplicaSetIcon
    case 'DaemonSet':
      return DaemonSetIcon
    case 'StatefulSet':
      return StatefulSetIcon
    case 'Job':
      return JobIcon
    case 'CronJob':
      return CronJobIcon
    case 'Ingress':
      return IngressIcon
    case 'PersistentVolume':
    case 'PersistentVolumeClaim':
      return VolumeIcon
    case 'ServiceAccount':
      return UserIcon
    default:
      return DefaultIcon
  }
})

// Icon Components
const PodIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
      <circle cx="9" cy="9" r="2"/>
      <path d="M21 15.5C21 16.88 19.88 18 18.5 18S16 16.88 16 15.5 17.12 13 18.5 13 21 14.12 21 15.5z"/>
      <path d="M16 13V9a4 4 0 00-8 0v4"/>
    </svg>
  `
}

const DeploymentIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2L2 7v10l10 5 10-5V7L12 2z"/>
      <path d="M12 22V12"/>
      <path d="M2 7l10 5 10-5"/>
      <circle cx="8" cy="10" r="1"/>
      <circle cx="16" cy="10" r="1"/>
      <circle cx="12" cy="14" r="1"/>
    </svg>
  `
}

const ServiceIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="3"/>
      <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1"/>
      <path d="M20.2 7.8l-4.2 4.2m0 0l-4.2 4.2M3.8 7.8l4.2 4.2m0 0l4.2 4.2"/>
    </svg>
  `
}

const ConfigIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
      <path d="M14 2v6h6"/>
      <path d="M9 13h6"/>
      <path d="M9 17h3"/>
    </svg>
  `
}

const SecretIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
      <circle cx="12" cy="16" r="1"/>
      <path d="M7 11V7a5 5 0 0110 0v4"/>
    </svg>
  `
}

const NodeIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="2" y="6" width="20" height="12" rx="2"/>
      <circle cx="7" cy="12" r="2"/>
      <circle cx="17" cy="12" r="2"/>
      <path d="M12 8v8"/>
    </svg>
  `
}

const NamespaceIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
      <path d="M8 11h8"/>
      <path d="M8 15h5"/>
    </svg>
  `
}

const ReplicaSetIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="4" y="4" width="6" height="6" rx="1"/>
      <rect x="14" y="4" width="6" height="6" rx="1"/>
      <rect x="4" y="14" width="6" height="6" rx="1"/>
      <rect x="14" y="14" width="6" height="6" rx="1"/>
    </svg>
  `
}

const DaemonSetIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <circle cx="12" cy="8" r="2"/>
      <circle cx="8" cy="16" r="2"/>
      <circle cx="16" cy="16" r="2"/>
      <path d="M12 10v2"/>
      <path d="M10 14l4 0"/>
    </svg>
  `
}

const StatefulSetIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="5" width="6" height="14" rx="1"/>
      <rect x="15" y="5" width="6" height="14" rx="1"/>
      <path d="M9 12h6"/>
      <circle cx="6" cy="8" r="1"/>
      <circle cx="18" cy="8" r="1"/>
    </svg>
  `
}

const JobIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M20 6L9 17l-5-5"/>
      <rect x="2" y="3" width="20" height="14" rx="2" fill="none"/>
    </svg>
  `
}

const CronJobIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <path d="M12 6v6l4 2"/>
      <path d="M12 2v2"/>
      <path d="M12 20v2"/>
      <path d="M4.22 4.22l1.42 1.42"/>
      <path d="M18.36 18.36l1.42 1.42"/>
    </svg>
  `
}

const IngressIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/>
      <path d="M7.5 4.21l4.5 2.6 4.5-2.6"/>
      <path d="M7.5 19.79V14.6L3 12"/>
      <path d="M21 12l-4.5 2.6v5.19"/>
      <path d="M3.27 6.96L12 12.01l8.73-5.05"/>
      <path d="M12 22.08V12"/>
    </svg>
  `
}

const VolumeIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/>
      <circle cx="12" cy="12" r="4"/>
    </svg>
  `
}

const UserIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/>
      <circle cx="12" cy="7" r="4"/>
    </svg>
  `
}

const DefaultIcon = {
  template: `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
      <rect x="9" y="9" width="6" height="6"/>
    </svg>
  `
}
</script>
