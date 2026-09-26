import { parse, stringify } from 'yaml';
import { basename, format } from 'node:path';
import * as path from 'path';
import json5 from 'json5';
import 'yaml';
import { readFile } from 'node:fs';
import { exec } from 'node:child_process';
import { get } from 'undici';

parse('a: 1');
basename('a/b');
stringify({ a: 1 });
format({ root: '/' });
path.join('a', 'b');
json5.parse('{}');
readFile('x', () => {});
exec('true');
get('https://example.com');
