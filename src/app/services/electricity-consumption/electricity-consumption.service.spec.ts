import { TestBed } from '@angular/core/testing';

import { ElectricityConsumptionService } from './electricity-consumption.service';

describe('ElectricityConsumptionService', () => {
  let service: ElectricityConsumptionService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ElectricityConsumptionService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
