import { DOCUMENT } from '@angular/common';
import { Inject, Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { StoreService } from '../store/store.service';

@Injectable({
  providedIn: 'root',
})
export class ThemeService {
  private readonly themeStorageKey = 'theme';
  private readonly isLightModeSubject: BehaviorSubject<boolean>;

  public constructor(
    @Inject(DOCUMENT) private document: Document,
    private readonly storeService: StoreService,
  ) {
    this.isLightModeSubject = new BehaviorSubject<boolean>(
      (localStorage.getItem(this.themeStorageKey) || 'light') === 'light',
    );

    this.storeService.get(this.themeStorageKey).then((v) => {
      const isLightMode = ((v as string) || 'light') === 'light';

      this.setDocumentClass(isLightMode);

      this.isLightModeSubject.next(isLightMode);
    });
  }

  public async toggleTheme(): Promise<void> {
    const newIsLightModeValue = !this.isLightModeSubject.value;

    this.setDocumentClass(newIsLightModeValue);

    await this.storeService.set(
      this.themeStorageKey,
      newIsLightModeValue ? 'light' : 'dark',
    );

    this.isLightModeSubject.next(newIsLightModeValue);
  }

  public isLightMode(): Observable<boolean> {
    return this.isLightModeSubject.asObservable();
  }

  private setDocumentClass(isLightMode: boolean): void {
    if (isLightMode) {
      this.document.body.classList.remove('dark');
    } else {
      this.document.body.classList.add('dark');
    }
  }
}
