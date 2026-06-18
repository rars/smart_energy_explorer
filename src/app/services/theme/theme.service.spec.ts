import { TestBed } from '@angular/core/testing';

import { StoreService } from '../store/store.service';
import { ThemeService } from './theme.service';

describe('ThemeService', () => {
  let service: ThemeService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        {
          provide: StoreService,
          useValue: {
            get: vi.fn().mockResolvedValue(undefined),
            safe_set: vi.fn().mockResolvedValue(undefined),
          },
        },
      ],
    });
    service = TestBed.inject(ThemeService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
