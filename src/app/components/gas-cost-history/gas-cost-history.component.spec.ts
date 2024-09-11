import { ComponentFixture, TestBed } from '@angular/core/testing';

import { GasCostHistoryComponent } from './gas-cost-history.component';

describe('GasCostHistoryComponent', () => {
  let component: GasCostHistoryComponent;
  let fixture: ComponentFixture<GasCostHistoryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [GasCostHistoryComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(GasCostHistoryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
