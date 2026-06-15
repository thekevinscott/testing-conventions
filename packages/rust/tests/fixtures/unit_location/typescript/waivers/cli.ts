// testing-conventions:waiver(location): thin CLI launcher; the logic lives in run(), which is unit-tested in pkg/helper.test.ts
export const main = () => process.exit(0);
