import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { of } from 'rxjs';
import { vi } from 'vitest';

import { MqttSettingsComponent } from './mqtt-settings.component';
import { MqttService } from '../../services/mqtt/mqtt.service';

describe('MqttSettingsComponent', () => {
  let component: MqttSettingsComponent;
  let fixture: ComponentFixture<MqttSettingsComponent>;
  let mockMqttService: any;

  beforeEach(async () => {
    mockMqttService = {
      getMqttSettings: vi.fn().mockReturnValue(
        of({
          hostname: 'localhost',
          topic: 'test/topic',
          gasTopic: 'test/gas',
          username: 'user',
          password: 'pwd',
        }),
      ),
      saveMqttSettings: vi.fn().mockResolvedValue(undefined),
      resetMqttSettings: vi.fn().mockResolvedValue(undefined),
    };

    await TestBed.configureTestingModule({
      imports: [MqttSettingsComponent],
      providers: [
        provideRouter([]),
        { provide: MqttService, useValue: mockMqttService },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(MqttSettingsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

