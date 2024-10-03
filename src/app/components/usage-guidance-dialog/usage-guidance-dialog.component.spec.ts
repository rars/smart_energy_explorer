import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UsageGuidanceDialogComponent } from './usage-guidance-dialog.component';

describe('UsageGuidanceDialogComponent', () => {
  let component: UsageGuidanceDialogComponent;
  let fixture: ComponentFixture<UsageGuidanceDialogComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [UsageGuidanceDialogComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(UsageGuidanceDialogComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
