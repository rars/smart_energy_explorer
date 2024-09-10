import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ElectricityTariffHistoryComponent } from './electricity-tariff-history.component';

describe('ElectricityTariffHistoryComponent', () => {
  let component: ElectricityTariffHistoryComponent;
  let fixture: ComponentFixture<ElectricityTariffHistoryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ElectricityTariffHistoryComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ElectricityTariffHistoryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
