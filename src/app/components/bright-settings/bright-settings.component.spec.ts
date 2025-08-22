import { ComponentFixture, TestBed } from '@angular/core/testing';

import { BrightSettingsComponent } from './bright-settings.component';

describe('BrightSettingsComponent', () => {
  let component: BrightSettingsComponent;
  let fixture: ComponentFixture<BrightSettingsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [BrightSettingsComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(BrightSettingsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
