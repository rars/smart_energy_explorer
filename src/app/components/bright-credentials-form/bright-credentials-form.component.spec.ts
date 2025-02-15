import { ComponentFixture, TestBed } from '@angular/core/testing';

import { BrightCredentialsFormComponent } from './bright-credentials-form.component';

describe('BrightCredentialsFormComponent', () => {
  let component: BrightCredentialsFormComponent;
  let fixture: ComponentFixture<BrightCredentialsFormComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [BrightCredentialsFormComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(BrightCredentialsFormComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
