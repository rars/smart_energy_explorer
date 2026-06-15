import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { Component } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';

import { firstValueFrom } from 'rxjs';

import { DataDownloadingComponent } from '../data-downloading/data-downloading.component';
import { StatusBarComponent } from './status-bar.component';
import {
  invoke,
  listen,
  listenHandlers,
  unlistenSpy,
} from './tauri-event-mock';

vi.mock('@tauri-apps/api/core', () => import('./tauri-event-mock'));
vi.mock('@tauri-apps/api/event', () => import('./tauri-event-mock'));
@Component({
  selector: 'app-data-downloading',
  template: '',
  standalone: true,
})
class MockDataDownloadingComponent {}

describe('StatusBarComponent', () => {
  let component: StatusBarComponent;
  let fixture: ComponentFixture<StatusBarComponent>;

  beforeEach(async () => {
    vi.clearAllMocks();
    vi.useFakeTimers();

    for (const key in listenHandlers) delete listenHandlers[key];

    (invoke as any).mockResolvedValue({ isDownloading: false });

    await TestBed.configureTestingModule({
      imports: [StatusBarComponent, MockDataDownloadingComponent],
    })
      .overrideComponent(StatusBarComponent, {
        remove: { imports: [DataDownloadingComponent] },
        add: { imports: [MockDataDownloadingComponent] },
      })
      .compileComponents();

    fixture = TestBed.createComponent(StatusBarComponent);
    component = fixture.componentInstance;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should create and fetch initial app status via invoke', async () => {
    (invoke as any).mockResolvedValueOnce({ isDownloading: true });

    fixture.detectChanges();

    await fixture.whenStable();

    expect(component).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith('get_app_status', {});

    const isDownloading = await firstValueFrom(component.isDownloading$);
    expect(isDownloading).toBe(true);
  });

  it('should register listeners for Tauri events on initialization', () => {
    fixture.detectChanges();

    expect(listen).toHaveBeenCalledWith(
      'electricityUpdate',
      expect.any(Function),
    );
    expect(listen).toHaveBeenCalledWith(
      'appStatusUpdate',
      expect.any(Function),
    );
  });

  it('should update isDownloading$ when appStatusUpdate event fires', async () => {
    fixture.detectChanges();

    if (listenHandlers['appStatusUpdate']) {
      listenHandlers['appStatusUpdate']({
        payload: { isDownloading: true },
      });
    }

    const isDownloading = await firstValueFrom(component.isDownloading$);
    expect(isDownloading).toBe(true);
  });

  it('should format electricity data message correctly on electricityUpdate event', async () => {
    fixture.detectChanges();

    const mockPayload = {
      payload: {
        electricitymeter: {
          timestamp: '2026-06-15T10:00:00Z',
          energy: { import: { day: '12.5', units: 'kWh' } },
          power: { value: '350', units: 'W' },
        },
      },
    };

    if (listenHandlers['electricityUpdate']) {
      listenHandlers['electricityUpdate'](mockPayload);
    }

    let powerMsg = '';
    let dayMsg = '';
    let updateReceived = false;

    component['electricityPower$'].subscribe((val) => (powerMsg = val));
    component['cumulativeDay$'].subscribe((val) => (dayMsg = val));
    component['electricityUpdateReceived$'].subscribe(
      (val) => (updateReceived = val),
    );

    expect(powerMsg).toBe('350 W');
    expect(dayMsg).toContain('12.5 kWh used today');
    expect(updateReceived).toBe(true);

    vi.runOnlyPendingTimers();
  });

  it('should toggle electricityUpdateReceived$ to false after a 30 second timeout delay', async () => {
    fixture.detectChanges();

    const mockPayload = {
      payload: {
        electricitymeter: {
          timestamp: new Date().toISOString(),
          energy: { import: { day: '0', units: 'kWh' } },
          power: { value: '0', units: 'W' },
        },
      },
    };

    if (listenHandlers['electricityUpdate']) {
      listenHandlers['electricityUpdate'](mockPayload);
    }

    let updateReceived: boolean | undefined;
    component['electricityUpdateReceived$'].subscribe(
      (val) => (updateReceived = val),
    );

    expect(updateReceived).toBe(true);

    vi.advanceTimersByTime(30000);

    expect(updateReceived).toBe(false);
  });

  it('should invoke unlisten functions when component is destroyed', async () => {
    fixture.detectChanges();

    await fixture.whenStable();

    component.ngOnDestroy();

    expect(unlistenSpy).toHaveBeenCalledTimes(2);
  });
});
