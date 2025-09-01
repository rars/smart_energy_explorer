import { ComponentFixture, TestBed } from '@angular/core/testing';

import { GasConsumptionChartComponent } from './gas-consumption-chart.component';

describe('GasConsumptionChartComponent', () => {
  let component: GasConsumptionChartComponent;
  let fixture: ComponentFixture<GasConsumptionChartComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [GasConsumptionChartComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(GasConsumptionChartComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
