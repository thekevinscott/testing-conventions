import { widget } from './widget';

process.exitCode = widget() === 'widget' ? 0 : 1;
