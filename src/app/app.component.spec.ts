import { TestBed } from '@angular/core/testing';

import { of } from 'rxjs';

import { AppComponent } from './app.component';
import { ThemeService } from './services/theme/theme.service';

describe('AppComponent', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AppComponent],
      providers: [
        {
          provide: ThemeService,
          useValue: {
            isLightMode: () => of(true),
            toggleTheme: vi.fn().mockResolvedValue(undefined),
          },
        },
      ],
    }).compileComponents();
  });

  it('should create the app', () => {
    const fixture = TestBed.createComponent(AppComponent);
    const app = fixture.componentInstance;
    expect(app).toBeTruthy();
  });
});
