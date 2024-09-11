import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ElectricityCostHistoryComponent } from './electricity-cost-history.component';

describe('ElectricityCostHistoryComponent', () => {
  let component: ElectricityCostHistoryComponent;
  let fixture: ComponentFixture<ElectricityCostHistoryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ElectricityCostHistoryComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ElectricityCostHistoryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
