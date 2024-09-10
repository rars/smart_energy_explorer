import { ComponentFixture, TestBed } from '@angular/core/testing';

import { GasTariffHistoryComponent } from './gas-tariff-history.component';

describe('GasTariffHistoryComponent', () => {
  let component: GasTariffHistoryComponent;
  let fixture: ComponentFixture<GasTariffHistoryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [GasTariffHistoryComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(GasTariffHistoryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
