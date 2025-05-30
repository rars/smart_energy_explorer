import { CommonModule } from '@angular/common';
import { Component, OnDestroy, OnInit } from '@angular/core';
import { MatProgressBarModule } from '@angular/material/progress-bar';

import { BehaviorSubject } from 'rxjs';

import { UnlistenFn, listen } from '@tauri-apps/api/event';

@Component({
    selector: 'app-data-downloading',
    imports: [MatProgressBarModule, CommonModule],
    templateUrl: './data-downloading.component.html',
    styleUrl: './data-downloading.component.scss'
})
export class DataDownloadingComponent implements OnInit, OnDestroy {
  protected percentage$ = new BehaviorSubject<number>(0);
  protected name$ = new BehaviorSubject<string>('');

  private unlistenFn?: UnlistenFn;

  public ngOnInit() {
    listen<{ percentage: number; name: string }>('downloadUpdate', (event) => {
      this.percentage$.next(event.payload.percentage);
      this.name$.next(event.payload.name);
    }).then((unlisten) => {
      this.unlistenFn = unlisten;
    });
  }

  public ngOnDestroy(): void {
    this.unlistenFn?.();
  }
}
