import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UsageGuidanceComponent } from './usage-guidance.component';

describe('UsageGuidanceComponent', () => {
  let component: UsageGuidanceComponent;
  let fixture: ComponentFixture<UsageGuidanceComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [UsageGuidanceComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(UsageGuidanceComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
