import { type Dirent } from 'node:fs';
import { parse } from 'json5';
import { parse as parseYaml } from 'yaml';
import { basename, join } from 'node:path';
import { dirname } from 'path';

parse('{a: 1}');
parseYaml('a: 1');
basename('a/b');
join('a', 'b');
dirname('a/b');
