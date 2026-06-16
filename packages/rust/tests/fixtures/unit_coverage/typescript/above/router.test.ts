import { expect, test } from 'vitest';

import { route } from './router';

test('get home', () => {
  expect(route('GET', '/')).toBe('home');
});

test('get about', () => {
  expect(route('GET', '/about')).toBe('about');
});

test('get other', () => {
  expect(route('GET', '/x')).toBe('page');
});

test('post create', () => {
  expect(route('POST', '/')).toBe('create');
});

test('post accepted', () => {
  expect(route('POST', '/x')).toBe('accepted');
});
