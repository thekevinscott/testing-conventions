export let ok = true;
try {
  JSON.parse('{');
} catch {
  ok = false;
}
