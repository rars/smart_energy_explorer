import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ElectricityConsumptionLineChartComponent } from './electricity-consumption-line-chart.component';

describe('ElectricityConsumptionLineChartComponent', () => {
  let component: ElectricityConsumptionLineChartComponent;
  let fixture: ComponentFixture<ElectricityConsumptionLineChartComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ElectricityConsumptionLineChartComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ElectricityConsumptionLineChartComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
