import { CommonModule, JsonPipe, TitleCasePipe } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
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
import { CanComponentDeactivate } from '../../unsaved-changes.guard';
import { confirm } from '@tauri-apps/plugin-dialog';
import { is } from 'date-fns/locale';

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
export class SettingsComponent implements OnInit, CanComponentDeactivate {
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
            energyProfileId: p.energyProfileId,
            name: p.name,
            startDate: new Date(p.startDate),
            isActive: p.isActive,
          }),
        );
      }
    });
  }

  public update(): void {
    if (this.form.valid && this.form.dirty) {
      this.form.disable();

      const energyProfileUpdates = this.form.value.profiles.map((x: any) => {
        return {
          energyProfileId: x.energyProfileId,
          startDate: this.dateService.formatISODate(x.startDate),
          isActive: x.isActive,
        };
      });

      from(
        invoke('update_energy_profile_settings', {
          energyProfileUpdates,
        }),
      ).subscribe(() => {
        this.form.markAsPristine();
        this.form.enable();
      });
    }
  }

  public async canDeactivate(): Promise<boolean> {
    if (this.form.dirty) {
      const confirmation = await confirm(
        'There are unsaved changes. Do you want to discard these changes?',
        { title: 'Discard changes?', kind: 'warning' },
      );
      return confirmation;
    }
    return true;
  }
}
