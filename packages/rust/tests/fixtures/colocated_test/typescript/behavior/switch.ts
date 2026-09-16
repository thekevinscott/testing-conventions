export let mode = 'posix';
switch (process.platform) {
  case 'win32':
    mode = 'win';
    break;
  default:
    mode = 'posix';
}
