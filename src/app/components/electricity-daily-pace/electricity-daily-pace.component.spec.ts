import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ElectricityDailyPaceComponent } from './electricity-daily-pace.component';

describe('ElectricityDailyPaceComponent', () => {
  let component: ElectricityDailyPaceComponent;
  let fixture: ComponentFixture<ElectricityDailyPaceComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ElectricityDailyPaceComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(ElectricityDailyPaceComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
