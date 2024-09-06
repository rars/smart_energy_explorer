import {
  ApplicationConfig,
  ErrorHandler,
  provideZoneChangeDetection,
} from '@angular/core';
import { provideRouter } from '@angular/router';

import { GlobalErrorHandler } from './services/error/global-error-handler';
import { routes } from './app.routes';
import {
  provideHttpClient,
  withInterceptorsFromDi,
} from '@angular/common/http';
import { BASE_PATH } from './core/modules/n3rgyapi';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideDateFnsAdapter } from '@angular/material-date-fns-adapter';
import { MAT_DATE_LOCALE } from '@angular/material/core';
import { enGB } from 'date-fns/locale';

export const appConfig: ApplicationConfig = {
  providers: [
    provideZoneChangeDetection({ eventCoalescing: true }),
    provideRouter(routes),
    provideHttpClient(withInterceptorsFromDi()),
    { provide: BASE_PATH, useValue: 'http://localhost:4200/api' },
    provideAnimationsAsync(),
    { provide: MAT_DATE_LOCALE, useValue: enGB },
    provideDateFnsAdapter(),
    { provide: ErrorHandler, useClass: GlobalErrorHandler },
  ],
};
