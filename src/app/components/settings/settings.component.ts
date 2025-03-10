import { CommonModule } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { FormArray, FormBuilder, FormGroup } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTooltipModule } from '@angular/material/tooltip';

import { from } from 'rxjs';

import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';

import { DateService } from '../../services/date/date.service';
import { CanComponentDeactivate } from '../../unsaved-changes.guard';
import { BrightCredentialsFormComponent } from '../bright-credentials-form/bright-credentials-form.component';

@Component({
  selector: 'app-settings',
  imports: [
    BrightCredentialsFormComponent,
    CommonModule,
    MatButtonModule,
    MatCardModule,
    MatDatepickerModule,
    MatFormFieldModule,
    MatInputModule,
    MatSlideToggleModule,
    MatTooltipModule,
    ReactiveFormsModule,
    BrightCredentialsFormComponent,
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

  public async clearAllData(): Promise<void> {
    const confirmation = await confirm(
      'This will delete all local data. Are you sure you want to perform this action?',
      { title: 'Clear local data?', kind: 'warning' },
    );

    if (confirmation) {
      await invoke<void>('clear_all_data', {});
    }
  }

  public async fetchData(): Promise<void> {
    return invoke<void>('fetch_data', {});
  }

  public async reset(): Promise<void> {
    const confirmation = await confirm(
      'This will delete all local data and restore settings to their initial state. Are you sure you want to perform this action?',
      { title: 'Reset?', kind: 'warning' },
    );

    if (confirmation) {
      await invoke<void>('reset', {});
    }
  }
}
