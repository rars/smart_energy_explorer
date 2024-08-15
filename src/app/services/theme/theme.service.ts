import { DOCUMENT } from '@angular/common';
import { Inject, Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class ThemeService {
  private readonly isLightModeSubject = new BehaviorSubject<boolean>(true);

  public constructor(@Inject(DOCUMENT) private document: Document) {}

  public toggleTheme(): void {
    const newIsLightModeValue = !this.isLightModeSubject.value;

    if (newIsLightModeValue) {
      this.document.body.classList.remove('dark');
    } else {
      this.document.body.classList.add('dark');
    }

    this.isLightModeSubject.next(newIsLightModeValue);
  }

  public isLightMode(): Observable<boolean> {
    return this.isLightModeSubject.asObservable();
  }
}
