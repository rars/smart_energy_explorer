import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ThirdPartyLicensingComponent } from './third-party-licensing.component';

describe('ThirdPartyLicensingComponent', () => {
  let component: ThirdPartyLicensingComponent;
  let fixture: ComponentFixture<ThirdPartyLicensingComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ThirdPartyLicensingComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ThirdPartyLicensingComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
