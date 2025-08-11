import { vi } from 'vitest'

// Monaco Editor mock for testing (YAML + JSON only version)
export const editor = {
  create: vi.fn(() => ({
    dispose: vi.fn(),
    setValue: vi.fn(),
    getValue: vi.fn(() => ''),
    onDidChangeModelContent: vi.fn(() => ({ dispose: vi.fn() })),
    layout: vi.fn(),
    focus: vi.fn(),
    getModel: vi.fn(() => ({
      pushEditOperations: vi.fn(),
      setValue: vi.fn(),
      getValue: vi.fn(() => ''),
    })),
  })),
  setTheme: vi.fn(),
  defineTheme: vi.fn(),
  getModels: vi.fn(() => []),
  createModel: vi.fn(() => ({
    dispose: vi.fn(),
    setValue: vi.fn(),
    getValue: vi.fn(() => ''),
  })),
}

export const languages = {
  yaml: {},
  json: {},
  register: vi.fn(),
  setLanguageConfiguration: vi.fn(),
  setMonarchTokensProvider: vi.fn(),
}

export const Range = vi.fn()
export const Selection = vi.fn()

export default {
  editor,
  languages,
  Range,
  Selection,
}