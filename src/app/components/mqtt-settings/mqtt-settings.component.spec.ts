import { ComponentFixture, TestBed } from '@angular/core/testing';

import { MqttSettingsComponent } from './mqtt-settings.component';

describe('MqttSettingsComponent', () => {
  let component: MqttSettingsComponent;
  let fixture: ComponentFixture<MqttSettingsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [MqttSettingsComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(MqttSettingsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
