// Copy next to a fresh installation to test the actual npm tarball.
import { createRequire } from 'node:module';
import { verify } from './verify-api.mjs';
verify(await import('@corbet-foss/cgrade'));
verify(createRequire(import.meta.url)('@corbet-foss/cgrade'));
verify(await import('@corbet-foss/cgrade/browser'));
console.log('cgrade: installed ESM, CommonJS, and browser exports passed');
