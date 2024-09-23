import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TermsOfUseDialogComponent } from './terms-of-use-dialog.component';

describe('TermsOfUseDialogComponent', () => {
  let component: TermsOfUseDialogComponent;
  let fixture: ComponentFixture<TermsOfUseDialogComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TermsOfUseDialogComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(TermsOfUseDialogComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
