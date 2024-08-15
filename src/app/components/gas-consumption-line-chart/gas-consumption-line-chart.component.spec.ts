import { ComponentFixture, TestBed } from '@angular/core/testing';

import { GasConsumptionLineChartComponent } from './gas-consumption-line-chart.component';

describe('GasConsumptionLineChartComponent', () => {
  let component: GasConsumptionLineChartComponent;
  let fixture: ComponentFixture<GasConsumptionLineChartComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [GasConsumptionLineChartComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(GasConsumptionLineChartComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
