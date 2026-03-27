import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, OnDestroy, OnInit, signal } from '@angular/core';
import { MatProgressBarModule } from '@angular/material/progress-bar';


import { UnlistenFn, listen } from '@tauri-apps/api/event';

@Component({
    selector: 'app-data-downloading',
    imports: [MatProgressBarModule, CommonModule],
    templateUrl: './data-downloading.component.html',
    styleUrl: './data-downloading.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DataDownloadingComponent implements OnInit, OnDestroy {
  protected percentage = signal<number>(0);
  protected name = signal<string>('');

  private unlistenFn?: UnlistenFn;

  public ngOnInit() {
    listen<{ percentage: number; name: string }>('downloadUpdate', (event) => {
      this.percentage.set(event.payload.percentage);
      this.name.set(event.payload.name);
    }).then((unlisten) => {
      this.unlistenFn = unlisten;
    });
  }

  public ngOnDestroy(): void {
    this.unlistenFn?.();
  }
}
