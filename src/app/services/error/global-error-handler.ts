import { ErrorHandler, Injectable, Injector } from '@angular/core';
import { ErrorService } from './error.service';

@Injectable()
export class GlobalErrorHandler implements ErrorHandler {
  public constructor(private readonly injector: Injector) {}

  public handleError(error: any): void {
    const errorService = this.injector.get(ErrorService);
    errorService.showError(error);
  }
}
