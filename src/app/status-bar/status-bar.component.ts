import { Component, OnDestroy, OnInit } from '@angular/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { BehaviorSubject, from } from 'rxjs';
import { DataDownloadingComponent } from '../components/data-downloading/data-downloading.component';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-status-bar',
  standalone: true,
  imports: [CommonModule, DataDownloadingComponent, DataDownloadingComponent],
  templateUrl: './status-bar.component.html',
  styleUrl: './status-bar.component.scss',
})
export class StatusBarComponent implements OnInit, OnDestroy {
  public isDownloading$ = new BehaviorSubject(false);

  private unlistenFn?: UnlistenFn;

  public ngOnInit(): void {
    console.log('listening...');
    listen<{ isDownloading: boolean }>('appStatusUpdate', (event) => {
      // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
      // event.payload is the payload object
      this.isDownloading$.next(event.payload.isDownloading);

      console.log(event);
    }).then((unlisten) => {
      this.unlistenFn = unlisten;
    });

    from(invoke<{ isDownloading: boolean }>('get_app_status', {})).subscribe(
      (status) => {
        this.isDownloading$.next(status.isDownloading);
      },
    );
  }

  public ngOnDestroy(): void {
    this.unlistenFn?.();
    this.isDownloading$.complete();
  }
}
