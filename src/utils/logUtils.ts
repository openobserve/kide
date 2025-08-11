// Log processing utilities

export function extractTimestamp(line: string): string | null {
  // Match ISO 8601 timestamp at the beginning of the line
  const timestampRegex = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\s+/
  const match = line.match(timestampRegex)
  return match ? match[0] : null
}

export function extractLogContent(line: string): string {
  const timestampRegex = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\s+/
  return line.replace(timestampRegex, '')
}

export function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

export function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

export interface SearchMatch {
  lineIndex: number
  matchIndex: number
}

export class LogSearchManager {
  private searchMatches: SearchMatch[] = []
  private currentMatchIndex = 0

  updateSearchMatches(logLines: string[], searchQuery: string): void {
    this.searchMatches = []
    this.currentMatchIndex = 0
    
    if (!searchQuery.trim()) return
    
    const query = searchQuery.toLowerCase()
    
    logLines.forEach((line, lineIndex) => {
      const searchText = extractTimestamp(line) ? extractLogContent(line) : line
      const lowerLine = searchText.toLowerCase()
      let matchIndex = 0
      let startIndex = 0
      
      while ((startIndex = lowerLine.indexOf(query, startIndex)) !== -1) {
        this.searchMatches.push({ lineIndex, matchIndex })
        startIndex += query.length
        matchIndex++
      }
    })
  }

  get matches(): SearchMatch[] {
    return this.searchMatches
  }

  get currentIndex(): number {
    return this.currentMatchIndex
  }

  get hasMatches(): boolean {
    return this.searchMatches.length > 0
  }

  nextMatch(): void {
    if (this.searchMatches.length === 0) return
    this.currentMatchIndex = (this.currentMatchIndex + 1) % this.searchMatches.length
  }

  previousMatch(): void {
    if (this.searchMatches.length === 0) return
    this.currentMatchIndex = this.currentMatchIndex === 0 
      ? this.searchMatches.length - 1 
      : this.currentMatchIndex - 1
  }

  getCurrentMatch(): SearchMatch | null {
    return this.searchMatches[this.currentMatchIndex] || null
  }

  isMatchingLine(lineIndex: number): boolean {
    return this.searchMatches.some(match => match.lineIndex === lineIndex)
  }

  clearMatches(): void {
    this.searchMatches = []
    this.currentMatchIndex = 0
  }

  highlightSearchInText(text: string, lineIndex: number, searchQuery: string): string {
    if (!searchQuery.trim()) return escapeHtml(text)
    
    const query = searchQuery
    const regex = new RegExp(`(${escapeRegExp(query)})`, 'gi')
    const currentMatch = this.getCurrentMatch()
    
    let matchIndex = 0
    return escapeHtml(text).replace(regex, (match) => {
      const isCurrentMatch = currentMatch && 
        currentMatch.lineIndex === lineIndex && 
        currentMatch.matchIndex === matchIndex
      matchIndex++
      return isCurrentMatch 
        ? `<span class="bg-yellow-400 text-black font-bold">${match}</span>`
        : `<span class="bg-yellow-600 text-black">${match}</span>`
    })
  }
}