import { CanDeactivateFn } from '@angular/router';

import { Observable } from 'rxjs';

export interface CanComponentDeactivate {
  canDeactivate: () => Observable<boolean> | Promise<boolean> | boolean;
}

export const unsavedChangesGuard: CanDeactivateFn<CanComponentDeactivate> = (
  component: CanComponentDeactivate,
  _currentRoute,
  _currentState,
  _nextState,
) => {
  return component.canDeactivate ? component.canDeactivate() : true;
};
