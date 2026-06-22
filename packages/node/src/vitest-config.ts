import { defineConfig } from 'vitest/config';

// RED stub (#217): the shared base config is not wired up yet. The colocated
// unit test and the e2e specifier test pin the contract this must satisfy —
// they fail against this empty config until the real export lands.
export const vitestConfig = defineConfig({});
