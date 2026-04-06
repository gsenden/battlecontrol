#!/usr/bin/env node

import { generateI18n } from './i18n-generator.js';

const result = generateI18n();
console.log(`Generated ${result.outFile} with ${result.keys.length} keys in ${result.langCodes.length} languages`);
