import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ElectricityConsumptionChartComponent } from './electricity-consumption-chart.component';

describe('ElectricityConsumptionChartComponent', () => {
  let component: ElectricityConsumptionChartComponent;
  let fixture: ComponentFixture<ElectricityConsumptionChartComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ElectricityConsumptionChartComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(ElectricityConsumptionChartComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
