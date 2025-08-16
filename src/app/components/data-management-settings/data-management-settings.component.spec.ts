import { ComponentFixture, TestBed } from '@angular/core/testing';

import { DataManagementSettingsComponent } from './data-management-settings.component';

describe('DataManagementSettingsComponent', () => {
  let component: DataManagementSettingsComponent;
  let fixture: ComponentFixture<DataManagementSettingsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DataManagementSettingsComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(DataManagementSettingsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
