export function getAge(timestamp?: string): string {
  if (!timestamp) return 'Unknown'
  
  const now = Date.now()
  const created = new Date(timestamp).getTime()
  const diff = now - created
  
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))
  const seconds = Math.floor((diff % (1000 * 60)) / 1000)
  
  if (days > 0) return `${days}d`
  if (hours > 0) return `${hours}h`
  if (minutes > 0) return `${minutes}m`
  return `${seconds}s`
}

export function getDetailedAge(timestamp?: string): string {
  if (!timestamp) return 'Unknown'
  
  const now = Date.now()
  const created = new Date(timestamp).getTime()
  const diff = now - created
  
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))
  
  if (days > 0) return `${days} days, ${hours} hours ago`
  if (hours > 0) return `${hours} hours, ${minutes} minutes ago`
  return `${minutes} minutes ago`
}

export function formatTimestamp(timestamp?: string): string {
  if (!timestamp) return 'N/A'
  return new Date(timestamp).toLocaleString()
}