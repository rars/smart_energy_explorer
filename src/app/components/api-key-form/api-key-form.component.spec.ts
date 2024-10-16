import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ApiKeyFormComponent } from './api-key-form.component';

describe('ApiKeyFormComponent', () => {
  let component: ApiKeyFormComponent;
  let fixture: ComponentFixture<ApiKeyFormComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ApiKeyFormComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ApiKeyFormComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
