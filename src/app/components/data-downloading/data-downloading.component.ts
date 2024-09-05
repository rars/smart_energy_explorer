import { AsyncPipe, JsonPipe } from '@angular/common';
import { Component, OnDestroy, OnInit } from '@angular/core';
import { MatProgressBarModule } from '@angular/material/progress-bar';

import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { BehaviorSubject } from 'rxjs';

@Component({
  selector: 'app-data-downloading',
  standalone: true,
  imports: [MatProgressBarModule, AsyncPipe, JsonPipe],
  templateUrl: './data-downloading.component.html',
  styleUrl: './data-downloading.component.scss',
})
export class DataDownloadingComponent implements OnInit, OnDestroy {
  private unlistenFn?: UnlistenFn;
  public percentage$ = new BehaviorSubject<number>(0);
  public message$ = new BehaviorSubject<string>('');

  public ngOnInit() {
    listen<{ percentage: number; message: string }>(
      'downloadUpdate',
      (event) => {
        // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
        // event.payload is the payload object
        this.percentage$.next(event.payload.percentage);
        this.message$.next(event.payload.message);
        console.log(event);
      },
    ).then((unlisten) => {
      this.unlistenFn = unlisten;
    });
  }

  public ngOnDestroy(): void {
    this.unlistenFn?.();
  }
}
