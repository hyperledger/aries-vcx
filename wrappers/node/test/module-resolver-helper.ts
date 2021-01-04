import * as path from 'path';

import * as appModulePath from 'app-module-path';
appModulePath.addPath(path.resolve(__dirname, '../'));
appModulePath.addPath(__dirname);
