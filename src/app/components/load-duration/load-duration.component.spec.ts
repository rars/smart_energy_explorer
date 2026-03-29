import { ComponentFixture, TestBed } from '@angular/core/testing';

import { LoadDurationComponent } from './load-duration.component';

describe('LoadDurationComponent', () => {
  let component: LoadDurationComponent;
  let fixture: ComponentFixture<LoadDurationComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [LoadDurationComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(LoadDurationComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
