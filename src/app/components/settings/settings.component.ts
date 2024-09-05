import { CommonModule, JsonPipe, TitleCasePipe } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { FormControl, FormsModule, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatCardModule } from '@angular/material/card';
import { FormBuilder, FormGroup, FormArray } from '@angular/forms';
import { invoke } from '@tauri-apps/api/core';
import { from } from 'rxjs';
import { DateService } from '../../services/date/date.service';

@Component({
  selector: 'app-settings',
  standalone: true,
  imports: [
    JsonPipe,
    TitleCasePipe,
    ReactiveFormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatDatepickerModule,
    MatSlideToggleModule,
    MatButtonModule,
    MatCardModule,
    CommonModule,
  ],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.scss',
})
export class SettingsComponent implements OnInit {
  public form: FormGroup;
  public data?: any;

  public constructor(
    private readonly dateService: DateService,
    private readonly fb: FormBuilder,
  ) {
    this.form = this.fb.group({
      profiles: this.fb.array([]),
    });
  }

  get profiles() {
    return this.form.get('profiles') as FormArray;
  }

  public ngOnInit(): void {
    from(invoke<any[]>('get_energy_profiles', {})).subscribe((x) => {
      this.data = x;

      for (const p of x) {
        this.profiles.push(
          this.fb.group({
            name: p.name,
            startDate: new Date(p.startDate),
            isActive: p.isActive,
          }),
        );
      }
    });
  }

  public update(): void {
    /*from(
      invoke('update_energy_profile_settings', {
        energyProfileId: this.data.energyProfileId,
        startDate: this.dateService.formatISODate(
          this.startDateControl.value ?? new Date(),
        ),
        isActive: this.isActive.value,
      }),
    ).subscribe();*/
  }
}
