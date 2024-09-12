import { ComponentFixture, TestBed } from '@angular/core/testing';

import { EnergyCostHistoryComponent } from './energy-cost-history.component';

describe('ElectricityCostHistoryComponent', () => {
  let component: EnergyCostHistoryComponent;
  let fixture: ComponentFixture<EnergyCostHistoryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [EnergyCostHistoryComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(EnergyCostHistoryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
